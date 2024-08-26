use soroban_sdk::{Address, Env, String};

use crate::{
    collection,
    contract::{MarketplaceContract, MarketplaceContractClient},
};

pub const WEEKLY: u64 = 604_800u64;
pub const DAY: u64 = 86_400u64;

pub fn generate_marketplace_and_collection_client<'a>(
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
