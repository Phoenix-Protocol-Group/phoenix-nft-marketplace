use soroban_sdk::{contracttype, log, panic_with_error, vec, Address, Env, Vec};

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
    AllAuctions,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ItemInfo {
    pub collection_addr: Address,
    pub item_id: u64,
    pub minimum_price: Option<u64>,
    pub buy_now_price: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, PartialEq, Debug)]
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

// TODO: rework this get_all_auctions to use pagination and to use index: Option<u32> and limit:
// Option<u32>
pub fn get_all_auctions(env: &Env) -> Result<Vec<Auction>, ContractError> {
    let all_aucitons = env
        .storage()
        .instance()
        .get(&DataKey::AllAuctions)
        .unwrap_or_else(|| {
            log!(
                env,
                "Auctions: Get all auctions: No previous auctions found."
            );
            panic_with_error!(&env, ContractError::AuctionNotFound);
        });

    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);
    Ok(all_aucitons)
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
pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::IsInitialized)
        .unwrap_or(false)
}

pub fn set_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&DataKey::IsInitialized, &true);
}
