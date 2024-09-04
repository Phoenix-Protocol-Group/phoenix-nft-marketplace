use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, Env, String};

use crate::{
    contract::{Collections, CollectionsClient},
    error::ContractError,
    storage::{Config, URIValue},
};

use super::setup::initialize_collection_contract;
use test_case::test_case;

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
fn initialization_should_fail_when_done_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let name = &String::from_str(&env, "Stellar kitties");
    let symbol = &String::from_str(&env, "STK");

    let collections = CollectionsClient::new(&env, &env.register_contract(None, Collections {}));

    collections.initialize(&admin, name, symbol);

    assert_eq!(
        collections.try_initialize(&admin, name, symbol),
        Err(Ok(ContractError::AlreadyInitialized))
    );
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
fn safe_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&admin), None, None);

    client.mint(&admin, &user_a, &1, &1);

    assert_eq!(client.balance_of(&user_a, &1), 1u64);
    assert_eq!(client.balance_of(&user_b, &1), 0u64);

    client.safe_transfer_from(&user_a, &user_a, &user_b, &1, &1);

    assert_eq!(client.balance_of(&user_a, &1), 0u64);
    assert_eq!(client.balance_of(&user_b, &1), 1u64);
}

#[test]
fn safe_batch_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let user_c = Address::generate(&env);
    let user_d = Address::generate(&env);
    let user_e = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&admin), None, None);

    let ids = vec![&env, 1, 2, 3, 4, 5];
    let amounts = vec![&env, 5, 5, 5, 5, 5];
    client.mint_batch(&admin, &user_a, &ids, &amounts);

    let accounts = vec![&env, user_a.clone(), user_b.clone(), user_c, user_d, user_e];
    assert_eq!(
        client.balance_of_batch(&accounts, &ids),
        vec![&env, 5, 0, 0, 0, 0]
    );

    client.safe_batch_transfer_from(&user_a, &user_b, &ids, &amounts);
    assert_eq!(
        client.balance_of_batch(&accounts, &ids),
        vec![&env, 0, 5, 0, 0, 0]
    );
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

#[test]
fn set_collection_uri_should_work() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let client = initialize_collection_contract(&env, Some(&user), None, None);

    assert_eq!(
        client.try_collection_uri(),
        Err(Ok(ContractError::NoUriSet))
    );
    let uri = Bytes::from_slice(&env, &[42]);
    client.set_collection_uri(&uri);

    assert_eq!(client.collection_uri(), URIValue { uri });
}

#[test]
fn should_fail_when_balance_of_batch_has_different_sizes_for_accounts_and_ids() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let collections_client = initialize_collection_contract(&env, Some(&admin), None, None);

    // not neceserally to mint anything, as we expect the test to fali

    assert_eq!(
        collections_client.try_balance_of_batch(&vec![&env, user], &vec![&env, 1, 2, 3]),
        Err(Ok(ContractError::AccountsIdsLengthMissmatch))
    )
}

#[test]
fn should_fail_when_set_approval_for_all_tries_to_approve_self() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let collections_client = initialize_collection_contract(&env, Some(&admin), None, None);

    assert_eq!(
        collections_client.try_set_approval_for_all(&admin, &admin, &true),
        Err(Ok(ContractError::CannotApproveSelf))
    )
}

#[test]
fn should_fail_when_sender_balance_not_enough() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&admin), None, None);

    // mint 1
    client.mint(&admin, &user_a, &1, &1);
    client.mint(&admin, &user_b, &1, &1);

    assert_eq!(client.balance_of(&user_a, &1), 1u64);

    // try to send 10
    assert_eq!(
        client.try_safe_transfer_from(&user_a, &user_a, &user_b, &1, &10),
        Err(Ok(ContractError::InsufficientBalance))
    )
}

#[test]
fn safe_batch_transfer_should_fail_when_id_mismatch() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&admin), None, None);

    let ids = vec![&env, 1, 2, 3, 4, 5];
    let amounts = vec![&env, 5, 5, 5, 5, 5];
    client.mint_batch(&admin, &user_a, &ids, &amounts);

    assert_eq!(
        client.try_safe_batch_transfer_from(
            &user_a,
            &Address::generate(&env),
            &ids,
            // only 4 amounts, when 5 are needed
            &vec![&env, 10, 10, 10, 10],
        ),
        Err(Ok(ContractError::IdsAmountsLengthMismatch))
    );
}

