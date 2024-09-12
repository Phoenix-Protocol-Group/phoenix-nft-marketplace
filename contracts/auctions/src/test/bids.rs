use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env,
};

use crate::{
    contract::{MarketplaceContract, MarketplaceContractClient},
    error::ContractError,
    storage::{Auction, AuctionStatus, HighestBid, ItemInfo},
    test::setup::{
        deploy_token_contract, generate_marketplace_and_collection_client, DAY, FOUR_HOURS, WEEKLY,
    },
};

use super::setup::create_and_initialize_collection;

#[test]
fn should_place_a_bid() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);
    let bidder_a = Address::generate(&env);
    let bidder_b = Address::generate(&env);
    let bidder_c = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&seller, &10);
    let (mp_client, nft_collection_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );
    token_client.mint(&bidder_a, &10i128);
    token_client.mint(&bidder_b, &20i128);
    token_client.mint(&bidder_c, &40i128);

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    mp_client.place_bid(&1, &bidder_a, &10);
    assert_eq!(
        mp_client.get_highest_bid(&1),
        HighestBid {
            bid: 10u64,
            bidder: bidder_a.clone()
        }
    );
    assert_eq!(token_client.balance(&mp_client.address), 20i128);
    assert_eq!(token_client.balance(&bidder_a), 0i128);

    mp_client.place_bid(&1, &bidder_b, &20);
    assert_eq!(
        mp_client.get_highest_bid(&1),
        HighestBid {
            bid: 20u64,
            bidder: bidder_b.clone()
        }
    );
    assert_eq!(token_client.balance(&mp_client.address), 30i128);
    assert_eq!(token_client.balance(&bidder_a), 10i128);
    assert_eq!(token_client.balance(&bidder_b), 0i128);

    //bidder_a tries to place a bid, that's lower than the bid of bidder_b
    assert_eq!(
        mp_client.try_place_bid(&1, &bidder_a, &15),
        Err(Ok(ContractError::BidNotEnough))
    );
    assert_eq!(token_client.balance(&mp_client.address), 30i128);
    assert_eq!(token_client.balance(&bidder_a), 10i128);
    assert_eq!(token_client.balance(&bidder_b), 0i128);

    assert_eq!(
        mp_client.get_highest_bid(&1),
        HighestBid {
            bid: 20u64,
            bidder: bidder_b.clone()
        }
    );

    mp_client.place_bid(&1, &bidder_c, &40);
    assert_eq!(
        mp_client.get_highest_bid(&1),
        HighestBid {
            bid: 40u64,
            bidder: bidder_c.clone()
        }
    );
    assert_eq!(token_client.balance(&mp_client.address), 50i128);
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

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&seller, &10);

    let (mp_client, nft_collection_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );
    nft_collection_client.set_approval_for_all(&mp_client.address, &true);

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    env.ledger().with_mut(|li| li.timestamp = WEEKLY + DAY);

    mp_client.finalize_auction(&1);

    assert_eq!(
        mp_client.try_place_bid(&1, &bidder_a, &10),
        Err(Ok(ContractError::AuctionNotActive))
    );
}

