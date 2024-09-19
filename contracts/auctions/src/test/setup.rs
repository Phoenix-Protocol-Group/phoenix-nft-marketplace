use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, Bytes, Env, String};

use crate::{
    collection::{self, Client},
    contract::{MarketplaceContract, MarketplaceContractClient},
    storage::{AuctionType, ItemInfo},
    token,
};

pub const WEEKLY: u64 = 604_800u64;
pub const DAY: u64 = 86_400u64;
pub const FOUR_HOURS: u64 = 14_400u64;

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token::Client<'a> {
    token::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

pub mod auctions_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_nft_auctions.wasm"
    );
}

pub fn generate_marketplace_and_collection_client<'a>(
    env: &Env,
    admin: &Address,
    auction_token: &Address,
    name: Option<String>,
    symbol: Option<String>,
) -> (MarketplaceContractClient<'a>, collection::Client<'a>) {
    let mp_client =
        MarketplaceContractClient::new(env, &env.register_contract(None, MarketplaceContract {}));

    mp_client.initialize(admin, auction_token);

    let alt_name = String::from_str(env, "Stellar kitties");
    let alt_symbol = String::from_str(env, "STK");

    let name = name.unwrap_or(alt_name);
    let symbol = symbol.unwrap_or(alt_symbol);
    let collection_addr = env.register_contract_wasm(None, collection::WASM);

    let collection_client = collection::Client::new(env, &collection_addr);
    collection_client.initialize(admin, &name, &symbol);
    collection_client.mint(admin, admin, &1, &2);

    (mp_client, collection_client)
}

pub fn create_multiple_auctions(
    mp_client: &crate::contract::MarketplaceContractClient,
    seller: &Address,
    collection_client: &Client,
    number_of_auctions_to_make: usize,
) {
    for idx in 1..=number_of_auctions_to_make {
        collection_client.mint(seller, seller, &(idx as u64), &2);

        let item_info = ItemInfo {
            collection_addr: collection_client.address.clone(),
            item_id: idx as u64,
            minimum_price: None,
            buy_now_price: None,
            penny_price_increment: None,
            time_extension: None,
        };
        mp_client.create_auction(&item_info, seller, &WEEKLY, &AuctionType::English);
    }
}

/// This also mints 5 items of id #1 to the owner of the collection
pub fn create_and_initialize_collection<'a>(
    env: &Env,
    seller: &Address,
    collection_name: &str,
    collection_symbol: &str,
) -> collection::Client<'a> {
    let collection_name = String::from_str(env, collection_name);
    let collection_symbol = String::from_str(env, collection_symbol);

    let mut salt = Bytes::new(env);
    salt.append(&seller.clone().to_xdr(env));
    let salt = env.crypto().sha256(&salt);

    let collection_addr = env
        .deployer()
        .with_address(Address::generate(env), salt)
        .deploy(env.deployer().upload_contract_wasm(collection::WASM));

    let collection_client = collection::Client::new(env, &collection_addr);
    collection_client.initialize(seller, &collection_name, &collection_symbol);

    collection_client.mint(seller, seller, &1, &5);

    collection_client
}
