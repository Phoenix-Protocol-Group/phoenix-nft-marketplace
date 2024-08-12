use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AccountsIdsLengthMissmatch = 0,
    CannotApproveSelf = 1,
    InsuficientBalance = 2,
}