#[test]
fn seller_tries_to_place_a_bid_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&seller, &11);
    let (mp_client, collection_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collection_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collection_client.address,
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

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

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: None,
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &DAY);

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

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address,
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info, &seller, &DAY);

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
    token_client.mint(&seller, &10);

    token_client.mint(&fomo_buyer, &100);
    token_client.mint(&bidder_a, &100);
    token_client.mint(&bidder_b, &100);

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    collections_client.set_approval_for_transfer(&mp_client.address, &1u64, &true);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    // 4 hours in and we have a first highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS);
    mp_client.place_bid(&1, &bidder_a, &5);
    assert_eq!(token_client.balance(&bidder_a), 95);
    assert_eq!(token_client.balance(&mp_client.address), 15);

    // 8 hours in and we have a second highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 2);
    mp_client.place_bid(&1, &bidder_b, &10);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 90);
    assert_eq!(token_client.balance(&mp_client.address), 20);

    // 16 hours in and we have a third highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 4);
    mp_client.place_bid(&1, &fomo_buyer, &25);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 100);
    assert_eq!(token_client.balance(&fomo_buyer), 75);
    assert_eq!(token_client.balance(&mp_client.address), 35);

    // 24 hours in and we have a 4th highest bid
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 6);
    mp_client.place_bid(&1, &bidder_b, &30);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 70);
    assert_eq!(token_client.balance(&fomo_buyer), 100);
    assert_eq!(token_client.balance(&mp_client.address), 40);

    // 36 hours in and we have a 5th highest bid, which is over the buy now price
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 9);
    mp_client.place_bid(&1, &bidder_a, &60);
    assert_eq!(token_client.balance(&bidder_a), 40);
    assert_eq!(token_client.balance(&bidder_b), 100);
    assert_eq!(token_client.balance(&fomo_buyer), 100);
    assert_eq!(token_client.balance(&mp_client.address), 70);

    // 40 hours in and the fomo buyer sees the previous user mistake and buys now
    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 10);
    mp_client.buy_now(&1, &fomo_buyer);
    assert_eq!(token_client.balance(&bidder_a), 100);
    assert_eq!(token_client.balance(&bidder_b), 100);
    assert_eq!(token_client.balance(&fomo_buyer), 50);
    // mp_client has the fees from the auction creation
    assert_eq!(token_client.balance(&mp_client.address), 10);
    assert_eq!(token_client.balance(&seller), 50);

    assert_eq!(
        mp_client.get_auction(&1),
        Auction {
            id: 1,
            item_info,
            seller,
            highest_bid: Some(50),
            end_time: WEEKLY,
            status: AuctionStatus::Ended,
            auction_token: token_client.address
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

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    env.ledger().with_mut(|li| li.timestamp = DAY);

    mp_client.pause(&1);

    assert_eq!(mp_client.get_auction(&1).status, AuctionStatus::Paused);

    assert_eq!(
        mp_client.try_pause(&1),
        Err(Ok(ContractError::AuctionNotActive))
    );

    // 4 weeks after the creation, after the auction has already expired the seller tries to
    //   pause it
    env.ledger().with_mut(|li| li.timestamp = WEEKLY * 4);

    assert_eq!(
        mp_client.try_pause(&1),
        Err(Ok(ContractError::AuctionNotActive))
    );
}

#[test]
fn pause_after_enddate_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    env.ledger().with_mut(|li| li.timestamp = WEEKLY + DAY);

    assert_eq!(
        mp_client.try_pause(&1),
        Err(Ok(ContractError::AuctionNotActive))
    );
}

#[test]
fn unpause_changes_status_and_second_attempt_fails_to_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);
    token_client.mint(&bidder, &100);

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: None,
        buy_now_price: Some(10),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    env.ledger().with_mut(|li| li.timestamp = DAY);

    assert_eq!(
        mp_client.try_unpause(&1),
        Err(Ok(ContractError::AuctionNotPaused))
    );

    mp_client.pause(&1);
    assert_eq!(mp_client.get_auction(&1).status, AuctionStatus::Paused);

    assert_eq!(
        mp_client.try_place_bid(&1, &bidder, &100),
        Err(Ok(ContractError::AuctionNotActive))
    );

    assert_eq!(
        mp_client.try_buy_now(&1, &bidder),
        Err(Ok(ContractError::AuctionNotActive))
    );

    assert_eq!(
        mp_client.try_finalize_auction(&1),
        Err(Ok(ContractError::AuctionNotActive))
    );

    mp_client.unpause(&1);
    assert_eq!(mp_client.get_auction(&1).status, AuctionStatus::Active);

    mp_client.place_bid(&1, &bidder, &100);

    assert_eq!(token_client.balance(&bidder), 0);
    assert_eq!(token_client.balance(&mp_client.address), 100);
    assert_eq!(
        mp_client.get_highest_bid(&1),
        HighestBid { bid: 100, bidder }
    );

    mp_client.pause(&1);
    // 4 weeks after the creation, after the auction has already expired the seller tries to
    //   unpause it
    env.ledger().with_mut(|li| li.timestamp = WEEKLY * 4);

    assert_eq!(
        mp_client.try_unpause(&1),
        Err(Ok(ContractError::AuctionNotActive))
    );
}

