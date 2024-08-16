use crate::{CollectionsDeployer, CollectionsDeployerClient};
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
    let client =
        CollectionsDeployerClient::new(&env, &env.register_contract(None, CollectionsDeployer));

    // Upload the Wasm to be deployed from the deployer contract.
    // This can also be called from within a contract if needed.
    let wasm_hash = env.deployer().upload_contract_wasm(collections::WASM);

    client.initialize(&wasm_hash);

    env.mock_all_auths();

    let salt = BytesN::from_array(&env, &[0; 32]);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Stellar Kitties");

    client.deploy_new_collection(&salt, &creator, &name);

    assert_eq!(client.query_all_collections(), vec![&env, name.clone()]);
    assert_eq!(
        client.query_collection_by_creator(&creator),
        vec![&env, name]
    );
}

#[test]
#[ignore = "once we rebase from main"]
#[should_panic(
    expected = "Collections Deployer: Initialize: initializing the contract twice is not allowed"
)]
fn initialize_twice() {
    let env = Env::default();
    let deployer_client =
        CollectionsDeployerClient::new(&env, &env.register_contract(None, CollectionsDeployer));

    let wasm_hash = env.deployer().upload_contract_wasm(collections::WASM);
    deployer_client.initialize(&wasm_hash);
    deployer_client.initialize(&wasm_hash);
}
