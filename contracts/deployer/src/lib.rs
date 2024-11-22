#![no_std]

use soroban_sdk::{
    contract, contractimpl, contractmeta, contracttype, log, vec, Address, BytesN, Env, IntoVal,
    String, Symbol, Val, Vec,
};

// Values used to extend the TTL of storage
pub const DAY_IN_LEDGERS: u32 = 17280;
pub const BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Collections Deployer Contract"
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
                "Collections Deployer: Initialize: initializing the contract twice is not allowed"
            );
            panic!(
                "Collections Deployer: Initialize: initializing the contract twice is not allowed"
            );
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
        symbol: String,
    ) -> Address {
        admin.require_auth();
        let collections_wasm_hash = get_wasm_hash(&env);

        let deployed_collection = env
            .deployer()
            .with_address(admin.clone(), salt)
            .deploy(collections_wasm_hash);

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

        deployed_collection
    }

    pub fn query_all_collections(env: &Env) -> Vec<String> {
        let maybe_all = env
            .storage()
            .persistent()
            .get(&DataKey::AllCollections)
            .unwrap_or(Vec::new(env));

        env.storage()
            .persistent()
            .has(&DataKey::AllCollections)
            .then(|| {
                env.storage().persistent().extend_ttl(
                    &DataKey::AllCollections,
                    LIFETIME_THRESHOLD,
                    BUMP_AMOUNT,
                )
            });

        maybe_all
    }

    pub fn query_collection_by_creator(
        env: &Env,
        creator: Address,
    ) -> Vec<CollectionByCreatorResponse> {
        let data_key = DataKey::Creator(creator);
        let maybe_collections = env
            .storage()
            .persistent()
            .get(&data_key)
            .unwrap_or(Vec::new(env));

        env.storage().persistent().has(&data_key).then(|| {
            env.storage()
                .persistent()
                .extend_ttl(&data_key, LIFETIME_THRESHOLD, BUMP_AMOUNT)
        });

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
    env.storage().instance().set(&DataKey::IsInitialized, &());
    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

pub fn is_initialized(env: &Env) -> bool {
    let is_initialized = env
        .storage()
        .instance()
        .get::<_, ()>(&DataKey::IsInitialized)
        .is_some();

    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);

    is_initialized
}

pub fn set_wasm_hash(env: &Env, hash: &BytesN<32>) {
    env.storage()
        .instance()
        .set(&DataKey::CollectionsWasmHash, hash);
    env.storage()
        .instance()
        .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

pub fn get_wasm_hash(env: &Env) -> BytesN<32> {
    let wasm_hash = env
        .storage()
        .instance()
        .get(&DataKey::CollectionsWasmHash)
        .unwrap();
    env.storage()
        .instance()
        .has(&DataKey::CollectionsWasmHash)
        .then(|| {
            env.storage()
                .instance()
                .extend_ttl(LIFETIME_THRESHOLD, BUMP_AMOUNT)
        });

    wasm_hash
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
        LIFETIME_THRESHOLD,
        BUMP_AMOUNT,
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
    env.storage()
        .persistent()
        .extend_ttl(&data_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);
}

#[cfg(test)]
mod tests;
