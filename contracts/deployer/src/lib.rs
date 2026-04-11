#![no_std]

use helpers::ttl::{
    INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL, PERSISTENT_RENEWAL_THRESHOLD,
    PERSISTENT_TARGET_TTL,
};
use soroban_sdk::{
    contract, contracterror, contractimpl, contractmeta, contracttype, log, vec, Address, BytesN,
    Env, IntoVal, String, Symbol, Val, Vec,
};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Collections Deployer Contract"
);

#[contract]
pub struct CollectionsDeployer;

#[contractimpl]
impl CollectionsDeployer {
    pub fn initialize(env: Env, collections_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        if is_initialized(&env) {
            log!(
                &env,
                "Collections Deployer: Initialize: initializing the contract twice is not allowed"
            );
            return Err(ContractError::AlreadyInitialized);
        }
        set_initialized(&env);

        set_wasm_hash(&env, &collections_wasm_hash);

        Ok(())
    }

    pub fn deploy_new_collection(
        env: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
        symbol: String,
    ) -> Result<Address, ContractError> {
        admin.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let collections_wasm_hash = get_wasm_hash(&env)?;

        let deployed_collection = env
            .deployer()
            .with_address(admin.clone(), salt)
            .deploy_v2(collections_wasm_hash, ());

        let init_fn = Symbol::new(&env, "initialize");
        let init_fn_args: Vec<Val> = vec![
            &env,
            admin.into_val(&env),
            name.into_val(&env),
            symbol.into_val(&env),
        ];
        let _: Val = env.invoke_contract(&deployed_collection, &init_fn, init_fn_args);

        save_collection_with_generic_key(&env, name.clone());
        save_collection_with_admin_address_as_key(&env, admin, deployed_collection.clone(), name);

        Ok(deployed_collection)
    }

    pub fn query_all_collections(env: &Env) -> Vec<String> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let maybe_all: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::AllCollections)
            .unwrap_or(Vec::new(env));

        if !maybe_all.is_empty() {
            env.storage().persistent().extend_ttl(
                &DataKey::AllCollections,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
        }

        maybe_all
    }

    pub fn query_collection_by_creator(
        env: &Env,
        creator: Address,
    ) -> Vec<CollectionByCreatorResponse> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let data_key = DataKey::Creator(creator);
        let maybe_collections: Vec<CollectionByCreatorResponse> = env
            .storage()
            .persistent()
            .get(&data_key)
            .unwrap_or(Vec::new(env));

        if !maybe_collections.is_empty() {
            env.storage().persistent().extend_ttl(
                &data_key,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
        }

        maybe_collections
    }
}

// ---------- Storage types ----------

#[contracttype]
#[derive(Clone, Debug)]
pub struct CollectionByCreatorResponse {
    collection: Address,
    name: String,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    IsInitialized,
    CollectionsWasmHash,
    AllCollections,
    Creator(Address),
}

pub fn set_initialized(env: &Env) {
    env.storage().persistent().set(&DataKey::IsInitialized, &());
    env.storage().persistent().extend_ttl(
        &DataKey::IsInitialized,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub fn is_initialized(env: &Env) -> bool {
    let is_initialized = env
        .storage()
        .persistent()
        .get::<_, ()>(&DataKey::IsInitialized)
        .is_some();

    if is_initialized {
        env.storage().persistent().extend_ttl(
            &DataKey::IsInitialized,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }

    is_initialized
}

pub fn set_wasm_hash(env: &Env, hash: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&DataKey::CollectionsWasmHash, hash);
    env.storage().persistent().extend_ttl(
        &DataKey::CollectionsWasmHash,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub fn get_wasm_hash(env: &Env) -> Result<BytesN<32>, ContractError> {
    let wasm_hash: BytesN<32> = env
        .storage()
        .persistent()
        .get(&DataKey::CollectionsWasmHash)
        .ok_or(ContractError::WasmHashNotSet)?;

    env.storage().persistent().extend_ttl(
        &DataKey::CollectionsWasmHash,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );

    Ok(wasm_hash)
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

    env.storage().persistent().extend_ttl(
        &DataKey::AllCollections,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

pub fn save_collection_with_admin_address_as_key(
    env: &Env,
    creator: Address,
    collection_addr: Address,
    name: String,
) {
    let data_key = DataKey::Creator(creator);

    let mut existent_collection: Vec<CollectionByCreatorResponse> = env
        .storage()
        .persistent()
        .get(&data_key)
        .unwrap_or(vec![&env]);

    let new_collection = CollectionByCreatorResponse {
        collection: collection_addr,
        name: name.clone(),
    };

    existent_collection.push_back(new_collection);

    env.storage()
        .persistent()
        .set(&data_key, &existent_collection);
    env.storage().persistent().extend_ttl(
        &data_key,
        PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    );
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    WasmHashNotSet = 0,
    AlreadyInitialized = 1,
}

#[cfg(test)]
mod tests;
