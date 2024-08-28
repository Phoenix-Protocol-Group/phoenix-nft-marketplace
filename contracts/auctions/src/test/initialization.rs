extern crate std;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{
    collection::{self},
    error::ContractError,
    storage::{Auction, AuctionStatus, ItemInfo},
    test::setup::{create_multiple_auctions, generate_marketplace_and_collection_client, WEEKLY},
};

use super::setup::deploy_token_contract;

#[test]
fn mp_should_create_auction() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);
    let token_client = token::Client::new(&env, &Address::generate(&env));

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    // check if we have minted two
    assert_eq!(nft_collection_client.balance_of(&seller, &1), 2);
    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);

    assert_eq!(
        mp_client.get_auction(&1),
        Auction {
            id: 1,
            item_info,
            seller: seller.clone(),
            highest_bid: None,
            highest_bidder: seller,
            end_time: WEEKLY,
            status: AuctionStatus::Active,
            currency: token_client.address
        }
    );
}

#[test]
fn create_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);

    let (mp_client, nft_collection_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);
    let token_client = token::Client::new(&env, &Address::generate(&env));

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    // check if we have minted two
    assert_eq!(nft_collection_client.balance_of(&seller, &1), 2);
    mp_client.create_auction(&item_info, &seller, &WEEKLY, &token_client.address);
    assert_eq!(
        mp_client.try_create_auction(&item_info, &seller, &WEEKLY, &token_client.address),
        Err(Ok(ContractError::AlreadyInitialized))
    );
}

#[test]
fn mp_should_fail_to_create_auction_where_not_enought_balance_of_the_item() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let seller = Address::generate(&env);

    // we don't want to use the collection from the setup method, as this will automatically
    // mint an item for the auction.
    let (mp_client, _) = generate_marketplace_and_collection_client(&env, &seller, None, None);
    let token_client = token::Client::new(&env, &Address::generate(&env));

    let collection_addr = env.register_contract_wasm(None, collection::WASM);

    let collection_client = collection::Client::new(&env, &collection_addr);
    collection_client.initialize(
        &seller,
        &soroban_sdk::String::from_str(&env, "Soroban Kitties"),
        &soroban_sdk::String::from_str(&env, "SKT"),
    );

    let item_info = ItemInfo {
        collection_addr: collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
    };

    assert_eq!(
        mp_client.try_create_auction(&item_info, &seller, &WEEKLY, &token_client.address),
        Err(Ok(ContractError::NotEnoughBalance))
    );
}

#[test]
fn mp_should_be_able_create_multiple_auctions_and_query_them_with_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let seller = Address::generate(&env);
    let token_client = deploy_token_contract(&env, &Address::generate(&env));

    let (mp_client, collection_client) =
        generate_marketplace_and_collection_client(&env, &seller, None, None);

    create_multiple_auctions(
        &mp_client,
        &seller,
        &token_client.address,
        &collection_client,
        25,
    );

    //we have created 25 auctions and if we don't specify anything the default search would be
    //from 1..=10
    let result = mp_client.get_active_auctions(&None, &None);
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );

    // manual from 1..=10
    let result = mp_client.get_active_auctions(&Some(1), &Some(10));
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );

    // manaul from 10..=20
    let result = mp_client.get_active_auctions(&Some(10), &Some(20));
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        std::vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
    );

    // manaul from 1..=25
    let result = mp_client.get_active_auctions(&Some(1), &Some(25));
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        // I'm lazy kek
        (1..=25).collect::<std::vec::Vec<u64>>()
    );
}
