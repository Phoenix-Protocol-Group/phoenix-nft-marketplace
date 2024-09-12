use soroban_sdk::{contracttype, log, panic_with_error, vec, Address, Env, Vec};

use crate::error::ContractError;

// Values used to extend the TTL of storage
pub const DAY_IN_LEDGERS: u32 = 17280;
pub const BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

// consts for Pagination
// since we start counting from 1, default would be 1 as well
pub const DEFAULT_INDEX: u64 = 1;
pub const DEFAULT_LIMIT: u64 = 10;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    IsInitialized,
    AuctionId,
    AllAuctions,
    HighestBid(u64),
    Config,
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
    pub end_time: u64,
    pub status: AuctionStatus,
    pub auction_token: Address,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct HighestBid {
    pub bid: u64,
    pub bidder: Address,
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
        .instance()
        .get(&DataKey::AuctionId)
        .expect("no previous value");

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(current_highest_id);

    let mut auctions = vec![&env];

    for id in start_index..=limit {
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

    match seller_auctions_list.iter().position(|a| a.id == auction.id) {
        Some(existing_idx) => seller_auctions_list.set(existing_idx as u32, auction.clone()),
        None => seller_auctions_list.push_back(auction.clone()),
    };

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

pub fn validate_input_params(env: &Env, values_to_check: &[&u64]) -> Result<(), ContractError> {
    values_to_check.iter().for_each(|i| {
        if i < &&1 {
            log!(
                &env,
                "Auction: Validate input: parameters cannot be less than 1"
            );
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

pub fn save_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, &admin);
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Admin, LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    let admin = env
        .storage()
        .persistent()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| {
            log!(env, "Auction: Get Admin: Admin not found");
            Err(ContractError::AdminNotFound)
        })?;
    env.storage().persistent().has(&DataKey::Admin).then(|| {
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Admin, LIFETIME_THRESHOLD, BUMP_AMOUNT);
    });

    Ok(admin)
}

pub fn update_admin(env: &Env, new_admin: &Address) -> Result<Address, ContractError> {
    env.storage().persistent().set(&DataKey::Admin, new_admin);

    Ok(new_admin.clone())
}

pub fn get_highest_bid(env: &Env, auction_id: u64) -> Result<HighestBid, ContractError> {
    let highest_bid = env
        .storage()
        .instance()
        .get(&DataKey::HighestBid(auction_id))
        .unwrap_or(HighestBid {
            bid: 0,
            // I know
            bidder: get_admin(env)?,
        });

    env.storage()
        .instance()
        .has(&DataKey::HighestBid(auction_id))
        .then(|| {
            env.storage()
                .instance()
                .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT)
        });

    Ok(highest_bid)
}

pub fn set_highest_bid(
    env: &Env,
    auction_id: u64,
    bid: u64,
    bidder: Address,
) -> Result<(), ContractError> {
    env.storage().instance().set(
        &DataKey::HighestBid(auction_id),
        &HighestBid { bid, bidder },
    );
    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

    Ok(())
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&DataKey::Config, &config);
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Config, LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

pub fn get_config(env: &Env) -> Result<Config, ContractError> {
    let config = env
        .storage()
        .persistent()
        .get(&DataKey::Config)
        .ok_or(ContractError::ConfigNotFound);

    env.storage().persistent().has(&DataKey::Config).then(|| {
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Config, LIFETIME_THRESHOLD, BUMP_AMOUNT)
    });

    Ok(config)?
}

//pub fn save_auction_token(env: &Env, auction_token: Address) {
//    env.storage()
//        .persistent()
//        .set(&DataKey::AuctionToken, &auction_token);
//
//    env.storage()
//        .persistent()
//        .extend_ttl(&DataKey::AuctionToken, LIFETIME_THRESHOLD, BUMP_AMOUNT);
//}
//
//pub fn get_auction_token(env: &Env) -> Result<Address, ContractError> {
//    let auction_token = env
//        .storage()
//        .persistent()
//        .get(&DataKey::AuctionToken)
//        .ok_or(ContractError::CurrencyNotFound)?;
//
//    env.storage()
//        .persistent()
//        .has(&DataKey::AuctionToken)
//        .then(|| {
//            env.storage().persistent().extend_ttl(
//                &DataKey::AuctionToken,
//                LIFETIME_THRESHOLD,
//                BUMP_AMOUNT,
//            );
//        });
//
//    Ok(auction_token)
//}

#[cfg(test)]
mod test {
    use soroban_sdk::Env;

    use super::validate_input_params;

    #[test]
    #[should_panic(expected = "Auction: Validate input: parameters cannot be less than 1")]
    fn validate_input_params_should_fail_with_invalid_input() {
        let env = Env::default();
        let _ = validate_input_params(&env, &[&1, &2, &3, &0]);
    }

    #[test]
    fn validate_input_params_should_work() {
        let env = Env::default();
        assert!(validate_input_params(&env, &[&1, &2, &3]).is_ok());
    }
}
