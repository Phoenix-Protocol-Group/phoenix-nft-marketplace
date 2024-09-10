use soroban_sdk::{contract, contractimpl, log, vec, Address, BytesN, Env, Vec};

use crate::{
    collection,
    error::ContractError,
    storage::{
        generate_auction_id, get_admin, get_auction_by_id, get_auctions, get_auctions_by_seller_id,
        get_currency, get_highest_bid, is_initialized, save_admin, save_auction_by_id,
        save_auction_by_seller, save_currency, set_highest_bid, set_initialized, update_admin,
        validate_input_params, Auction, AuctionStatus, HighestBid, ItemInfo,
    },
    token,
    utils::{check_auction_can_be_finalized, minimum_price_reached},
};

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    #[allow(dead_code)]
    pub fn initialize(env: Env, admin: Address, currency: Address) -> Result<(), ContractError> {
        admin.require_auth();

        if is_initialized(&env) {
            log!(&env, "Auction: Initialize: Already initialized");
            return Err(ContractError::AlreadyInitialized);
        }

        save_admin(&env, &admin);
        save_currency(&env, currency);

        set_initialized(&env);

        env.events().publish(("initialize", "admin: "), admin);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_auction(
        env: Env,
        item_info: ItemInfo,
        seller: Address,
        duration: u64,
    ) -> Result<Auction, ContractError> {
        seller.require_auth();

        let input_values = [
            &duration,
            &item_info.item_id,
            // we want to valid only valid input, in case of `None` we will simply use 1 as
            // placeholder
            &item_info.buy_now_price.unwrap_or(1),
            &item_info.minimum_price.unwrap_or(1),
        ];
        validate_input_params(&env, &input_values[..])?;

        let currency = get_currency(&env)?;
        let nft_client = collection::Client::new(&env, &item_info.collection_addr);
        let item_balance = nft_client.balance_of(&seller, &item_info.item_id);

        nft_client.set_approval_for_transfer(
            &env.current_contract_address(),
            &item_info.item_id,
            &true,
        );

        // we need at least one item to start an auction
        if item_balance < 1 {
            log!(
                &env,
                "Auction: Create Auction: Not enough balance of the item to sell"
            );
            return Err(ContractError::NotEnoughBalance);
        }

        let id = generate_auction_id(&env)?;
        let end_time = env.ledger().timestamp() + duration;

        let auction = Auction {
            id,
            item_info,
            seller: seller.clone(),
            highest_bid: None,
            end_time,
            status: AuctionStatus::Active,
            currency,
        };

        save_auction(&env, &auction)?;

        env.events()
            .publish(("create auction", "auction id: "), auction.id);
        env.events().publish(("create auction", "seller: "), seller);
        env.events().publish(("initialize", "duration: "), duration);

        Ok(auction)
    }

    #[allow(dead_code)]
    pub fn place_bid(
        env: Env,
        auction_id: u64,
        bidder: Address,
        bid_amount: u64,
    ) -> Result<(), ContractError> {
        bidder.require_auth();

        let mut auction = get_auction_by_id(&env, auction_id)?;

        if env.ledger().timestamp() > auction.end_time {
            log!(&env, "Auction: Place Bid: Auction not active: ", auction_id);
            return Err(ContractError::AuctionNotActive);
        }

        if auction.status != AuctionStatus::Active {
            log!(
                &env,
                "Auction: Place Bid: Trying to place a bid for inactive/cancelled auction with id: ", auction_id
            );
            return Err(ContractError::AuctionNotActive);
        }

        if bidder == auction.seller {
            log!(&env, "Auction Place Bid: Seller cannot place bids.");
            return Err(ContractError::InvalidBidder);
        }

        let token_client = token::Client::new(&env, &auction.currency);

        match auction.highest_bid {
            Some(current_highest_bid) if bid_amount > current_highest_bid => {
                // refund the previous highest bidder
                let old_bid_info = get_highest_bid(&env, auction_id)?;
                token_client.transfer(
                    &env.current_contract_address(),
                    &old_bid_info.bidder,
                    &(old_bid_info.bid as i128),
                );
            }
            Some(_) => {
                log!(
                    &env,
                    "Auction: Place Bid: Bid not enough. Amount bid: ",
                    bid_amount
                );
                return Err(ContractError::BidNotEnough);
            }
            None => {}
        };

        token_client.transfer(
            &bidder,
            &env.current_contract_address(),
            &(bid_amount as i128),
        );

        set_highest_bid(&env, auction_id, bid_amount, bidder.clone())?;
        // Update auction state

        auction.highest_bid = Some(bid_amount);
        save_auction(&env, &auction)?;

        env.events()
            .publish(("place bid", "auction id"), auction_id);
        env.events().publish(("place bid", "bidder"), bidder);
        env.events().publish(("place bid", "bid"), bid_amount);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn finalize_auction(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = get_auction_by_id(&env, auction_id)?;

        // Check if the auction can be finalized
        check_auction_can_be_finalized(&env, &auction)?;

        let token_client = token::Client::new(&env, &auction.currency);
        let highest_bid = get_highest_bid(&env, auction_id)?;

        if minimum_price_reached(&auction) {
            token_client.transfer(
                &env.current_contract_address(),
                &auction.seller,
                &(highest_bid.bid as i128),
            );

            let nft_client = collection::Client::new(&env, &auction.item_info.collection_addr);
            nft_client.safe_transfer_from(
                &env.current_contract_address(),
                &auction.seller,
                &highest_bid.bidder,
                &auction.item_info.item_id,
                &1,
            );

            auction.status = AuctionStatus::Ended;
            save_auction(&env, &auction)?;
            env.events()
                .publish(("finalize auction", "highest bidder: "), highest_bid.bidder);
            env.events()
                .publish(("finalize auction", "highest bid: "), highest_bid.bid);
        } else if auction.highest_bid.is_none() {
            auction.status = AuctionStatus::Ended;
            save_auction(&env, &auction)?;

            env.events().publish(("finalize auction", "no bids"), ());
        } else {
            token_client.transfer(
                &env.current_contract_address(),
                &highest_bid.bidder,
                &(highest_bid.bid as i128),
            );
            auction.status = AuctionStatus::Ended;
            save_auction(&env, &auction)?;
            log!(
                env,
                "Auction: Finalize auction: Miniminal price not reached"
            );

            env.events()
                .publish(("finalize auction", "auction id: "), auction_id);
            env.events()
                .publish(("finalize auction", "highest bid: "), auction.highest_bid);
            env.events().publish(
                ("finalize auction", "minimum price: "),
                auction.item_info.minimum_price,
            );
        };

        Ok(())
    }

    #[allow(dead_code)]
    pub fn buy_now(env: Env, auction_id: u64, buyer: Address) -> Result<(), ContractError> {
        buyer.require_auth();

        let mut auction = get_auction_by_id(&env, auction_id)?;

        if env.ledger().timestamp() > auction.end_time || auction.status != AuctionStatus::Active {
            log!(&env, "Auction: Buy Now: Auction not active: ", auction_id);
            return Err(ContractError::AuctionNotActive);
        }

        if auction.item_info.buy_now_price.is_none() {
            log!(
                env,
                "Auction: Buy Now: trying to buy an item that does not allow `buy now`"
            );
            return Err(ContractError::NoBuyNowOption);
        }

        let old_highest_bid = get_highest_bid(&env, auction_id)?;

        // refund any previous highest bid
        let token = token::Client::new(&env, &auction.currency);

        token.transfer(
            &env.current_contract_address(),
            &old_highest_bid.bidder,
            &(old_highest_bid.bid as i128),
        );

        // pay for the item
        token.transfer(
            &buyer,
            &auction.seller,
            &(auction
                .item_info
                .buy_now_price
                .expect("Auction: Buy Now: Buy now price has not been set") as i128),
        );

        let collection_client = collection::Client::new(&env, &auction.item_info.collection_addr);

        collection_client.safe_transfer_from(
            &env.current_contract_address(),
            &auction.seller,
            &buyer,
            &auction.item_info.item_id,
            &1,
        );

        auction.status = AuctionStatus::Ended;
        auction.highest_bid = Some(
            auction
                .item_info
                .buy_now_price
                .expect("Auction: Buy Now: Buy now price has not been set"),
        );

        save_auction(&env, &auction)?;

        env.events()
            .publish(("buy now", "auction id: "), auction_id);
        env.events().publish(("buy now", "buyer: "), buyer);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn pause(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = get_auction_by_id(&env, auction_id)?;
        auction.seller.require_auth();

        if auction.status != AuctionStatus::Active {
            log!(
                &env,
                "Auction: Pause: Cannot pause inactive/ended auction: ",
                auction_id
            );
            return Err(ContractError::AuctionNotActive);
        }

        if env.ledger().timestamp() > auction.end_time {
            log!(&env, "Auction: Pause: Auction expired: ", auction_id);
            return Err(ContractError::AuctionNotActive);
        }

        auction.status = AuctionStatus::Paused;

        save_auction_by_id(&env, auction_id, &auction)?;

        env.events().publish(("pause", "auction id: "), auction_id);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn unpause(env: &Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = get_auction_by_id(env, auction_id)?;
        auction.seller.require_auth();

        if auction.status != AuctionStatus::Paused {
            log!(
                &env,
                "Auction: Unpause: Cannot activate unpaused auction: ",
                auction_id
            );
            return Err(ContractError::AuctionNotPaused);
        }

        if env.ledger().timestamp() > auction.end_time {
            log!(env, "Auction: Unpause: Auction expired: ", auction_id);
            return Err(ContractError::AuctionNotActive);
        }

        auction.status = AuctionStatus::Active;

        save_auction_by_id(env, auction_id, &auction)?;

        env.events()
            .publish(("unpause", "auction id: "), auction_id);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_auction(env: Env, auction_id: u64) -> Result<Auction, ContractError> {
        let auction = get_auction_by_id(&env, auction_id)?;

        Ok(auction)
    }

    #[allow(dead_code)]
    pub fn get_active_auctions(
        env: Env,
        start_index: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<Auction>, ContractError> {
        let all_auctions = get_auctions(&env, start_index, limit)?;

        let mut filtered_auctions = vec![&env];

        for auction in all_auctions.iter() {
            if auction.status == AuctionStatus::Active {
                filtered_auctions.push_back(auction);
            }
        }

        Ok(filtered_auctions)
    }

    #[allow(dead_code)]
    pub fn get_auctions_by_seller(
        env: Env,
        seller: Address,
    ) -> Result<Vec<Auction>, ContractError> {
        let seller_auction_list = get_auctions_by_seller_id(&env, &seller)?;

        Ok(seller_auction_list)
    }

    #[allow(dead_code)]
    pub fn get_highest_bid(env: Env, auction_id: u64) -> Result<HighestBid, ContractError> {
        let highest_bid_info = get_highest_bid(&env, auction_id)?;

        Ok(highest_bid_info)
    }

    #[allow(dead_code)]
    pub fn update_admin(env: Env, new_admin: Address) -> Result<Address, ContractError> {
        let old_admin = get_admin(&env)?;
        old_admin.require_auth();

        env.events()
            .publish(("update admin", "old admin: "), old_admin);
        env.events()
            .publish(("update admin", "new admin: "), &new_admin);

        Ok(update_admin(&env, &new_admin))?
    }

    #[allow(dead_code)]
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        let admin: Address = get_admin(&env)?;
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        env.events().publish(("upgrade", "admin: "), admin);

        Ok(())
    }
}

fn save_auction(env: &Env, auction: &Auction) -> Result<(), ContractError> {
    save_auction_by_id(env, auction.id, auction)?;
    save_auction_by_seller(env, &auction.seller, auction)?;
    Ok(())
}
