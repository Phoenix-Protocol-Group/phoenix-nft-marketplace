use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, Env, String};

use crate::storage::{Config, URIValue};

use super::setup::initialize_collection_contract;

#[test]
fn proper_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let uri_value = URIValue {
        uri: Bytes::from_slice(&env, &[64]),
    };

    let name = &String::from_str(&env, "Stellar kitties");

    let collections_client =
        initialize_collection_contract(&env, Some(&admin), Some(&name), Some(&uri_value));

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

#[test]
fn mint_and_check_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let collections_client = initialize_collection_contract(&env, Some(&admin), None, None);

    collections_client.mint(&admin, &user, &1, &10);

    collections_client.mint(&admin, &user, &2, &10);

    assert_eq!(collections_client.balance_of(&user, &1), 10);
}

#[test]
fn mint_batch_and_balance_of_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let id_list = vec![&env, 1, 2, 3, 4, 5];
    let amounts_list = vec![&env, 10, 20, 30, 40, 50];

    let collections_client = initialize_collection_contract(&env, Some(&admin), None, None);

    collections_client.mint_batch(&admin, &user, &id_list, &amounts_list);

    let actual = collections_client.balance_of_batch(
        &vec![
            &env,
            user,
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ],
        &id_list,
    );

    assert_eq!(vec![&env, 10, 0, 0, 0, 0], actual);
}
