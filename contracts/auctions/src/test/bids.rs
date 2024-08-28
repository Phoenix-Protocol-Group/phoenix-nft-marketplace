use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use crate::{
    error::ContractError,
    storage::{Auction, AuctionStatus, ItemInfo},
    test::setup::{deploy_token_contract, generate_marketplace_and_collection_client, DAY, WEEKLY},
    token,
};

#[test]
fn should_place_a_bid() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);
    let bidder_a = Address::generate(&env);
    let bidder_b = Address::generate(&env);
    let bidder_c = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&bidder_a, &10i128);
    token_client.mint(&bidder_b, &20i128);
    token_client.mint(&bidder_c, &40i128);

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    mp_client.place_bid(&1, &bidder_a, &10);
    assert_eq!(
        mp_client.get_highest_bid(&1),
        (Some(10u64), bidder_a.clone())
    );
    assert_eq!(token_client.balance(&mp_client.address), 10i128);
    assert_eq!(token_client.balance(&bidder_a), 0i128);

    mp_client.place_bid(&1, &bidder_b, &20);
    assert_eq!(
        mp_client.get_highest_bid(&1),
        (Some(20u64), bidder_b.clone())
    );
    assert_eq!(token_client.balance(&mp_client.address), 20i128);
    assert_eq!(token_client.balance(&bidder_a), 10i128);
    assert_eq!(token_client.balance(&bidder_b), 0i128);

    //bidder_a tries to place a bid, that's lower than the bid of bidder_b
    assert_eq!(
        mp_client.try_place_bid(&1, &bidder_a, &15),
        Err(Ok(ContractError::BidNotEnough))
    );
    assert_eq!(token_client.balance(&mp_client.address), 20i128);
    assert_eq!(token_client.balance(&bidder_a), 10i128);
    assert_eq!(token_client.balance(&bidder_b), 0i128);

    assert_eq!(
        mp_client.get_highest_bid(&1),
        (Some(20u64), bidder_b.clone())
    );

    mp_client.place_bid(&1, &bidder_c, &40);
    assert_eq!(
        mp_client.get_highest_bid(&1),
        (Some(40u64), bidder_c.clone())
    );
    assert_eq!(token_client.balance(&mp_client.address), 40i128);
    assert_eq!(token_client.balance(&bidder_a), 10i128);
    assert_eq!(token_client.balance(&bidder_b), 20i128);
    assert_eq!(token_client.balance(&bidder_c), 0i128);
}

#[test]
fn fail_to_place_bid_when_auction_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);
    let bidder_a = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
    let token_client = token::Client::new(&env, &Address::generate(&env));

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    env.ledger().with_mut(|li| li.timestamp = WEEKLY + DAY);

    mp_client.finalize_auction(&1);

    assert_eq!(
        mp_client.try_place_bid(&1, &bidder_a, &10),
        Err(Ok(ContractError::AuctionNotActive))
    );

    // uncomment when pagination is done
    //assert_eq!(mp_client.get_active_auctions(), vec![&env]);
}

#[test]
fn fail_to_finalyze_auction_when_not_correct_state() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
    let token_client = token::Client::new(&env, &Address::generate(&env));

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    env.ledger().with_mut(|li| li.timestamp = DAY);

    mp_client.pause(&1);

    assert_eq!(
        mp_client.try_finalize_auction(&1,),
        Err(Ok(ContractError::AuctionNotActive))
    );
}

#[test]
fn finalyze_auction_when_minimal_price_not_reached_should_refund_last_bidder() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let seller = Address::generate(&env);
    let bidder_a = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&bidder_a, &5);

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    // we got the highest bid on day #1
    env.ledger().with_mut(|li| li.timestamp = DAY);
    mp_client.place_bid(&1, &bidder_a, &5);

    assert_eq!(token_client.balance(&mp_client.address), 5i128);
    assert_eq!(token_client.balance(&bidder_a), 0i128);

    // we try to finalize the auction 2 weeks later
    env.ledger().with_mut(|li| li.timestamp = WEEKLY * 2);

    assert!(mp_client.try_finalize_auction(&1).is_ok());

    assert_eq!(
        mp_client.get_auction(&1),
        Auction {
            id: 1,
            item_info,
            seller,
            highest_bid: Some(5),
            highest_bidder: bidder_a.clone(),
            end_time: WEEKLY,
            status: AuctionStatus::Ended,
            currency: token_client.address.clone()
        }
    );
    assert_eq!(token_client.balance(&mp_client.address), 0i128);
    assert_eq!(token_client.balance(&bidder_a), 5i128);
}

#[test]
fn fail_to_finalyze_auction_when_endtime_not_reached() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
    let token_client = deploy_token_contract(&env, &Address::generate(&env));

    token_client.mint(&bidder, &50);

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    mp_client.place_bid(&1, &bidder, &50);

    assert_eq!(token_client.balance(&mp_client.address), 50i128);
    assert_eq!(token_client.balance(&bidder), 0i128);
    env.ledger().with_mut(|li| li.timestamp = DAY);

    assert_eq!(
        mp_client.try_finalize_auction(&1,),
        Err(Ok(ContractError::AuctionNotFinished))
    );

    // auction is not yet over, so the bid is still in place
    assert_eq!(token_client.balance(&mp_client.address), 50i128);
    assert_eq!(token_client.balance(&bidder), 0i128);
}
