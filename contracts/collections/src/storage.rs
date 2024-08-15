use soroban_sdk::{contracttype, symbol_short, Address, Bytes, String, Symbol};

// Constants for storage bump amounts
//pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
//pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
//pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
//pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
//pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

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

// Enum to represent different data keys in storage
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Balance(Address),
    OperatorApproval(OperatorApprovalKey),
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

pub const ADMIN: Symbol = symbol_short!("admin");

pub mod utils {
    use soroban_sdk::{log, Address, Env, Map};

    use crate::error::ContractError;

    use super::{Balance, Config, DataKey, TokenId, ADMIN};

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
        if let Some(config) = env.storage().persistent().get(&DataKey::Config) {
            Ok(config)
        } else {
            log!(&env, "Collections: Get config: Config not set");
            Err(ContractError::ConfigNotFound)
        }
    }

    pub fn save_admin(env: &Env, admin: &Address) -> Result<(), ContractError> {
        env.storage().persistent().set(&ADMIN, &admin);

        Ok(())
    }

    pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
        if let Some(admin) = env.storage().persistent().get(&ADMIN) {
            Ok(admin)
        } else {
            log!(&env, "Collections: Get admin: Admin not set");
            Err(ContractError::AdminNotSet)
        }
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
