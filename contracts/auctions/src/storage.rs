use soroban_sdk::{contracttype, log, panic_with_error, token, vec, Address, Env, Vec};

use crate::error::ContractError;

// Values used to extend the TTL of storage
pub const DAY_IN_LEDGERS: u32 = 17280;
pub const BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    IsInitialized,
    AuctionId,
}

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

pub fn generate_auction_id(env: &Env) -> Result<u64, ContractError> {
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

pub fn distribute_funds(env: &Env, auction: &Auction) -> Result<(), ContractError> {
    let highest_bidder = auction.highest_bidder.clone();
    let amount_due = auction.highest_bid.unwrap_or_else(|| {
        log!(
            env,
            "Auction: Distribute Funds: Missing value for highest bid."
        );
        panic_with_error!(env, ContractError::MissingHighestBid);
    });

    let rcpt = auction.seller.clone();

    let token = token::Client::new(env, &auction.currency);
    token.transfer(&highest_bidder, &rcpt, &(amount_due as i128));

    Ok(())
}

pub fn save_auction_by_id(
    env: &Env,
    auction_id: u64,
    auction: &Auction,
) -> Result<(), ContractError> {
    env.storage().instance().set(&auction_id, auction);
    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

    Ok(())
}

pub fn save_auction_by_seller(
    env: &Env,
    seller: &Address,
    auction: &Auction,
) -> Result<(), ContractError> {
    let mut seller_auctions_list: Vec<Auction> =
        env.storage().instance().get(seller).unwrap_or(vec![&env]);

    seller_auctions_list.push_back(auction.clone());

    env.storage().instance().set(seller, &seller_auctions_list);

    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

    Ok(())
}

pub fn get_auction_by_id(env: &Env, auction_id: u64) -> Result<Auction, ContractError> {
    let auction = env
        .storage()
        .instance()
        .get(&auction_id)
        .unwrap_or_else(|| {
            log!(env, "Auction: Get auction by id: Auction not present");
            panic_with_error!(&env, ContractError::AuctionNotFound);
        });
    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

    auction
}

pub fn get_auctions_by_seller_id(
    env: &Env,
    seller: &Address,
) -> Result<Vec<Auction>, ContractError> {
    let seller_auctions_list = env.storage().instance().get(seller).unwrap_or_else(|| {
        log!(env, "Auction: Get auction by seller: No auctions found");
        panic_with_error!(&env, ContractError::AuctionNotFound);
    });
    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

    Ok(seller_auctions_list)
}

pub fn update_auction(env: &Env, id: u64, auction: Auction) -> Result<(), ContractError> {
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

pub fn validate_input_params(env: &Env, values_to_check: &[&u64]) -> Result<(), ContractError> {
    values_to_check.iter().for_each(|i| {
        if i < &&1 {
            log!(&env, "Auction: Validate input: Invalid inputs used");
            panic_with_error!(&env, ContractError::InvalidInputs);
        }
    });

    Ok(())
}
