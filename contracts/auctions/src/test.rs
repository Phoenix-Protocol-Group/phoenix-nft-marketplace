use soroban_sdk::{testutils::Address as _, token, Address, Env, String};

use crate::{
    collection,
    contract::{MarketplaceContract, MarketplaceContractClient},
    error::ContractError,
    storage::{Auction, AuctionStatus, ItemInfo},
};

const WEEKLY: u64 = 604_800u64;

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
fn generate_marketplace_and_collection_client<'a>(
    env: Env,
    seller: Address,
    name: Option<String>,
    symbol: Option<String>,
) -> (MarketplaceContractClient<'a>, collection::Client<'a>) {
    let mp_client =
        MarketplaceContractClient::new(&env, &env.register_contract(None, MarketplaceContract {}));

    let alt_name = String::from_str(&env, "Stellar kitties");
    let alt_symbol = String::from_str(&env, "STK");

    let name = name.unwrap_or(alt_name);
    let symbol = symbol.unwrap_or(alt_symbol);
    let collection_addr = env.register_contract_wasm(None, collection::WASM);

    let collection_client = collection::Client::new(&env, &collection_addr);
    collection_client.initialize(&seller, &name, &symbol);
    collection_client.mint(&seller, &seller, &1, &2);

    (mp_client, collection_client)
}