#[test_case(10, 10, 10, 10, 10; "very greedy")]
#[test_case(5, 4, 3, 2, 10; "just a single case is greedy")]
#[test_case(1, 1, 10, 1, 1; "same as the previous")]
fn safe_batch_transfer_should_fail_when_insufficient_balance(
    amount_a: u64,
    amount_b: u64,
    amount_c: u64,
    amount_d: u64,
    amount_e: u64,
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&admin), None, None);

    let ids = vec![&env, 1, 2, 3, 4, 5];
    let amounts = vec![&env, 5, 5, 5, 5, 5];
    client.mint_batch(&admin, &user_a, &ids, &amounts);

    assert_eq!(
        client.try_safe_batch_transfer_from(
            &user_a,
            &Address::generate(&env),
            &ids,
            &vec![&env, amount_a, amount_b, amount_c, amount_d, amount_e],
        ),
        Err(Ok(ContractError::InsufficientBalance))
    );
}

#[test]
fn mint_should_fail_when_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let client = initialize_collection_contract(&env, None, None, None);

    assert_eq!(
        client.try_mint(&Address::generate(&env), &Address::generate(&env), &1, &1),
        Err(Ok(ContractError::Unauthorized))
    );
}

#[test]
fn mint_batch_should_fail_when_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let client = initialize_collection_contract(&env, None, None, None);

    assert_eq!(
        client.try_mint_batch(
            &Address::generate(&env),
            &Address::generate(&env),
            &vec![&env, 1],
            &vec![&env, 1]
        ),
        Err(Ok(ContractError::Unauthorized))
    );
}

#[test]
fn mint_batch_should_fail_when_different_lengths_of_vecs() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&admin), None, None);

    assert_eq!(
        client.try_mint_batch(
            &admin,
            &Address::generate(&env),
            &vec![&env, 1, 2],
            &vec![&env, 1]
        ),
        Err(Ok(ContractError::IdsAmountsLengthMismatch))
    );
}

#[test]
fn burn_should_fail_when_not_enough_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let user = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&user), None, None);

    assert_eq!(
        client.try_burn(&user, &1, &1),
        Err(Ok(ContractError::InsufficientBalance))
    );
}

#[test]
fn burn_batch_should_fail_when_vec_length_missmatch() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let client = initialize_collection_contract(&env, Some(&user), None, None);

    assert_eq!(
        client.try_burn_batch(&user, &vec![&env, 1, 2], &vec![&env, 1]),
        Err(Ok(ContractError::IdsAmountsLengthMismatch))
    );
}

#[test]
fn burn_batch_should_fail_when_not_enough_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let client = initialize_collection_contract(&env, Some(&user), None, None);

    assert_eq!(
        client.try_burn_batch(&user, &vec![&env, 1], &vec![&env, 1]),
        Err(Ok(ContractError::InsufficientBalance))
    );
}

#[test]
fn set_uri_should_fail_when_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let client = initialize_collection_contract(&env, Some(&user), None, None);

    assert_eq!(
        client.try_set_uri(
            &Address::generate(&env),
            &1,
            &Bytes::from_slice(&env, &[42])
        ),
        Err(Ok(ContractError::Unauthorized))
    )
}

#[test]
fn uri_should_fail_when_none_set() {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let client = initialize_collection_contract(&env, Some(&user), None, None);

    assert_eq!(client.try_uri(&1), Err(Ok(ContractError::NoUriSet)))
}

#[test]
fn should_transfer_when_sender_is_operator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    let client = initialize_collection_contract(&env, Some(&admin), None, None);

    client.mint(&admin, &user_a, &1, &1);
    client.set_approval_for_all(&admin, &operator, &true);

    assert_eq!(client.balance_of(&user_a, &1), 1u64);
    assert_eq!(client.balance_of(&user_b, &1), 0u64);

    client.safe_transfer_from(&operator, &user_a, &user_b, &1, &1);

    assert_eq!(client.balance_of(&user_a, &1), 0u64);
    assert_eq!(client.balance_of(&user_b, &1), 1u64);
}
