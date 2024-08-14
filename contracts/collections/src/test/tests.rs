use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, Env, String};

use crate::storage::{Config, URIValue};

use super::setup::initialize_collection_contract;

#[test]
fn proper_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let name = &String::from_str(&env, "Stellar kitties");
    let symbol = &String::from_str(&env, "STK");

    let collections_client =
        initialize_collection_contract(&env, Some(&admin), Some(name), Some(symbol));

    let actual_admin_addr = collections_client.show_admin();
    assert_eq!(admin, actual_admin_addr);

    let actual_config = collections_client.show_config();
    let expected_config = Config {
        name: name.clone(),
        symbol: symbol.clone(),
    };

    assert_eq!(actual_config.name, expected_config.name);
    assert_eq!(actual_config.symbol, expected_config.symbol);
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
    assert_eq!(collections_client.balance_of(&user, &2), 10);
}

#[test]
fn mint_batch_and_balance_of_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let user_c = Address::generate(&env);
    let user_d = Address::generate(&env);
    let user_e = Address::generate(&env);

    let id_list = vec![&env, 1, 2, 3, 4, 5];
    let amounts_list = vec![&env, 10, 20, 30, 40, 50];

    let collections_client = initialize_collection_contract(&env, Some(&admin), None, None);

    collections_client.mint_batch(&admin, &user_a, &id_list, &amounts_list);
    collections_client.mint_batch(&admin, &user_b, &id_list, &amounts_list);
    collections_client.mint_batch(&admin, &user_c, &id_list, &amounts_list);
    collections_client.mint_batch(&admin, &user_d, &id_list, &amounts_list);
    collections_client.mint_batch(&admin, &user_e, &id_list, &amounts_list);

    let actual = collections_client.balance_of_batch(
        &vec![&env, user_a, user_b, user_c, user_d, user_e],
        &id_list,
    );

    // here we compare what amount of each nft_id does each user has
    assert_eq!(vec![&env, 10, 20, 30, 40, 50], actual);
}

#[test]
fn approval_tests() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let operator = Address::generate(&env);

    let collectoins_client = initialize_collection_contract(&env, None, None, None);

    collectoins_client.set_approval_for_all(&user, &operator, &true);

    assert!(collectoins_client.is_approved_for_all(&user, &operator));
}

#[test]
fn burning() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let collectoins_client = initialize_collection_contract(&env, Some(&admin), None, None);

    collectoins_client.mint(&admin, &user, &1, &2);
    assert_eq!(collectoins_client.balance_of(&user, &1), 2);

    collectoins_client.burn(&user, &1, &1);
    assert_eq!(collectoins_client.balance_of(&user, &1), 1);
}

#[test]
fn batch_burning() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let collections_client = initialize_collection_contract(&env, Some(&admin), None, None);

    collections_client.mint_batch(
        &admin,
        &user,
        &vec![&env, 1, 2, 3, 4, 5],
        &vec![&env, 10, 20, 30, 40, 50],
    );

    assert_eq!(
        collections_client.balance_of_batch(
            &vec![
                &env,
                user.clone(),
                user.clone(),
                user.clone(),
                user.clone(),
                user.clone()
            ],
            &vec![&env, 1, 2, 3, 4, 5]
        ),
        vec![&env, 10, 20, 30, 40, 50]
    );

    collections_client.burn_batch(
        &user,
        &vec![&env, 1, 2, 3, 4, 5],
        &vec![&env, 5, 10, 15, 20, 25],
    );

    assert_eq!(
        collections_client.balance_of_batch(
            &vec![
                &env,
                user.clone(),
                user.clone(),
                user.clone(),
                user.clone(),
                user.clone()
            ],
            &vec![&env, 1, 2, 3, 4, 5]
        ),
        vec![&env, 5, 10, 15, 20, 25],
    );
}

#[test]
fn test_uri() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let collections_client = initialize_collection_contract(&env, Some(&admin), None, None);

    collections_client.mint(&admin, &user, &1, &5);

    let secret_uri = Bytes::from_slice(
        &env,
        &[
            110, 101, 118, 101, 114, 32, 103, 111, 110, 110, 97, 32, 103, 105, 118, 101, 32, 121,
            111, 117, 32, 117, 112, 44, 32, 110, 101, 118, 101, 114, 32, 103, 111, 110, 110, 97,
            32, 108, 101, 116, 32, 121, 111, 117, 32, 100, 111, 119, 110,
        ],
    );
    collections_client.set_uri(&admin, &1, &secret_uri);

    assert_eq!(collections_client.uri(&1), URIValue { uri: secret_uri });
}
