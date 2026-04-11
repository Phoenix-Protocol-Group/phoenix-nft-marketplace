use helpers::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
use soroban_sdk::{contract, contractevent, contractimpl, log, vec, Address, BytesN, Env, Vec};

use crate::{
    collection,
    error::ContractError,
    storage::{
        generate_auction_id, get_admin, get_auction_by_id, get_auctions,
        get_auctions_by_seller_id, get_config, get_highest_bid, is_initialized, save_admin,
        save_auction_by_id, save_auction_by_seller, save_config, set_highest_bid, set_initialized,
        update_admin, validate_input_params, Auction, AuctionStatus, Config, HighestBid, ItemInfo,
    },
    token,
};

// ---- Contract Events ----

#[contractevent]
pub struct InitializeEvent {
    pub admin: Address,
}

#[contractevent]
pub struct CreateAuctionEvent {
    pub auction_id: u64,
    pub seller: Address,
    pub duration: u64,
}

#[contractevent]
pub struct PlaceBidEvent {
    pub auction_id: u64,
    pub bidder: Address,
    pub bid: u64,
}

#[contractevent]
pub struct FinalizeAuctionEvent {
    pub auction_id: u64,
    pub highest_bidder: Address,
    pub highest_bid: u64,
}

#[contractevent]
pub struct FinalizeNoBidsEvent {
    pub auction_id: u64,
}

#[contractevent]
pub struct MinPriceNotMetEvent {
    pub auction_id: u64,
}

#[contractevent]
pub struct BuyNowEvent {
    pub auction_id: u64,
    pub buyer: Address,
}

#[contractevent]
pub struct PauseEvent {
    pub auction_id: u64,
}

#[contractevent]
pub struct UnpauseEvent {
    pub auction_id: u64,
}

#[contractevent]
pub struct CancelAuctionEvent {
    pub auction_id: u64,
}

#[contractevent]
pub struct WithdrawFeesEvent {
    pub recipient: Address,
    pub amount: u64,
}

#[contractevent]
pub struct UpdateAdminEvent {
    pub old_admin: Address,
    pub new_admin: Address,
}

#[contractevent]
pub struct UpgradeEvent {
    pub admin: Address,
}

// ---- Helpers ----

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
}

