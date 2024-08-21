use soroban_sdk::{log, panic_with_error, token, Env};

use crate::{
    error::ContractError,
    storage::{Auction, DataKey, BUMP_AMOUNT, LIFETIME_THRESHOLD},
};

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

pub fn get_auction_by_id(env: &Env, auction_id: u64) -> Result<Auction, ContractError> {
    env.storage()
        .instance()
        .get(&auction_id)
        .unwrap_or_else(|| {
            log!(env, "Auction: Get auction by id: Auction not present");
            panic_with_error!(&env, ContractError::AuctionIdNotFound);
        })
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
