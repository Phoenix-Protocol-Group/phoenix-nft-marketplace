use soroban_sdk::{contracttype, symbol_short, Address, Bytes, String, Symbol};

// Constants for storage bump amounts
//pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
//pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
//pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
//pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
//pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

type NftId = u64;

// Struct to represent a token balance for a specific address and token ID
#[derive(Clone)]
#[contracttype]
pub struct BalanceDataKey {
    pub token_id: u64,
    pub owner: Address,
}

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
    Balance(BalanceDataKey),
    OperatorApproval(OperatorApprovalKey),
    Uri(NftId),
    Config,
}

// Struct to represent token URI
#[derive(Clone)]
#[contracttype]
pub struct URIValue {
    pub uri: Bytes,
}

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub name: String,
    pub image: URIValue,
}

pub const ADMIN: Symbol = symbol_short!("admin");

pub mod utils {
    use soroban_sdk::{log, Address, Env};

    use crate::error::ContractError;

    use super::{Config, DataKey, ADMIN};

    pub fn get_balance_of(env: &Env, owner: &Address, id: u64) -> Result<u64, ContractError> {
        let result = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(crate::storage::BalanceDataKey {
                token_id: id,
                owner: owner.clone(),
            }))
            .unwrap_or(0u64);

        Ok(result)
    }

    pub fn update_balance_of(
        env: &Env,
        owner: &Address,
        id: u64,
        new_amount: u64,
    ) -> Result<(), ContractError> {
        env.storage().persistent().set(
            &DataKey::Balance(crate::storage::BalanceDataKey {
                token_id: id,
                owner: owner.clone(),
            }),
            &new_amount,
        );

        Ok(())
    }

    pub fn save_config(env: &Env, config: Config) -> Result<(), ContractError> {
        env.storage().persistent().set(&DataKey::Config, &config);

        Ok(())
    }

    pub fn _get_config(env: &Env) -> Result<Config, ContractError> {
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
}
