extern crate std;
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    collection,
    contract::{MarketplaceContract, MarketplaceContractClient},
    error::ContractError,
    storage::{Auction, AuctionStatus, ItemInfo},
    test::setup::{create_multiple_auctions, generate_marketplace_and_collection_client, WEEKLY},
};

use super::setup::deploy_token_contract;

#[test]
fn initialize_and_update_admin_should_work() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);
    let mp_client = MarketplaceContractClient::new(&env, &env.register(MarketplaceContract, ()));

    mp_client.initialize(&admin, &token_client.address, &10, &1);
    mp_client.update_admin(&new_admin);
}

#[test]
fn mp_should_create_auction() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&seller, &10);

    let (mp_client, nft_collection_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
        amount: 1,
    };

    // check if we have minted two
    assert_eq!(nft_collection_client.balance_of(&seller, &1), 2);
    mp_client.create_auction(&item_info, &seller, &WEEKLY);
    // After escrow, seller has 1 left
    assert_eq!(nft_collection_client.balance_of(&seller, &1), 1);

    assert_eq!(
        mp_client.get_auction(&1),
        Auction {
            id: 1,
            item_info,
            seller: seller.clone(),
            highest_bid: None,
            end_time: WEEKLY,
            status: AuctionStatus::Active,
            auction_token: token_client.address
        }
    );
}

#[test]
fn initialize_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);
    token_client.mint(&seller, &10);

    let (mp_client, _) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    assert_eq!(
        mp_client.try_initialize(&admin, &token_client.address, &10, &1),
        Err(Ok(ContractError::AlreadyInitialized))
    );
}

#[test]
fn mp_should_fail_to_create_auction_where_not_enought_balance_of_the_item() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&seller, &10);

    // we don't want to use the collection from the setup method, as this will automatically
    // mint an item for the auction.
    let (mp_client, _) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    let collection_addr = env.register(collection::WASM, ());

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
        amount: 1,
    };

    assert_eq!(
        mp_client.try_create_auction(&item_info, &seller, &WEEKLY),
        Err(Ok(ContractError::NotEnoughBalance))
    );
}

#[test]
fn mp_should_be_able_create_multiple_auctions_and_query_them_with_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let seller = Address::generate(&env);
    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    token_client.mint(&seller, &250);

    let (mp_client, collection_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    create_multiple_auctions(&mp_client, &seller, &collection_client, 25);

    // We have created 25 auctions. Default search returns first 10 (start=1, limit=10)
    let result = mp_client.get_active_auctions(&None, &None);
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );

    // Explicit: start=1, limit=10 -> IDs 1..=10
    let result = mp_client.get_active_auctions(&Some(1), &Some(10));
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );

    // start=10, limit=11 -> IDs 10..=20
    let result = mp_client.get_active_auctions(&Some(10), &Some(11));
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        std::vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
    );

    // start=1, limit=25 -> IDs 1..=25
    let result = mp_client.get_active_auctions(&Some(1), &Some(25));
    assert_eq!(
        result
            .into_iter()
            .map(|a| a.id)
            .collect::<std::vec::Vec<u64>>(),
        (1..=25).collect::<std::vec::Vec<u64>>()
    );
}

#[test]
fn get_auction_by_id_should_return_an_err_when_id_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    let (mp_client, _) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    assert_eq!(
        mp_client.try_get_auction(&5),
        Err(Ok(ContractError::AuctionNotFound))
    )
}

#[test]
fn get_auction_by_seller_should_return_an_err_when_id_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    let (mp_client, _) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    assert_eq!(
        mp_client.try_get_auctions_by_seller(&Address::generate(&env)),
        Err(Ok(ContractError::AuctionNotFound))
    )
}

#[test]
fn should_fail_to_create_auction_when_seller_cannot_cover_the_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let seller = Address::generate(&env);

    let token = deploy_token_contract(&env, &Address::generate(&env));

    let (mp_client, collection) =
        generate_marketplace_and_collection_client(&env, &seller, &token.address, None, None);

    collection.mint(&seller, &seller, &1, &1);

    let item_info = ItemInfo {
        collection_addr: collection.address,
        item_id: 1,
        minimum_price: None,
        buy_now_price: None,
        amount: 1,
    };

    assert_eq!(
        mp_client.try_create_auction(&item_info, &seller, &WEEKLY),
        Err(Ok(ContractError::AuctionCreationFeeNotCovered))
    );
}

#[test]
fn mp_should_not_create_auction_with_item_info_with_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let seller = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &Address::generate(&env));
    let (mp_client, nft_collection_client) = generate_marketplace_and_collection_client(
        &env,
        &seller,
        &token_client.address,
        None,
        None,
    );

    let item_info = ItemInfo {
        collection_addr: nft_collection_client.address.clone(),
        item_id: 1u64,
        minimum_price: Some(10),
        buy_now_price: Some(50),
        amount: 0,
    };

    assert_eq!(
        mp_client.try_create_auction(&item_info, &seller, &WEEKLY),
        Err(Ok(ContractError::InvalidInputs))
    );
}
