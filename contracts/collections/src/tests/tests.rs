use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

use crate::storage::URIValue;

use super::setup::initialize_collection_contract;

#[test]
fn test_should_initialize() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let uri_value = URIValue {
        uri: Bytes::from_slice(&env, &[64]),
    };

    let collections_client = initialize_collection_contract(
        &env,
        &admin,
        &String::from_str(&env, "Stellar kitties"),
        &uri_value,
    );

    let actual_admin_addr = collections_client.get_admin();

    assert_eq!(admin, actual_admin_addr);
}
