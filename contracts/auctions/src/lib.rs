#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, panic_with_error, token, Address,
    Env, Vec,
};

pub mod collection {
    type NftId = u64;
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_nft_collections.wasm"
    );
}

pub mod token {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
}

// Values used to extend the TTL of storage
pub const DAY_IN_LEDGERS: u32 = 17280;
pub const BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

#[derive(Clone)]
#[contracttype]
pub struct ItemInfo {
    pub item_address: Address, // Can be an NFT contract address or a collection contract address
    pub item_id: u64,
    pub minimum_price: Option<u64>,
    pub buy_now_price: Option<u64>,
}

#[derive(Clone)]
#[contracttype]
pub struct Auction {
    pub id: u64,
    pub item_info: ItemInfo,
    pub seller: Address,
    pub highest_bid: Option<u64>,
    pub highest_bidder: Address,
    pub end_time: u64,
    pub status: AuctionStatus,
    pub currency: Address,
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum AuctionStatus {
    Active,
    Ended,
    Cancelled,
    Paused,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 0,
    AuctionIdNotFound = 1,
    IDMissmatch = 2,
    BidNotEnough = 3,
    AuctionNotFinished = 4,
    NotEnoughBalance = 5,
    InvalidInputs = 6,
    AuctionNotActive = 7,
    MinPriceNotReached = 8,
    MissingHighestBid = 9,
    AuctionNotPaused = 10,
    PaymentProcessingFailed = 11,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    IsInitialized,
    AuctionId,
}

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    pub fn create_auction(
        env: Env,
        item_info: ItemInfo,
        seller: Address,
        duration: u64,
        currency: Address,
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
        Self::validate_input_params(&env, &input_values[..])?;

        let nft_client = collection::Client::new(&env, &item_info.item_address);
        let item_balance = nft_client.balance_of(&seller, &item_info.item_id);
        // we need at least one item to start an auction
        if item_balance < 1 {
            log!(
                &env,
                "Auction: Create Auction: Not enough balance of the item to sell"
            );
            return Err(ContractError::NotEnoughBalance);
        }

        let id = Self::generate_auction_id(&env)?;
        let end_time = env.ledger().timestamp() + duration;

        let auction = Auction {
            id,
            item_info,
            seller: seller.clone(),
            highest_bid: None,
            // we use the seller's address as we cannot add `Option<Address>` in the struct
            highest_bidder: seller,
            end_time,
            status: AuctionStatus::Active,
            currency,
        };

        env.storage().instance().set(&id, &auction);
        env.storage()
            .instance()
            .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

        Ok(auction)
    }

    pub fn place_bid(
        env: Env,
        auction_id: u64,
        bidder: Address,
        bid_amount: u64,
    ) -> Result<(), ContractError> {
        bidder.require_auth();

        let mut auction = Self::get_auction_by_id(&env, auction_id)?;

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

                Self::update_auction(&env, auction_id, auction)?;

                Ok(())
            }
            false => {
                log!(&env, "Auction: Place bid: Bid not enough");
                Err(ContractError::BidNotEnough)
            }
        }
    }

    pub fn finalize_auction(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = Self::get_auction_by_id(&env, auction_id)?;
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
            .is_some_and(|min_price| min_price > auction.highest_bid.unwrap())
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

        // first we try to transfer the funds from `highest_bidder` to `seller`
        // if that fails then we return an error

        let transfer_result = Self::distribute_funds(&env, &auction);
        if transfer_result.is_err() {
            log!(&env, "Auction: Finalize Auction: Payment for bid failed.");
            return Err(ContractError::PaymentProcessingFailed);
        }

        let nft_client = collection::Client::new(&env, &auction.item_info.item_address);
        nft_client.safe_transfer_from(
            &auction.seller,
            &auction.highest_bidder,
            &auction.item_info.item_id,
            &1,
        );

        auction.status = AuctionStatus::Ended;

        Self::update_auction(&env, auction_id, auction)?;

        Ok(())
    }

    pub fn buy_now(env: Env, auction_id: u64, buyer: Address) {
        todo!()
    }

    pub fn pause(env: Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = Self::get_auction_by_id(&env, auction_id)?;
        auction.seller.require_auth();

        if auction.status != AuctionStatus::Active {
            log!(env, "Auction: Pause: Cannot pause inactive/ended auction.");
            return Err(ContractError::AuctionNotActive);
        }

        auction.status = AuctionStatus::Paused;

        Self::update_auction(&env, auction_id, auction)?;

        Ok(())
    }

    pub fn unpause(env: &Env, auction_id: u64) -> Result<(), ContractError> {
        let mut auction = Self::get_auction_by_id(&env, auction_id)?;
        auction.seller.require_auth();

        if auction.status != AuctionStatus::Paused {
            log!(env, "Auction: Pause: Cannot activate unpaused auction.");
            return Err(ContractError::AuctionNotPaused);
        }

        auction.status = AuctionStatus::Active;

        Self::update_auction(&env, auction_id, auction)?;

        Ok(())
    }

    pub fn get_auction(env: Env, auction_id: u64) -> Result<Auction, ContractError> {
        let auction = Self::get_auction_by_id(&env, auction_id)?;

        Ok(auction)
    }

    pub fn get_active_auctions(env: Env) -> Vec<Auction> {
        todo!()
    }

    pub fn get_auctions_by_seller(env: Env, seller: Address) -> Vec<Auction> {
        todo!()
    }

    pub fn get_highest_bid(
        env: Env,
        auction_id: u64,
    ) -> Result<(Option<u64>, Address), ContractError> {
        let auction = Self::get_auction_by_id(&env, auction_id)?;

        Ok((auction.highest_bid, auction.highest_bidder))
    }

    fn generate_auction_id(env: &Env) -> Result<u64, ContractError> {
        let id = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::AuctionId)
            .unwrap_or_default()
            + 1u64;
        env.storage().instance().set(&DataKey::AuctionId, &id);
        env.storage()
            .instance()
            .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

        Ok(id)
    }

    fn distribute_funds(env: &Env, auction: &Auction) -> Result<(), ContractError> {
        let highest_bidder = auction.highest_bidder.clone();
        let amount_due = auction.highest_bid.unwrap_or_else(|| {
            log!(
                env,
                "Auction: Distribute Funds: Missing value for highest bid."
            );
            panic_with_error!(env, ContractError::MissingHighestBid);
        });
        let rcpt = auction.seller.clone();

        let token = token::Client::new(&env, &auction.currency);
        token.transfer(&highest_bidder, &rcpt, &(amount_due as i128));

        Ok(())
    }

    fn get_auction_by_id(env: &Env, auction_id: u64) -> Result<Auction, ContractError> {
        env.storage()
            .instance()
            .get(&auction_id)
            .unwrap_or_else(|| {
                log!(env, "Auction: Get auction by id: Auction not present");
                panic_with_error!(&env, ContractError::AuctionIdNotFound);
            })
    }

    fn update_auction(env: &Env, id: u64, auction: Auction) -> Result<(), ContractError> {
        if id != auction.id {
            log!(&env, "Auction update auction: Id missmatch");
            panic_with_error!(&env, ContractError::IDMissmatch);
        }
        env.storage().instance().set(&auction.id, &auction);
        env.storage()
            .instance()
            .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

        Ok(())
    }

    fn validate_input_params(env: &Env, values_to_check: &[&u64]) -> Result<(), ContractError> {
        values_to_check.iter().for_each(|i| {
            if i < &&1 {
                log!(&env, "Auction: Validate input: Invalid inputs used");
                panic_with_error!(&env, ContractError::InvalidInputs);
            }
        });

        Ok(())
    }
}
