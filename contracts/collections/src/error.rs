use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AccountsIdsLengthMissmatch = 0,
    CannotApproveSelf = 1,
    InsufficientBalance = 2,
    IdsAmountsLengthMismatch = 3,
    NoUriSet = 4,
    AdminNotSet = 5,
    ConfigNotFound = 6,
    Unauthorized = 7,
    InvalidAccountIndex = 8,
    InvalidIdIndex = 9,
    AlreadyInitialized = 10,
    InvalidAmountIndex = 11,
    InvalidId = 12,
}
