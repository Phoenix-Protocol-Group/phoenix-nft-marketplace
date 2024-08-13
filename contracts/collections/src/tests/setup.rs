use soroban_sdk::{Address, Env, String};

use crate::{
    contract::{Collections, CollectionsClient},
    storage::URIValue,
};

pub fn initialize_collection_contract<'a>(
    env: &Env,
    admin: &Address,
    name: &String,
    image: &URIValue,
) -> CollectionsClient<'a> {
    let collections = CollectionsClient::new(env, &env.register_contract(None, Collections {}));

    collections.initialize(admin, name, image);

    collections
}
