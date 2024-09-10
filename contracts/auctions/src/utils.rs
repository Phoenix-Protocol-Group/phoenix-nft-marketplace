use soroban_sdk::{log, Env};

use crate::{
    error::ContractError,
    storage::{Auction, AuctionStatus},
};

pub(crate) fn check_auction_can_be_finalized(
    env: &Env,
    auction: &Auction,
) -> Result<(), ContractError> {
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
    Ok(())
}

pub(crate) fn minimum_price_reached(auction: &Auction) -> bool {
    auction.item_info.minimum_price.map_or(true, |min_price| {
        auction
            .highest_bid
            .map_or(false, |highest_bid| highest_bid >= min_price)
    })
}
