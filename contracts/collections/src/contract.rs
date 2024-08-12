use soroban_sdk::{
    contract, contractimpl, log, panic_with_error, vec, Address, Bytes, Env, Symbol, Vec,
};

use crate::error::ContractError;

#[contract]
pub struct ERC1155Equivalent;

// Proposed storage structures
//struct Storage;
//impl Storage {
//    const BALANCES: Symbol = Symbol::new("balances");
//    const OPERATORS: Symbol = Symbol::new("operators");
//    const URIS: Symbol = Symbol::new("uris");
//}

#[contractimpl]
impl ERC1155Equivalent {
    // Returns the balance of the `account` for the token `id`
    #[allow(dead_code)]
    pub fn balance_of(env: Env, account: Address, id: u64) -> Result<u64, ContractError> {
        let result = env
            .storage()
            .persistent()
            .get(&(account, id))
            .unwrap_or_default();

        Ok(result)
    }

    // Returns the balance of multiple `accounts` for multiple `ids`
    #[allow(dead_code)]
    pub fn balance_of_batch(
        env: Env,
        accounts: Vec<Address>,
        ids: Vec<u64>,
    ) -> Result<Vec<u64>, ContractError> {
        if accounts.len() != ids.len() {
            log!(&env, "Collections: Balance of batch: length missmatch");
            return Err(ContractError::AccountsIdsLengthMissmatch);
        }

        let mut batch_balances: Vec<u64> = vec![&env];

        // we verified that the length of both `accounts` and `ids` is the same
        for idx in 0..accounts.len() {
            let temp: u64 = env
                .storage()
                .persistent()
                .get(&(accounts.get(idx), ids.get(idx)))
                .unwrap_or_default();
            batch_balances.insert(idx, temp);
        }

        Ok(batch_balances)
    }

    // Grants or revokes permission to `operator` to manage the caller's tokens
    #[allow(dead_code)]
    pub fn set_approval_for_all(env: Env, operator: Address, approved: bool) {
        todo!()
    }

    // Returns true if `operator` is approved to manage `account`'s tokens
    #[allow(dead_code)]
    pub fn is_approved_for_all(env: Env, account: Address, operator: Address) -> bool {
        todo!()
    }

    // Transfers `amount` tokens of token type `id` from `from` to `to`
    #[allow(dead_code)]
    pub fn safe_transfer_from(
        env: Env,
        from: Address,
        to: Address,
        id: u64,
        amount: u64,
        data: Bytes,
    ) {
        todo!()
    }

    // Transfers multiple types and amounts of tokens from `from` to `to`
    #[allow(dead_code)]
    pub fn safe_batch_transfer_from(
        env: Env,
        from: Address,
        to: Address,
        ids: Vec<u64>,
        amounts: Vec<u64>,
        data: Bytes,
    ) {
        todo!()
    }

    // Mints `amount` tokens of token type `id` to `to`
    #[allow(dead_code)]
    pub fn mint(env: Env, to: Address, id: u64, amount: u64, data: Bytes) {
        todo!()
    }

    // Mints multiple types and amounts of tokens to `to`
    #[allow(dead_code)]
    pub fn mint_batch(env: Env, to: Address, ids: Vec<u64>, amounts: Vec<u64>, data: Bytes) {
        todo!()
    }

    // Destroys `amount` tokens of token type `id` from `from`
    #[allow(dead_code)]
    pub fn burn(env: Env, from: Address, id: u64, amount: u64) {
        todo!()
    }

    // Destroys multiple types and amounts of tokens from `from`
    #[allow(dead_code)]
    pub fn burn_batch(env: Env, from: Address, ids: Vec<u64>, amounts: Vec<u64>) {
        todo!()
    }

    // Sets a new URI for a token type `id`
    #[allow(dead_code)]
    pub fn set_uri(env: Env, id: u64, uri: Bytes) {
        todo!()
    }

    // Returns the URI for a token type `id`
    #[allow(dead_code)]
    pub fn uri(env: Env, id: u64) -> Bytes {
        todo!()
    }
}
