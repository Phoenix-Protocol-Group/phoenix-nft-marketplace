use soroban_sdk::{Address, Env, String};

use crate::{
    collection::{self, Client},
    contract::{MarketplaceContract, MarketplaceContractClient},
    storage::ItemInfo,
    token,
};

pub const WEEKLY: u64 = 604_800u64;
pub const DAY: u64 = 86_400u64;

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
    seller: &Address,
    name: Option<String>,
    symbol: Option<String>,
) -> (MarketplaceContractClient<'a>, collection::Client<'a>) {
    let mp_client =
        MarketplaceContractClient::new(env, &env.register_contract(None, MarketplaceContract {}));

    let alt_name = String::from_str(env, "Stellar kitties");
    let alt_symbol = String::from_str(env, "STK");

    let name = name.unwrap_or(alt_name);
    let symbol = symbol.unwrap_or(alt_symbol);
    let collection_addr = env.register_contract_wasm(None, collection::WASM);

    let collection_client = collection::Client::new(env, &collection_addr);
    collection_client.initialize(seller, &name, &symbol);
    collection_client.mint(seller, seller, &1, &2);

    (mp_client, collection_client)
}

pub fn create_multiple_auctions(
    mp_client: &crate::contract::MarketplaceContractClient,
    seller: &Address,
    currency: &Address,
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
        };
        mp_client.create_auction(&item_info, seller, &WEEKLY, currency);
    }
}
