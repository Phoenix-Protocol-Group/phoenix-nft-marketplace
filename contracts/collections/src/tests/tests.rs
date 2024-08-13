use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

use crate::storage::{Config, URIValue};

use super::setup::initialize_collection_contract;

#[test]
fn proper_initialization() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let uri_value = URIValue {
        uri: Bytes::from_slice(&env, &[64]),
    };

    let name = &String::from_str(&env, "Stellar kitties");

    let collections_client = initialize_collection_contract(&env, &admin, name, &uri_value);

    let actual_admin_addr = collections_client.show_admin();
    assert_eq!(admin, actual_admin_addr);

    let actual_config = collections_client.show_config();
    let expected_config = Config {
        name: name.clone(),
        image: uri_value,
    };

    assert_eq!(actual_config.name, expected_config.name);
    assert_eq!(actual_config.image, expected_config.image);
}