// ---- Contract ----

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        auction_token: Address,
        auction_creation_fee: u128,
        min_bid_increment: u64,
    ) -> Result<(), ContractError> {
        admin.require_auth();

        if is_initialized(&env) {
            log!(&env, "Auction: Initialize: Already initialized");
            return Err(ContractError::AlreadyInitialized);
        }

        save_admin(&env, &admin);

        let config = Config {
            auction_token,
            auction_creation_fee,
            min_bid_increment,
        };

        save_config(&env, config);

        set_initialized(&env);

        InitializeEvent {
            admin,
        }
        .publish(&env);

        Ok(())
    }

    pub fn create_auction(
        env: Env,
        item_info: ItemInfo,
        seller: Address,
        duration: u64,
    ) -> Result<Auction, ContractError> {
        seller.require_auth();
        extend_instance_ttl(&env);

        let input_values = [
            &duration,
            &item_info.item_id,
            &item_info.buy_now_price.unwrap_or(1),
            &item_info.minimum_price.unwrap_or(1),
            &item_info.amount,
        ];

        validate_input_params(&env, &input_values[..])?;

        let config = get_config(&env)?;
        let auction_token = config.auction_token;
        let auction_creation_fee = config.auction_creation_fee as i128;

        let token_client = token::Client::new(&env, &auction_token);

        if token_client.balance(&seller) < auction_creation_fee {
            log!(
                &env,
                "Auction: Create Auction: Not enough balance to cover the auction creation fee. ",
                "Required: ",
                auction_creation_fee
            );
            return Err(ContractError::AuctionCreationFeeNotCovered);
        }

        token_client.transfer(
            &seller,
            env.current_contract_address(),
            &auction_creation_fee,
        );

        let nft_client = collection::Client::new(&env, &item_info.collection_addr);
        let item_balance = nft_client.balance_of(&seller, &item_info.item_id);

        if item_balance < item_info.amount {
            log!(
                &env,
                "Auction: Create Auction: Not enough balance of the item to sell"
            );
            return Err(ContractError::NotEnoughBalance);
        }

        // Escrow the NFT into the contract
        nft_client.safe_transfer_from(
            &seller,
            &seller,
            &env.current_contract_address(),
            &item_info.item_id,
            &item_info.amount,
        );

        let id = generate_auction_id(&env)?;
        let end_time = env.ledger().timestamp() + duration;

        let auction = Auction {
            id,
            item_info,
            seller: seller.clone(),
            highest_bid: None,
            end_time,
            status: AuctionStatus::Active,
            auction_token,
        };

        save_auction(&env, &auction)?;

        CreateAuctionEvent {
            auction_id: auction.id,
            seller,
            duration,
        }
        .publish(&env);

        Ok(auction)
    }

    pub fn place_bid(
        env: Env,
        auction_id: u64,
        bidder: Address,
        bid_amount: u64,
    ) -> Result<(), ContractError> {
        bidder.require_auth();
        extend_instance_ttl(&env);

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

        let config = get_config(&env)?;
        let token_client = token::Client::new(&env, &auction.auction_token);

        match auction.highest_bid {
            Some(current_highest_bid)
                if bid_amount >= current_highest_bid + config.min_bid_increment =>
            {
                // refund the previous highest bidder
                let old_bid_info = get_highest_bid(&env, auction_id)?;
                token_client.transfer(
                    &env.current_contract_address(),
                    &old_bid_info.bidder.ok_or(ContractError::BidderNotFound)?,
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
            env.current_contract_address(),
            &(bid_amount as i128),
        );

        set_highest_bid(&env, auction_id, bid_amount, bidder.clone())?;

        auction.highest_bid = Some(bid_amount);
        save_auction(&env, &auction)?;

        PlaceBidEvent {
            auction_id,
            bidder,
            bid: bid_amount,
        }
        .publish(&env);

        Ok(())
    }

    pub fn finalize_auction(env: Env, auction_id: u64) -> Result<(), ContractError> {
        extend_instance_ttl(&env);

        let mut auction = get_auction_by_id(&env, auction_id)?;

        if auction.status != AuctionStatus::Active {
            log!(
                env,
                "Auction: Finalize auction: Cannot finalize an inactive/ended auction."
            );
            return Err(ContractError::AuctionNotActive);
        }
        if env.ledger().timestamp() < auction.end_time {
            log!(
                env,
                "Auction: Finalize auction: Auction cannot be ended early"
            );
            return Err(ContractError::AuctionNotFinished);
        }

        let token_client = token::Client::new(&env, &auction.auction_token);
        let nft_client = collection::Client::new(&env, &auction.item_info.collection_addr);
        let highest_bid = get_highest_bid(&env, auction_id)?;

        if auction.item_info.minimum_price.is_none_or(|min_price| {
            auction
                .highest_bid
                .is_some_and(|highest_bid| highest_bid >= min_price)
        }) {
            token_client.transfer(
                &env.current_contract_address(),
                &auction.seller,
                &(highest_bid.bid as i128),
            );

            let winner = highest_bid
                .bidder
                .as_ref()
                .ok_or(ContractError::BidderNotFound)?
                .clone();
            nft_client.safe_transfer_from(
                &env.current_contract_address(),
                &env.current_contract_address(),
                &winner,
                &auction.item_info.item_id,
                &auction.item_info.amount,
            );

            auction.status = AuctionStatus::Ended;
            save_auction(&env, &auction)?;

            FinalizeAuctionEvent {
                auction_id,
                highest_bidder: winner,
                highest_bid: highest_bid.bid,
            }
            .publish(&env);
        } else if auction.highest_bid.is_none() {
            nft_client.safe_transfer_from(
                &env.current_contract_address(),
                &env.current_contract_address(),
                &auction.seller,
                &auction.item_info.item_id,
                &auction.item_info.amount,
            );

            auction.status = AuctionStatus::Ended;
            save_auction(&env, &auction)?;

            FinalizeNoBidsEvent { auction_id }.publish(&env);
        } else {
            token_client.transfer(
                &env.current_contract_address(),
                &highest_bid.bidder.ok_or(ContractError::BidderNotFound)?,
                &(highest_bid.bid as i128),
            );

            nft_client.safe_transfer_from(
                &env.current_contract_address(),
                &env.current_contract_address(),
                &auction.seller,
                &auction.item_info.item_id,
                &auction.item_info.amount,
            );

            auction.status = AuctionStatus::Ended;
            save_auction(&env, &auction)?;
            log!(env, "Auction: Finalize auction: Minimum price not reached");

            MinPriceNotMetEvent { auction_id }.publish(&env);
        };

        Ok(())
    }

    pub fn buy_now(env: Env, auction_id: u64, buyer: Address) -> Result<(), ContractError> {
        buyer.require_auth();
        extend_instance_ttl(&env);

        let mut auction = get_auction_by_id(&env, auction_id)?;

        if env.ledger().timestamp() > auction.end_time || auction.status != AuctionStatus::Active {
            log!(&env, "Auction: Buy Now: Auction not active: ", auction_id);
            return Err(ContractError::AuctionNotActive);
        }

        let buy_now_price = match auction.item_info.buy_now_price {
            Some(price) => price,
            None => {
                log!(
                    env,
                    "Auction: Buy Now: trying to buy an item that does not allow `buy now`"
                );
                return Err(ContractError::NoBuyNowOption);
            }
        };

        let old_highest_bid = get_highest_bid(&env, auction_id)?;

        let token = token::Client::new(&env, &auction.auction_token);

        if old_highest_bid.bid > 0 {
            token.transfer(
                &env.current_contract_address(),
                &old_highest_bid
                    .bidder
                    .ok_or(ContractError::BidderNotFound)?,
                &(old_highest_bid.bid as i128),
            );
        }

        token.transfer(&buyer, &auction.seller, &(buy_now_price as i128));

        let collection_client = collection::Client::new(&env, &auction.item_info.collection_addr);
        collection_client.safe_transfer_from(
            &env.current_contract_address(),
            &env.current_contract_address(),
            &buyer,
            &auction.item_info.item_id,
            &auction.item_info.amount,
        );

        auction.status = AuctionStatus::Ended;
        auction.highest_bid = Some(buy_now_price);

        save_auction(&env, &auction)?;

        BuyNowEvent {
            auction_id,
            buyer,
        }
        .publish(&env);

        Ok(())
    }

    pub fn pause(env: Env, auction_id: u64) -> Result<(), ContractError> {
        extend_instance_ttl(&env);
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

        save_auction(&env, &auction)?;

        PauseEvent { auction_id }.publish(&env);

        Ok(())
    }

    pub fn unpause(env: Env, auction_id: u64) -> Result<(), ContractError> {
        extend_instance_ttl(&env);
        let mut auction = get_auction_by_id(&env, auction_id)?;
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
            log!(&env, "Auction: Unpause: Auction expired: ", auction_id);
            return Err(ContractError::AuctionNotActive);
        }

        auction.status = AuctionStatus::Active;

        save_auction(&env, &auction)?;

        UnpauseEvent { auction_id }.publish(&env);

        Ok(())
    }

    pub fn cancel_auction(env: Env, auction_id: u64) -> Result<(), ContractError> {
        extend_instance_ttl(&env);

        let mut auction = get_auction_by_id(&env, auction_id)?;
        auction.seller.require_auth();

        if auction.status != AuctionStatus::Active && auction.status != AuctionStatus::Paused {
            log!(
                &env,
                "Auction: Cancel: Cannot cancel ended auction: ",
                auction_id
            );
            return Err(ContractError::AuctionNotActive);
        }

        if env.ledger().timestamp() > auction.end_time {
            log!(
                &env,
                "Auction: Cancel: Auction already expired: ",
                auction_id
            );
            return Err(ContractError::AuctionStillActive);
        }

        let nft_client = collection::Client::new(&env, &auction.item_info.collection_addr);
        let token_client = token::Client::new(&env, &auction.auction_token);

        let highest_bid = get_highest_bid(&env, auction_id)?;
        if highest_bid.bid > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &highest_bid.bidder.ok_or(ContractError::BidderNotFound)?,
                &(highest_bid.bid as i128),
            );
        }

        nft_client.safe_transfer_from(
            &env.current_contract_address(),
            &env.current_contract_address(),
            &auction.seller,
            &auction.item_info.item_id,
            &auction.item_info.amount,
        );

        auction.status = AuctionStatus::Cancelled;
        save_auction(&env, &auction)?;

        CancelAuctionEvent { auction_id }.publish(&env);

        Ok(())
    }

    pub fn withdraw_fees(env: Env, recipient: Address, amount: i128) -> Result<(), ContractError> {
        extend_instance_ttl(&env);

        let admin = get_admin(&env)?;
        admin.require_auth();

        let config = get_config(&env)?;
        let token_client = token::Client::new(&env, &config.auction_token);

        token_client.transfer(&env.current_contract_address(), &recipient, &amount);

        WithdrawFeesEvent {
            recipient,
            amount: amount as u64,
        }
        .publish(&env);

        Ok(())
    }

    pub fn get_auction(env: Env, auction_id: u64) -> Result<Auction, ContractError> {
        extend_instance_ttl(&env);
        get_auction_by_id(&env, auction_id)
    }

    pub fn get_active_auctions(
        env: Env,
        start_index: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<Auction>, ContractError> {
        extend_instance_ttl(&env);

        let all_auctions = get_auctions(&env, start_index, limit)?;

        let mut filtered_auctions = vec![&env];

        for auction in all_auctions.iter() {
            if auction.status == AuctionStatus::Active {
                filtered_auctions.push_back(auction);
            }
        }

        Ok(filtered_auctions)
    }

    pub fn get_auctions_by_seller(
        env: Env,
        seller: Address,
    ) -> Result<Vec<Auction>, ContractError> {
        extend_instance_ttl(&env);
        get_auctions_by_seller_id(&env, &seller)
    }

    pub fn get_highest_bid(env: Env, auction_id: u64) -> Result<HighestBid, ContractError> {
        extend_instance_ttl(&env);
        get_highest_bid(&env, auction_id)
    }

    pub fn update_admin(env: Env, new_admin: Address) -> Result<Address, ContractError> {
        extend_instance_ttl(&env);

        let old_admin = get_admin(&env)?;
        old_admin.require_auth();

        UpdateAdminEvent {
            old_admin,
            new_admin: new_admin.clone(),
        }
        .publish(&env);

        Ok(update_admin(&env, &new_admin))?
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        let admin: Address = get_admin(&env)?;
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        UpgradeEvent { admin }.publish(&env);

        Ok(())
    }
}

fn save_auction(env: &Env, auction: &Auction) -> Result<(), ContractError> {
    save_auction_by_id(env, auction.id, auction)?;
    save_auction_by_seller(env, &auction.seller, auction)?;
    Ok(())
}
