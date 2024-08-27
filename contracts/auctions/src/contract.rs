use soroban_sdk::{contract, contractimpl, log, vec, Address, Env, Vec};

use crate::{
    collection,
    error::ContractError,
    storage::{
        distribute_funds, generate_auction_id, get_all_auctions, get_auction_by_id,
        get_auctions_by_seller_id, is_initialized, save_auction_by_id, save_auction_by_seller,
        set_initialized, update_auction, validate_input_params, Auction, AuctionStatus, ItemInfo,
    },
    token,
};

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    #[allow(dead_code)]
    pub fn create_auction(
        env: Env,
        item_info: ItemInfo,
        seller: Address,
        duration: u64,
        currency: Address,
    ) -> Result<Auction, ContractError> {
        seller.require_auth();

        if is_initialized(&env) {
            log!(&env, "Collections: Initialize: Already initialized");
            return Err(ContractError::AlreadyInitialized);
        }

        let input_values = [
            &duration,
            &item_info.item_id,
            // we want to valid only valid input, in case of `None` we will simply use 1 as
            // placeholder
            &item_info.buy_now_price.unwrap_or(1),
            &item_info.minimum_price.unwrap_or(1),
        ];
        validate_input_params(&env, &input_values[..])?;

        let nft_client = collection::Client::new(&env, &item_info.collection_addr);
        let item_balance = nft_client.balance_of(&seller, &item_info.item_id);
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
            // we use the seller's address as we cannot add `Option<Address>` in the struct
            highest_bidder: seller.clone(),
            end_time,
            status: AuctionStatus::Active,
            currency,
        };

        // TODO: maybe these two can be merged into one key $(auction id, seller)
        save_auction_by_id(&env, id, &auction)?;
        save_auction_by_seller(&env, &seller, &auction)?;

        set_initialized(&env);

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

        if auction.status != AuctionStatus::Active {
            log!(
                &env,
                "Auction: Place Bid: Trying to place a bid for inactive/cancelled auction."
            );
            return Err(ContractError::AuctionNotActive);
        }

        match Some(bid_amount) > auction.highest_bid {
            true => {
                auction.highest_bid = Some(bid_amount);
                auction.highest_bidder = bidder;

                update_auction(&env, auction_id, auction)?;

                Ok(())
            }
            false => {
                log!(&env, "Auction: Place bid: Bid not enough");
                Err(ContractError::BidNotEnough)
            }
        }
    }

    #[allow(dead_code)]
    pub fn finalize_auction(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = get_auction_by_id(&env, auction_id)?;
        auction.seller.require_auth();

        let curr_time = env.ledger().timestamp();

        if auction.status != AuctionStatus::Active {
            log!(
                &env,
                "Auction: Finalize auction: Cannot finalize an inactive/ended auction."
            );
            return Err(ContractError::AuctionNotActive);
        }

        if auction
            .item_info
            .minimum_price
            .and_then(|min_price| {
                auction
                    .highest_bid
                    .map(|highest_bid| min_price > highest_bid)
            })
            .unwrap_or(false)
        {
            log!(
                &env,
                "Auction: Finalize auction: Miniminal price not reached"
            );
            return Err(ContractError::MinPriceNotReached);
        }

        if curr_time > auction.end_time {
            log!(
                env,
                "Auction: Finalize auction: Auction cannot be ended early"
            );
            return Err(ContractError::AuctionNotFinished);
        }

        // if the auction is over, but there are no bids placed, we just end it
        if auction.highest_bid.is_none() {
            auction.status = AuctionStatus::Ended;
            //NOTE: we have 3 diferent storage types, it's either this or we have to
            //use a single function to update all 3 storages
            // TODO: get rid of the save_auction (below)
            save_auction_by_id(&env, auction_id, &auction)?;
            save_auction_by_seller(&env, &auction.seller, &auction)?;

            return Ok(());
        }

        // first we try to transfer the funds from `highest_bidder` to `seller`
        // if that fails then we return an error

        let tx_result = distribute_funds(&env, &auction);
        if tx_result.is_err() {
            log!(&env, "Auction: Finalize Auction: Payment for bid failed.");
            return Err(ContractError::PaymentProcessingFailed);
        }

        let nft_client = collection::Client::new(&env, &auction.item_info.collection_addr);
        nft_client.safe_transfer_from(
            &auction.seller,
            &auction.highest_bidder,
            &auction.item_info.item_id,
            &1,
        );

        auction.status = AuctionStatus::Ended;

        update_auction(&env, auction_id, auction)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn buy_now(env: Env, auction_id: u64, buyer: Address) -> Result<(), ContractError> {
        buyer.require_auth();

        let mut auction = get_auction_by_id(&env, auction_id)?;
        if auction.item_info.buy_now_price.is_none() {
            log!(
                env,
                "Auction: Buy now: trying to buy an item that does not allow `buy now`"
            );
            return Err(ContractError::NoBuyNowOption);
        }

        // we should probably pause the auction while the tx is happening?
        let token = token::Client::new(&env, &auction.currency);
        token.transfer(
            &buyer,
            &auction.seller,
            &(auction.item_info.buy_now_price.unwrap() as i128),
        );

        auction.status = AuctionStatus::Ended;

        update_auction(&env, auction_id, auction)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn pause(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = get_auction_by_id(&env, auction_id)?;
        auction.seller.require_auth();

        if auction.status != AuctionStatus::Active {
            log!(env, "Auction: Pause: Cannot pause inactive/ended auction.");
            return Err(ContractError::AuctionNotActive);
        }

        auction.status = AuctionStatus::Paused;

        update_auction(&env, auction_id, auction)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn unpause(env: &Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = get_auction_by_id(env, auction_id)?;
        auction.seller.require_auth();

        if auction.status != AuctionStatus::Paused {
            log!(env, "Auction: Pause: Cannot activate unpaused auction.");
            return Err(ContractError::AuctionNotPaused);
        }

        auction.status = AuctionStatus::Active;

        update_auction(env, auction_id, auction)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_auction(env: Env, auction_id: u64) -> Result<Auction, ContractError> {
        let auction = get_auction_by_id(&env, auction_id)?;

        Ok(auction)
    }

    #[allow(dead_code)]
    pub fn get_active_auctions(env: Env) -> Result<Vec<Auction>, ContractError> {
        let all_auctions = get_all_auctions(&env)?;

        let mut filtered_auctions = vec![&env];

        // FromIterator not implemeneted for soroban_vec::<Auction>
        // let filtered_auctions = get_all_auctions(&env)?
        //    .into_iter()
        //    .filter(|auction| auction.status == AuctionStatus::Active)
        //    .collect::<Vec<Auction>>();

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
    pub fn get_highest_bid(
        env: Env,
        auction_id: u64,
    ) -> Result<(Option<u64>, Address), ContractError> {
        let auction = get_auction_by_id(&env, auction_id)?;

        Ok((auction.highest_bid, auction.highest_bidder))
    }
}
