use crate::{CollectionByCreatorResponse, CollectionsDeployer, CollectionsDeployerClient};
#[cfg(test)]
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env, String};

pub type NftId = u64;

// The contract that will be deployed by the deployer contract.
mod collections {
    use crate::tests::NftId;

    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_nft_collections.wasm"
    );
}

#[test]
fn test_deploy_collection_from_contract() {
    let env = Env::default();
    let client = CollectionsDeployerClient::new(&env, &env.register(CollectionsDeployer, ()));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(collections::WASM);

    client.initialize(&wasm_hash);

    env.mock_all_auths();

    let salt = BytesN::from_array(&env, &[0; 32]);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Stellar Kitties");
    let symbol = String::from_str(&env, "STK");

    let collection = client.deploy_new_collection(&salt, &creator, &name, &symbol);

    assert_eq!(client.query_all_collections(), vec![&env, name.clone()]);
    assert_eq!(
        client.query_collection_by_creator(&creator),
        vec![&env, CollectionByCreatorResponse { collection, name }]
    );
}

#[test]
fn test_deploy_multiple_collections() {
    let env = Env::default();
    env.mock_all_auths();

    let client = CollectionsDeployerClient::new(&env, &env.register(CollectionsDeployer, ()));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(collections::WASM);

    client.initialize(&wasm_hash);

    let creator = Address::generate(&env);
    let bob = Address::generate(&env);

    let first_salt = BytesN::from_array(&env, &[0; 32]);
    let first_collection_name = String::from_str(&env, "Stellar Kitties");
    let first_collection_symbol = String::from_str(&env, "STK");

    let second_salt = BytesN::from_array(&env, &[5; 32]);
    let second_collection_name = String::from_str(&env, "Stellar Puppers");
    let second_collection_symbol = String::from_str(&env, "STP");

    let third_salt = BytesN::from_array(&env, &[10; 32]);
    let third_collection_name = String::from_str(&env, "Horror of Cthulhu");
    let third_collection_symbol = String::from_str(&env, "HoC");

    let first = client.deploy_new_collection(
        &first_salt,
        &creator,
        &first_collection_name,
        &first_collection_symbol,
    );
    let second = client.deploy_new_collection(
        &second_salt,
        &creator,
        &second_collection_name,
        &second_collection_symbol,
    );
    let third = client.deploy_new_collection(
        &third_salt,
        &bob,
        &third_collection_name,
        &third_collection_symbol,
    );

    assert_eq!(
        client.query_all_collections(),
        vec![
            &env,
            first_collection_name.clone(),
            second_collection_name.clone(),
            third_collection_name.clone()
        ]
    );

    assert_eq!(
        client.query_collection_by_creator(&creator),
        vec![
            &env,
            CollectionByCreatorResponse {
                collection: first,
                name: first_collection_name,
            },
            CollectionByCreatorResponse {
                collection: second,
                name: second_collection_name,
            },
        ]
    );

    assert_eq!(
        client.query_collection_by_creator(&bob),
        vec![
            &env,
            CollectionByCreatorResponse {
                collection: third,
                name: third_collection_name
            }
        ]
    );
}

#[test]
#[should_panic(
    expected = "Collections Deployer: Initialize: initializing the contract twice is not allowed"
)]
fn initialize_twice() {
    let env = Env::default();
    let deployer_client =
        CollectionsDeployerClient::new(&env, &env.register(CollectionsDeployer, ()));

    let wasm_hash = env.deployer().upload_contract_wasm(collections::WASM);
    deployer_client.initialize(&wasm_hash);
    deployer_client.initialize(&wasm_hash);
}
