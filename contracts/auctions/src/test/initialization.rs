use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{
    collection,
    error::ContractError,
    storage::{Auction, AuctionStatus, ItemInfo},
    test::setup::{generate_marketplace_and_collection_client, WEEKLY},
};

#[test]
fn mp_should_create_auction() {
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
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
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
    let (mp_client, _) =
        generate_marketplace_and_collection_client(env.clone(), seller.clone(), None, None);
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
