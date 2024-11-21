use soroban_sdk::{contracttype, Address, Bytes, String};

type NftId = u64;
type TokenId = u64;
type Balance = u64;

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
    Uri(NftId),
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

    use soroban_sdk::{Address, Env, Map};

    use crate::error::ContractError;

    use super::{Balance, Config, DataKey, TokenId};

    pub fn get_balance_of(env: &Env, owner: &Address, id: u64) -> Result<u64, ContractError> {
        let balance_map: Map<TokenId, Balance> = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(owner.clone()))
            .unwrap_or(Map::new(env));

        if let Some(balance) = balance_map.get(id) {
            Ok(balance)
        } else {
            Ok(0u64)
        }
    }

    pub fn update_balance_of(
        env: &Env,
        owner: &Address,
        id: u64,
        new_amount: u64,
    ) -> Result<(), ContractError> {
        let mut balance_map: Map<TokenId, Balance> = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(owner.clone()))
            .unwrap_or(Map::new(env));

        balance_map.set(id, new_amount);

        env.storage()
            .persistent()
            .set(&DataKey::Balance(owner.clone()), &balance_map);

        Ok(())
    }

    pub fn save_config(env: &Env, config: Config) -> Result<(), ContractError> {
        env.storage().persistent().set(&DataKey::Config, &config);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_config(env: &Env) -> Result<Config, ContractError> {
        let config = env
            .storage()
            .persistent()
            .get(&DataKey::Config)
            .ok_or(ContractError::ConfigNotFound)?;

        Ok(config)
    }

    pub fn save_admin(env: &Env, admin: &Address) -> Result<(), ContractError> {
        env.storage().persistent().set(&DataKey::Admin, &admin);

        Ok(())
    }

    pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
        let admin = env
            .storage()
            .persistent()
            .get(&DataKey::Admin)
            .ok_or(ContractError::AdminNotSet)?;

        Ok(admin)
    }
    pub fn is_initialized(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::IsInitialized)
            .unwrap_or(false)
    }

    pub fn set_initialized(env: &Env) {
        env.storage()
            .persistent()
            .set(&DataKey::IsInitialized, &true);
    }
}
