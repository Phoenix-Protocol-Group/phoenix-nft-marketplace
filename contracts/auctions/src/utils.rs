use soroban_sdk::{log, Env};

use crate::{
    collection,
    error::ContractError,
    storage::{
        save_auction_by_id, save_auction_by_seller, update_auction, Auction, AuctionStatus,
        HighestBid,
    },
    token,
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
    !auction
        .item_info
        .minimum_price
        .and_then(|min_price| {
            auction
                .highest_bid
                .map(|highest_bid| min_price > highest_bid)
        })
        .unwrap_or(false)
}

pub(crate) fn handle_minimum_price_not_reached(
    env: &Env,
    token_client: &token::Client,
    auction: &mut Auction,
    auction_id: u64,
    highest_bid: &HighestBid,
) -> Result<(), ContractError> {
    token_client.transfer(
        &env.current_contract_address(),
        &highest_bid.bidder,
        &(highest_bid.bid as i128),
    );
    auction.status = AuctionStatus::Ended;
    save_auction(env, auction_id, auction)?;
    log!(
        env,
        "Auction: Finalize auction: Miniminal price not reached"
    );
    Ok(())
}

pub(crate) fn end_auction_without_bids(
    env: &Env,
    auction: &mut Auction,
    auction_id: u64,
) -> Result<(), ContractError> {
    auction.status = AuctionStatus::Ended;
    save_auction(env, auction_id, auction)?;
    Ok(())
}

pub(crate) fn finalize_successful_auction(
    env: &Env,
    token_client: &token::Client,
    auction: &mut Auction,
    auction_id: u64,
    highest_bid: &HighestBid,
) -> Result<(), ContractError> {
    token_client.transfer(
        &env.current_contract_address(),
        &auction.seller,
        &(highest_bid.bid as i128),
    );

    let nft_client = collection::Client::new(env, &auction.item_info.collection_addr);
    nft_client.safe_transfer_from(
        &env.current_contract_address(),
        &auction.seller,
        &highest_bid.bidder,
        &auction.item_info.item_id,
        &1,
    );

    auction.status = AuctionStatus::Ended;
    save_auction(env, auction_id, auction)?;
    Ok(())
}

fn save_auction(env: &Env, auction_id: u64, auction: &Auction) -> Result<(), ContractError> {
    save_auction_by_id(env, auction_id, auction)?;
    save_auction_by_seller(env, &auction.seller, auction)?;
    update_auction(env, auction_id, auction.clone())?;
    Ok(())
}