#[test]
fn multiple_auction_by_multiple_sellers() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let seller_a = Address::generate(&env);
    let seller_b = Address::generate(&env);
    let seller_c = Address::generate(&env);

    let bidder_a = Address::generate(&env);
    let bidder_b = Address::generate(&env);
    let bidder_c = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&bidder_a, &1_000);
    token_client.mint(&bidder_b, &1_000);
    token_client.mint(&bidder_c, &1_000);

    let mp_client =
        MarketplaceContractClient::new(&env, &env.register_contract(None, MarketplaceContract {}));

    mp_client.initialize(&admin, &token_client.address, &10);

    // ============ Collections client setup ============
    let collection_a_client =
        create_and_initialize_collection(&env, &seller_a, "Seller A Collection", "SAC");
    let collection_b_client =
        create_and_initialize_collection(&env, &seller_b, "Seller B Collection", "SBC");
    let collection_c_client =
        create_and_initialize_collection(&env, &seller_c, "Seller C Collection", "SCC");

    // ============ Auction item setup ============
    let first_item_info_seller_a = ItemInfo {
        collection_addr: collection_a_client.address.clone(),
        item_id: 1,
        minimum_price: Some(100),
        buy_now_price: Some(500),
    };

    collection_a_client.mint(&seller_a, &seller_a, &2, &1);

    mp_client.create_auction(&first_item_info_seller_a, &seller_a, &WEEKLY);

    let second_item_info_seller_a = ItemInfo {
        collection_addr: collection_a_client.address.clone(),
        item_id: 2,
        minimum_price: Some(500),
        buy_now_price: Some(900),
    };

    mp_client.create_auction(&second_item_info_seller_a, &seller_a, &WEEKLY);

    let item_info_seller_b = ItemInfo {
        collection_addr: collection_b_client.address.clone(),
        item_id: 1,
        minimum_price: Some(50),
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info_seller_b, &seller_b, &WEEKLY);

    let item_info_seller_c = ItemInfo {
        collection_addr: collection_c_client.address.clone(),
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
    };

    mp_client.create_auction(&item_info_seller_c, &seller_c, &DAY);
    // ============ Authorized transfer ============================
    collection_a_client.set_approval_for_transfer(&mp_client.address, &1, &true);
    collection_a_client.set_approval_for_transfer(&mp_client.address, &2, &true);
    collection_b_client.set_approval_for_transfer(&mp_client.address, &1, &true);
    collection_c_client.set_approval_for_transfer(&mp_client.address, &1, &true);

    // ============ Assert everything is before bidding ============

    assert_eq!(
        mp_client.get_auctions_by_seller(&seller_a),
        vec![
            &env,
            Auction {
                id: 1,
                item_info: first_item_info_seller_a.clone(),
                seller: seller_a.clone(),
                highest_bid: None,
                end_time: WEEKLY,
                status: AuctionStatus::Active,
                auction_token: token_client.address.clone()
            },
            Auction {
                id: 2,
                item_info: second_item_info_seller_a.clone(),
                seller: seller_a.clone(),
                highest_bid: None,
                end_time: WEEKLY,
                status: AuctionStatus::Active,
                auction_token: token_client.address.clone()
            }
        ]
    );

    assert_eq!(
        mp_client.get_auctions_by_seller(&seller_b),
        vec![
            &env,
            Auction {
                id: 3,
                item_info: item_info_seller_b.clone(),
                seller: seller_b.clone(),
                highest_bid: None,
                end_time: WEEKLY,
                status: AuctionStatus::Active,
                auction_token: token_client.address.clone()
            },
        ]
    );

    assert_eq!(
        mp_client.get_auctions_by_seller(&seller_c),
        vec![
            &env,
            Auction {
                id: 4,
                item_info: item_info_seller_c.clone(),
                seller: seller_c.clone(),
                highest_bid: None,
                end_time: DAY,
                status: AuctionStatus::Active,
                auction_token: token_client.address.clone()
            },
        ]
    );
    // ============ Start bidding ============

    // within day #1
    env.ledger().with_mut(|li| li.timestamp = DAY / 4);

    mp_client.place_bid(&1, &bidder_a, &50);
    mp_client.place_bid(&2, &bidder_a, &50);
    // `bidder_b` places a bid, but is immediately outbidden by `bidder_c`
    mp_client.place_bid(&3, &bidder_b, &25);
    mp_client.place_bid(&3, &bidder_c, &26);
    mp_client.place_bid(&4, &bidder_b, &100);

    assert_eq!(token_client.balance(&bidder_a), 900);
    assert_eq!(token_client.balance(&bidder_b), 900);
    assert_eq!(token_client.balance(&bidder_c), 974);
    assert_eq!(token_client.balance(&mp_client.address), 226);

    // within day #2
    // here auction #4 has ended
    env.ledger().with_mut(|li| li.timestamp = DAY * 2);

    assert_eq!(
        mp_client.try_place_bid(&4, &bidder_a, &200),
        Err(Ok(ContractError::AuctionNotActive))
    );

    mp_client.finalize_auction(&4);
    assert_eq!(token_client.balance(&mp_client.address), 126);
    assert_eq!(token_client.balance(&bidder_a), 900);
    assert_eq!(token_client.balance(&bidder_b), 900);
    assert_eq!(token_client.balance(&bidder_c), 974);
    assert_eq!(token_client.balance(&seller_c), 100);
    assert_eq!(collection_c_client.balance_of(&bidder_b, &1), 1);

    // day #3
    env.ledger().with_mut(|li| li.timestamp = DAY * 3);

    mp_client.place_bid(&1, &bidder_b, &100);
    mp_client.place_bid(&2, &bidder_c, &75);
    mp_client.place_bid(&3, &bidder_a, &50);

    // `bidder_a` has been outbid in both #1 and #2, so he gets his 100 in total back; then he
    // places a 50 bid on #3 leaving his balance with 950
    assert_eq!(token_client.balance(&bidder_a), 950);
    // `bidder_b` has won auctoin #4 with a 100 bid; now he placed another bid for a 100 in auction
    // #1
    assert_eq!(token_client.balance(&bidder_b), 800);
    // `bidder_c` had a bid of 26 for auction #3, but he has been outbid by `bidder_a` and after
    // `bidder_c`places a bit for 75 he now has 925 in total
    assert_eq!(token_client.balance(&bidder_c), 925);
    // total of the assets locked in the contract
    assert_eq!(token_client.balance(&mp_client.address), 225);

    // day #4
    env.ledger().with_mut(|li| li.timestamp = DAY * 4);
    // `bidder_b` fomos and buys the item in auction #1. Right after that `bidder_a`tries to bid on
    // that item but fails to do as the auction has ended.
    mp_client.buy_now(&1, &bidder_b);
    assert_eq!(
        mp_client.try_place_bid(&1, &bidder_a, &150),
        Err(Ok(ContractError::AuctionNotActive))
    );

    // verify the ownership
    assert_eq!(collection_a_client.balance_of(&bidder_b, &1), 1);

    // we have 2 auctions remaining: #2 and #3
    // the last highest bid on auction #3 is from `bidder_a` so when `bidder_c` places a bet the
    // previous bid of 50 is returned back to `bidder_a` making his total balance to 900
    mp_client.place_bid(&2, &bidder_a, &100);
    mp_client.place_bid(&3, &bidder_c, &100);

    // day #5
    env.ledger().with_mut(|li| li.timestamp = DAY * 5);

    // the bid of `bidder_b` for 150 returns the previous bid of `bidder_a`, thus `bidder_a` has a
    // total of 1000 again
    mp_client.place_bid(&2, &bidder_b, &150);
    mp_client.place_bid(&3, &bidder_a, &150);

    // day #6
    // let's count the balances again

    assert_eq!(token_client.balance(&bidder_a), 850);
    // `bidder_b` has the lowest balance, due to `buy_now`
    assert_eq!(token_client.balance(&bidder_b), 250);
    // `bidder_c` has been outbid by `bidder_a` the previous day, thus having his full portfolio of
    // 1_000 since he was outbid in the last day and got the last 100 refunded
    assert_eq!(token_client.balance(&bidder_c), 1_000);

    // day #15
    // okay, enough bidding, let's close the auctions
    env.ledger().with_mut(|li| li.timestamp = WEEKLY + DAY);

    mp_client.finalize_auction(&2);
    mp_client.finalize_auction(&3);

    // let's do some assertions

    // assertions of the state of the auctions
    assert_eq!(
        mp_client.get_auctions_by_seller(&seller_a),
        vec![
            &env,
            Auction {
                id: 1,
                item_info: first_item_info_seller_a.clone(),
                seller: seller_a.clone(),
                highest_bid: Some(500),
                end_time: WEEKLY,
                status: AuctionStatus::Ended,
                auction_token: token_client.address.clone()
            },
            Auction {
                id: 2,
                item_info: second_item_info_seller_a,
                seller: seller_a.clone(),
                highest_bid: Some(150),
                end_time: WEEKLY,
                status: AuctionStatus::Ended,
                auction_token: token_client.address.clone()
            }
        ]
    );

    assert_eq!(
        mp_client.get_auctions_by_seller(&seller_b),
        vec![
            &env,
            Auction {
                id: 3,
                item_info: item_info_seller_b,
                seller: seller_b.clone(),
                highest_bid: Some(150),
                end_time: WEEKLY,
                status: AuctionStatus::Ended,
                auction_token: token_client.address.clone()
            },
        ]
    );

    assert_eq!(
        mp_client.get_auctions_by_seller(&seller_c),
        vec![
            &env,
            Auction {
                id: 4,
                item_info: item_info_seller_c,
                seller: seller_c.clone(),
                highest_bid: Some(100),
                end_time: DAY,
                status: AuctionStatus::Ended,
                auction_token: token_client.address.clone()
            },
        ]
    );

    // assertions of the token balances
    // because `bidder_b` used `buy_now` for auction #1 and no one was able to put at least 500 as
    // bid to meet the minimum price
    assert_eq!(token_client.balance(&seller_a), 500);
    // `bidder_a` placed a bit of 150 for this auction
    assert_eq!(token_client.balance(&seller_b), 150);
    // `bidder_b` bought it with a bid of a 100
    assert_eq!(token_client.balance(&seller_c), 100);

    // bought the item from `seller_b`
    assert_eq!(token_client.balance(&bidder_a), 850);
    // bought with `buy_now` for a 500 tokens and another big and since he did not met the minimum
    // amount to buy the item and got a refund of 100
    assert_eq!(token_client.balance(&bidder_b), 400);
    // `bidder_c` has all the tokens he started with
    assert_eq!(token_client.balance(&bidder_c), 1_000);

    // make sure that we don't hold any tokens, as we are just intermediary
    assert_eq!(token_client.balance(&mp_client.address), 0);

    // let's check the item info
    // auction #1 sold item #1 from `collection_a` and the winner is `bidder_a`
    assert_eq!(collection_a_client.balance_of(&bidder_b, &1), 1);
    // auction #1 DID NOT SELL item #2 from collection_a; item remains with `seller_a`
    assert_eq!(collection_a_client.balance_of(&seller_a, &2), 1);
    // auction #3 sold item #1 from `collection_b` and the winner is `bidder_a`
    assert_eq!(collection_b_client.balance_of(&bidder_a, &1), 1);
    // auction #4 sold item #1 from `collection_c` and the winnder is `bidder_b`
    assert_eq!(collection_c_client.balance_of(&bidder_b, &1), 1);
}

