use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use crate::{
    error::ContractError,
    storage::{Auction, AuctionStatus, ItemInfo},
    test::setup::{
        deploy_token_contract, generate_marketplace_and_collection_client, DAY, FOUR_HOURS, WEEKLY,
    },
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
        generate_marketplace_and_collection_client(&env, &seller, None, None);
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
        generate_marketplace_and_collection_client(&env, &seller, None, None);
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
fn seller_tries_to_place_a_bid_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&seller, &1);
    let (mp_client, collection_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);

    collection_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collection_client.address,
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    env.ledger().with_mut(|li| li.timestamp = DAY);

    assert_eq!(
        mp_client.try_place_bid(&1, &seller, &1),
        Err(Ok(ContractError::InvalidBidder))
    );
}

#[test]
fn buy_now_should_fail_when_auction_not_active() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let fomo_buyer = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&fomo_buyer, &50);

    let (mp_client, collections_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: None,
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &DAY, &token_client.address);

    env.ledger().with_mut(|li| li.timestamp = WEEKLY);

    assert_eq!(
        mp_client.try_buy_now(&1, &fomo_buyer),
        Err(Ok(ContractError::AuctionNotActive))
    );
}

#[test]
fn buy_now_should_fail_when_no_buy_now_price_has_been_set() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let fomo_buyer = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&fomo_buyer, &50);

    let (mp_client, collections_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address,
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info, &seller, &DAY, &token_client.address);

    assert_eq!(
        mp_client.try_buy_now(&1, &fomo_buyer),
        Err(Ok(ContractError::NoBuyNowOption))
    );
}

#[test]
fn buy_now() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let bidder_a = Address::generate(&env);
    let bidder_b = Address::generate(&env);
    let fomo_buyer = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&fomo_buyer, &100);
    token_client.mint(&bidder_a, &100);
    token_client.mint(&bidder_b, &100);

    let (mp_client, collections_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    // 4 hours in and we have a first highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS);
    mp_client.place_bid(&1, &bidder_a, &5);
    assert_eq!(token_client.balance(&bidder_a), 95);
    assert_eq!(token_client.balance(&mp_client.address), 5);

    // 8 hours in and we have a second highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 2);
    mp_client.place_bid(&1, &bidder_b, &10);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 90);
    assert_eq!(token_client.balance(&mp_client.address), 10);

    // 16 hours in and we have a third highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 4);
    mp_client.place_bid(&1, &fomo_buyer, &25);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 100);
    assert_eq!(token_client.balance(&fomo_buyer), 75);
    assert_eq!(token_client.balance(&mp_client.address), 25);

    // 24 hours in and we have a 4th highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 6);
    mp_client.place_bid(&1, &bidder_b, &30);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 70);
    assert_eq!(token_client.balance(&fomo_buyer), 100);
    assert_eq!(token_client.balance(&mp_client.address), 30);

    // 36 hours in and we have a 5th highest bid, which is over the buy now price
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 9);
    mp_client.place_bid(&1, &bidder_a, &60);
    assert_eq!(token_client.balance(&bidder_a), 40);
    assert_eq!(token_client.balance(&bidder_b), 100);
    assert_eq!(token_client.balance(&fomo_buyer), 100);
    assert_eq!(token_client.balance(&mp_client.address), 60);

    // 40 hours in and the fomo buyer sees the previous user mistake and buys now
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 10);
    mp_client.buy_now(&1, &fomo_buyer);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 100);
    assert_eq!(token_client.balance(&fomo_buyer), 50);
    assert_eq!(token_client.balance(&mp_client.address), 0);
    assert_eq!(token_client.balance(&seller), 50);

    assert_eq!(
        mp_client.get_auction(&1),
        Auction {
            id: 1,
            item_info,
            seller,
            highest_bid: Some(60),
            highest_bidder: bidder_a,
            end_time: WEEKLY,
            status: AuctionStatus::Ended,
            currency: token_client.address
        }
    );

    assert_eq!(collections_client.balance_of(&fomo_buyer, &1), 1);
}

#[test]
fn pause_changes_status_and_second_attempt_fails_to_pause() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    let (mp_client, collections_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    env.ledger().with_mut(|li| li.timestamp = DAY);

    mp_client.pause(&1);

    assert_eq!(mp_client.get_auction(&1).status, AuctionStatus::Paused);

    assert_eq!(
        mp_client.try_pause(&1),
        Err(Ok(ContractError::AuctionNotActive))
    )
}
