use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::contract::{Collections, CollectionsClient};

pub fn initialize_collection_contract<'a>(
    env: &Env,
    admin: Option<&Address>,
    name: Option<&String>,
    symbol: Option<&String>,
) -> CollectionsClient<'a> {
    let collections = CollectionsClient::new(env, &env.register_contract(None, Collections {}));

    let alt_admin = &Address::generate(env);
    let alt_name = &String::from_str(env, "Stellar kitties");
    let alt_symbol = &String::from_str(env, "STK");

    let admin = admin.unwrap_or(alt_admin);
    let name = name.unwrap_or(alt_name);
    let image = symbol.unwrap_or(alt_symbol);

    collections.initialize(admin, name, image);

    collections
}
