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

