#![no_std]

use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, log, vec, Address, BytesN, Env, IntoVal,
    String, Symbol, Val, Vec,
};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Multisig Deployer Contract"
);

#[contract]
pub struct CollectionsDeployer;

#[contractimpl]
impl CollectionsDeployer {
    #[allow(dead_code)]
    pub fn initialize(env: Env, collections_wasm_hash: BytesN<32>) {
        if is_initialized(&env) {
            log!(
                &env,
                "Multisig Deployer: Initialize: initializing the contract twice is not allowed"
            );
            panic!("Multisig Deployer: Initialize: initializing the contract twice is not allowed");
        }
        set_initialized(&env);

        set_wasm_hash(&env, &collections_wasm_hash);
    }

    #[allow(dead_code)]
    pub fn deploy_new_collection(
        env: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
    ) -> Address {
        admin.require_auth();
        let collections_wasm_hash = get_wasm_hash(&env);

        let deployed_multisig = env
            .deployer()
            .with_address(admin.clone(), salt)
            .deploy(collections_wasm_hash);

        let init_fn = Symbol::new(&env, "initialize");
        let init_fn_args: Vec<Val> = vec![&env, admin.into_val(&env), name.into_val(&env)];
        let _: Val = env.invoke_contract(&deployed_multisig, &init_fn, init_fn_args);

        save_collection_with_generic_key(&env, name.clone());
        save_collection_with_admin_key(&env, name, admin);

        deployed_multisig
    }
}

// ---------- Storage types ----------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    IsInitialized,
    CollectionsWasmHash,
    AllCollections,
    AdminId(Address),
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&DataKey::IsInitialized, &());
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .instance()
        .get::<_, ()>(&DataKey::IsInitialized)
        .is_some()
}

pub fn set_wasm_hash(env: &Env, hash: &BytesN<32>) {
    env.storage()
        .instance()
        .set(&DataKey::CollectionsWasmHash, hash);
}

pub fn get_wasm_hash(env: &Env) -> BytesN<32> {
    env.storage()
        .instance()
        .get(&DataKey::CollectionsWasmHash)
        .unwrap()
}

pub fn save_collection_with_generic_key(env: &Env, name: String) {
    let mut existent_collection: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::AllCollections)
        .unwrap_or(vec![&env]);

    existent_collection.push_back(name);

    env.storage()
        .persistent()
        .set(&DataKey::AllCollections, &existent_collection);
}

pub fn save_collection_with_admin_key(env: &Env, name: String, admin: Address) {
    let mut existent_collection: Vec<String> = env
        .storage()
        .persistent()
        .get(&DataKey::AdminId(admin.clone()))
        .unwrap_or(vec![&env]);

    existent_collection.push_back(name);

    env.storage()
        .persistent()
        .set(&DataKey::AdminId(admin), &existent_collection);
}

