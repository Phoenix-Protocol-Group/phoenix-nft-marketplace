use soroban_sdk::{contracttype, symbol_short, Address, Bytes, String, Symbol};


pub const ADMIN: Symbol = symbol_short!("ADMIN");

// Struct to represent the operator approval status
#[derive(Clone)]
#[contracttype]
pub struct OperatorApprovalKey {
    pub owner: Address,
    pub operator: Address,
}

/// Struct that represents the Transfer approval status
/// Description.
///
/// * `owner` - The `Address` of the owner of the collection.
/// * `operator` - The `Address` of the operator that we will authorize to do transfer/batch
/// transfer
#[derive(Clone)]
#[contracttype]
pub struct TransferApprovalKey {
    pub owner: Address,
    pub operator: Address,
    pub nft_id: u64,
}

// Enum to represent different data keys in storage
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Balance(Address),
    OperatorApproval(OperatorApprovalKey),
    TransferApproval(TransferApprovalKey),
    Uri(u64),
    CollectionUri,
    Config,
    IsInitialized,
}

// Struct to represent token URI
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct URIValue {
    pub uri: Bytes,
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub name: String,
    pub symbol: String,
}

pub mod utils {

    use helpers::ttl::{PERSISTENT_RENEWAL_THRESHOLD, PERSISTENT_TARGET_TTL};
    use soroban_sdk::{Address, Env, Map};

    use crate::error::ContractError;

    use super::{Config, DataKey};

    pub fn get_balance_of(env: &Env, owner: &Address, id: u64) -> Result<u64, ContractError> {
        let key = DataKey::Balance(owner.clone());
        let balance_map: Map<u64, u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Map::new(env));

        if env.storage().persistent().has(&key) {
            env.storage().persistent().extend_ttl(
                &key,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
        }

        Ok(balance_map.get(id).unwrap_or(0u64))
    }

    pub fn update_balance_of(
        env: &Env,
        owner: &Address,
        id: u64,
        new_amount: u64,
    ) -> Result<(), ContractError> {
        let key = DataKey::Balance(owner.clone());
        let mut balance_map: Map<u64, u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Map::new(env));

        balance_map.set(id, new_amount);

        env.storage().persistent().set(&key, &balance_map);
        env.storage().persistent().extend_ttl(
            &key,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        Ok(())
    }

    pub fn save_config(env: &Env, config: Config) -> Result<(), ContractError> {
        env.storage().persistent().set(&DataKey::Config, &config);

        env.storage().persistent().extend_ttl(
            &DataKey::Config,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_config(env: &Env) -> Result<Config, ContractError> {
        let config = env
            .storage()
            .persistent()
            .get(&DataKey::Config)
            .ok_or(ContractError::ConfigNotFound)?;

        env.storage().persistent().extend_ttl(
            &DataKey::Config,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        Ok(config)
    }

    pub fn save_admin_old(env: &Env, admin: &Address) -> Result<(), ContractError> {
        env.storage().persistent().set(&DataKey::Admin, &admin);

        env.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        Ok(())
    }

    pub fn get_admin_old(env: &Env) -> Result<Address, ContractError> {
        let admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(ContractError::AdminNotSet)?;

        env.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        Ok(admin)
    }

    pub fn is_initialized(env: &Env) -> bool {
        let result: bool = env
            .storage()
            .persistent()
            .get(&DataKey::IsInitialized)
            .unwrap_or(false);

        if result {
            env.storage().persistent().extend_ttl(
                &DataKey::IsInitialized,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
        }

        result
    }

    pub fn set_initialized(env: &Env) {
        env.storage()
            .persistent()
            .set(&DataKey::IsInitialized, &true);

        env.storage().persistent().extend_ttl(
            &DataKey::IsInitialized,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );
    }
}
