use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

use crate::{
    contract::{Collections, CollectionsClient},
    storage::URIValue,
};

pub fn initialize_collection_contract<'a>(
    env: &Env,
    admin: Option<&Address>,
    name: Option<&String>,
    image: Option<&URIValue>,
) -> CollectionsClient<'a> {
    let collections = CollectionsClient::new(env, &env.register_contract(None, Collections {}));

    let alt_admin = &Address::generate(env);
    let alt_name = &String::from_str(env, "Stellar kitties");
    let alt_image = URIValue {
        uri: Bytes::from_slice(env, &[64]),
    };

    let admin = admin.unwrap_or(alt_admin);
    let name = name.unwrap_or(alt_name);
    let image = image.unwrap_or(&alt_image);

    collections.initialize(admin, name, image);

    collections
}
