use soroban_sdk::{contracttype, Address, Bytes};

// Constants for storage bump amounts
pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

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
    URI(u64),
}

// Struct to represent token URI
#[contracttype]
pub struct URIValue {
    pub uri: Bytes,
}

pub mod utils {
    use soroban_sdk::{Address, Env};

    use crate::error::ContractError;

    use super::DataKey;

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
}