#[test]
fn buy_now_should_fail_when_status_is_different_from_active() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&bidder, &10);

    let (mp_client, collection) =
        generate_marketplace_and_collection_client(&env, &seller, &token.address, None, None);

    collection.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collection.address,
        item_id: 1,
        minimum_price: None,
        buy_now_price: Some(10),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    env.ledger().with_mut(|li| li.timestamp = DAY);

    mp_client.pause(&1);

    assert_eq!(
        mp_client.try_buy_now(&1, &bidder),
        Err(Ok(ContractError::AuctionNotActive))
    );
    assert_eq!(token.balance(&bidder), 10);
}

#[test]
fn buy_now_should_work_when_no_previous_bid() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let fomo_buyer = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&fomo_buyer, &100);

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    collections_client.set_approval_for_transfer(&mp_client.address, &1u64, &true);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS);

    mp_client.buy_now(&1, &fomo_buyer);

    assert_eq!(token_client.balance(&fomo_buyer), 50);
    assert_eq!(token_client.balance(&mp_client.address), 0);
    assert_eq!(token_client.balance(&seller), 50);
}

#[test]
fn buy_now_should_refund_previous_buyer() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let fomo_buyer = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&fomo_buyer, &100);
    token_client.mint(&bidder, &100);

    let (mp_client, collections_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    collections_client.mint(&seller, &seller, &1, &1);

    collections_client.set_approval_for_transfer(&mp_client.address, &1u64, &true);

    let item_info = ItemInfo {
        collection_addr: collections_client.address.clone(),
        item_id: 1,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    mp_client.create_auction(&item_info, &seller, &WEEKLY);

    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS);

    mp_client.place_bid(&1, &bidder, &40);
    assert_eq!(token_client.balance(&bidder), 60);
    assert_eq!(token_client.balance(&mp_client.address), 40);

    env.ledger().with_mut(|li| li.timestamp = FOUR_HOURS * 2);

    mp_client.buy_now(&1, &fomo_buyer);

    assert_eq!(token_client.balance(&fomo_buyer), 50);
    assert_eq!(token_client.balance(&bidder), 100);
    assert_eq!(token_client.balance(&mp_client.address), 0);
    assert_eq!(token_client.balance(&seller), 50);
}
