use helpers::ttl::{PERSISTENT_RENEWAL_THRESHOLD, PERSISTENT_TARGET_TTL};
use soroban_sdk::{contracttype, log, symbol_short, vec, Address, Env, Symbol, Vec};

use crate::error::ContractError;

// consts for Pagination
// since we start counting from 1, default would be 1 as well
pub const DEFAULT_INDEX: u64 = 1;
pub const DEFAULT_LIMIT: u64 = 10;
pub const ADMIN: Symbol = symbol_short!("ADMIN");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    IsInitialized,
    AuctionId,
    AllAuctions,
    HighestBid(u64),
    Config,
    Auction(u64),
    SellerAuctions(Address),
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ItemInfo {
    pub collection_addr: Address,
    pub item_id: u64,
    pub minimum_price: Option<u64>,
    pub buy_now_price: Option<u64>,
    pub amount: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Auction {
    pub id: u64,
    pub item_info: ItemInfo,
    pub seller: Address,
    pub highest_bid: Option<u64>,
    pub end_time: u64,
    pub status: AuctionStatus,
    pub auction_token: Address,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct HighestBid {
    pub bid: u64,
    pub bidder: Option<Address>,
}

#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum AuctionStatus {
    Active,
    Ended,
    Cancelled,
    Paused,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Config {
    pub auction_token: Address,
    pub auction_creation_fee: u128,
    pub min_bid_increment: u64,
}

pub fn generate_auction_id(env: &Env) -> Result<u64, ContractError> {
    let id = env
        .storage()
        .persistent()
        .get::<_, u64>(&DataKey::AuctionId)
        .unwrap_or_default()
        + 1u64;
    env.storage().persistent().set(&DataKey::AuctionId, &id);
    env.storage().persistent().extend_ttl(
        &DataKey::AuctionId,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(id)
}

pub fn get_auctions(
    env: &Env,
    start_index: Option<u64>,
    limit: Option<u64>,
) -> Result<Vec<Auction>, ContractError> {
    let start_index = start_index.unwrap_or(DEFAULT_INDEX);

    // this is a safeguard only for the case when `DEFAULT_LIMIT` is higher than the actually
    // saved auctions and we use `None` and `None` for `start_index` and `limit`.
    // I.e. we have just 3 auctions and we want to query them
    let current_highest_id: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::AuctionId)
        .ok_or(ContractError::KeyNotFound)?;

    env.storage().persistent().extend_ttl(
        &DataKey::AuctionId,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let end = (start_index + limit - 1).min(current_highest_id);

    let mut auctions = vec![&env];

    for id in start_index..=end {
        match get_auction_by_id(env, id) {
            Ok(auction) => auctions.push_back(auction),
            Err(ContractError::AuctionNotFound) => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(auctions)
}

pub fn save_auction_by_id(
    env: &Env,
    auction_id: u64,
    auction: &Auction,
) -> Result<(), ContractError> {
    let key = DataKey::Auction(auction_id);
    env.storage().persistent().set(&key, auction);
    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(())
}

pub fn save_auction_by_seller(
    env: &Env,
    seller: &Address,
    auction: &Auction,
) -> Result<(), ContractError> {
    let key = DataKey::SellerAuctions(seller.clone());
    let mut seller_auction_ids: Vec<u64> =
        env.storage().persistent().get(&key).unwrap_or(vec![&env]);

    // Only add the ID if not already present
    if !seller_auction_ids.iter().any(|id| id == auction.id) {
        seller_auction_ids.push_back(auction.id);
    }

    env.storage()
        .persistent()
        .set(&key, &seller_auction_ids);

    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(())
}

pub fn get_auction_by_id(env: &Env, auction_id: u64) -> Result<Auction, ContractError> {
    let key = DataKey::Auction(auction_id);
    let auction: Auction = env.storage().persistent().get(&key).ok_or_else(|| {
        log!(env, "Auction: Get auction by id: Auction not present");
        ContractError::AuctionNotFound
    })?;

    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(auction)
}

pub fn get_auctions_by_seller_id(
    env: &Env,
    seller: &Address,
) -> Result<Vec<Auction>, ContractError> {
    let key = DataKey::SellerAuctions(seller.clone());
    let seller_auction_ids: Vec<u64> =
        env.storage().persistent().get(&key).ok_or_else(|| {
            log!(env, "Auction: Get auction by seller: No auctions found");
            ContractError::AuctionNotFound
        })?;

    env.storage().persistent().extend_ttl(
        &key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    let mut auctions = vec![env];
    for id in seller_auction_ids.iter() {
        auctions.push_back(get_auction_by_id(env, id)?);
    }

    Ok(auctions)
}

pub fn validate_input_params(env: &Env, values_to_check: &[&u64]) -> Result<(), ContractError> {
    for i in values_to_check.iter() {
        if **i < 1 {
            log!(
                &env,
                "Auction: Validate input: parameter is less than 1: ",
                **i
            );
            return Err(ContractError::InvalidInputs);
        }
    }

    Ok(())
}
pub fn is_initialized(env: &Env) -> bool {
    let result: bool = env
        .storage()
        .persistent()
        .get(&DataKey::IsInitialized)
        .unwrap_or(false);

    if result {
        env.storage().persistent().extend_ttl(
            &DataKey::IsInitialized,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }

    result
}

pub fn set_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&DataKey::IsInitialized, &true);

    env.storage().persistent().extend_ttl(
        &DataKey::IsInitialized,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub fn save_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&ADMIN, admin);
    env.storage().persistent().extend_ttl(
        &ADMIN,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    let admin: Address = env
        .storage()
        .persistent()
        .get(&ADMIN)
        .ok_or_else(|| {
            log!(env, "Auction: Get Admin: Admin not found");
            ContractError::AdminNotFound
        })?;

    env.storage().persistent().extend_ttl(
        &ADMIN,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(admin)
}

pub fn update_admin(env: &Env, new_admin: &Address) -> Result<Address, ContractError> {
    env.storage().persistent().set(&ADMIN, new_admin);

    env.storage().persistent().extend_ttl(
        &ADMIN,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(new_admin.clone())
}

pub fn get_highest_bid(env: &Env, auction_id: u64) -> Result<HighestBid, ContractError> {
    let key = DataKey::HighestBid(auction_id);
    let highest_bid: HighestBid = env.storage().persistent().get(&key).unwrap_or(HighestBid {
        bid: 0,
        bidder: None,
    });

    if highest_bid.bid > 0 {
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }

    Ok(highest_bid)
}

pub fn set_highest_bid(
    env: &Env,
    auction_id: u64,
    bid: u64,
    bidder: Address,
) -> Result<(), ContractError> {
    env.storage().persistent().set(
        &DataKey::HighestBid(auction_id),
        &HighestBid {
            bid,
            bidder: Some(bidder),
        },
    );

    env.storage().persistent().extend_ttl(
        &DataKey::HighestBid(auction_id),
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(())
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&DataKey::Config, &config);
    env.storage().persistent().extend_ttl(
        &DataKey::Config,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub fn get_config(env: &Env) -> Result<Config, ContractError> {
    let config: Config = env
        .storage()
        .persistent()
        .get(&DataKey::Config)
        .ok_or(ContractError::ConfigNotFound)?;

    env.storage().persistent().extend_ttl(
        &DataKey::Config,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(config)
}

#[cfg(test)]
mod test {
    use soroban_sdk::Env;

    use crate::error::ContractError;

    use super::validate_input_params;

    #[test]
    fn validate_input_params_should_fail_with_invalid_input() {
        let env = Env::default();
        assert_eq!(
            validate_input_params(&env, &[&1, &2, &3, &0]),
            Err(ContractError::InvalidInputs)
        );
    }

    #[test]
    fn validate_input_params_should_work() {
        let env = Env::default();
        assert!(validate_input_params(&env, &[&1, &2, &3]).is_ok());
    }
}
