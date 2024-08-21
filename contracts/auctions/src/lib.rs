#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, panic_with_error, Address, Env, Vec,
};

pub mod collection {
    type NftId = u64;
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_nft_collections.wasm"
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
    pub buy_now_price: u64,
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
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum AuctionStatus {
    Active,
    Ended,
    Cancelled,
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
    ) -> Result<Auction, ContractError> {
        seller.require_auth();
        // TODO: check if the input params are valid numbers
        let input_values = [
            &duration,
            &item_info.item_id,
            &item_info.buy_now_price,
            &item_info.minimum_price.unwrap_or(0),
        ];
        Self::validate_input_params(&env, &input_values[..]);

        // TODO: check if the creator actually has the item that they can sell and they own it
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
        let auction = Self::get_auction_by_id(&env, auction_id)?;
        auction.seller.require_auth();

        let curr_time = env.ledger().timestamp();

        // check if the auction min price has been reached, check if the bidder is different than
        // the seller etc.
        if curr_time > auction.end_time {
            log!(
                env,
                "Auction: Finalize auction: Auction cannot be ended early"
            );
            return Err(ContractError::AuctionNotFinished);
        }

        let nft_client = collection::Client::new(&env, &auction.item_address);

        nft_client.safe_transfer_from(
            &auction.seller,
            &auction.highest_bidder,
            &auction.item_id,
            &1,
        );
        //TODO update the auction in storage

        Ok(())
    }

    pub fn buy_now(env: Env, auction_id: u64, buyer: Address) {
        todo!()
    }

    fn distribute_funds(env: Env, auction: Auction) {
        todo!()
    }

    pub fn pause(env: Env) {
        todo!()
    }

    pub fn unpause(env: Env) {
        todo!()
    }

    pub fn get_auction(env: Env, auction_id: u64) -> Auction {
        todo!()
    }

    pub fn get_active_auctions(env: Env) -> Vec<Auction> {
        todo!()
    }

    pub fn get_auctions_by_seller(env: Env, seller: Address) -> Vec<Auction> {
        todo!()
    }

    pub fn get_highest_bid(env: Env, auction_id: u64) -> (u64, Address) {
        todo!()
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
