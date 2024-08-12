use soroban_sdk::{contract, contractimpl, log, vec, Address, Bytes, Env, Vec};

use crate::{
    error::ContractError,
    storage::{
        utils::{get_balance_of, update_balance_of},
        DataKey, OperatorApprovalKey,
    },
};

#[contract]
pub struct StellarizedERC1155;

#[contractimpl]
impl StellarizedERC1155 {
    // Returns the balance of the `account` for the token `id`
    #[allow(dead_code)]
    pub fn balance_of(env: Env, account: Address, id: u64) -> Result<u64, ContractError> {
        Ok(get_balance_of(&env, &account, id))?
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
            let current = get_balance_of(&env, &accounts.get(idx).unwrap(), ids.get(idx).unwrap())?;
            batch_balances.insert(idx, current);
        }

        Ok(batch_balances)
    }

    // Grants or revokes permission to `operator` to manage the caller's tokens
    #[allow(dead_code)]
    pub fn set_approval_for_all(
        env: Env,
        sender: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        if sender == operator {
            log!(
                &env,
                "Collection: Set approval for all: Cannot set approval for self"
            );
            return Err(ContractError::CannotApproveSelf);
        }

        env.storage().persistent().set(
            &DataKey::OperatorApproval(OperatorApprovalKey {
                owner: sender,
                operator,
            }),
            &approved,
        );

        Ok(())
    }

    // Returns true if `operator` is approved to manage `owner`'s tokens
    #[allow(dead_code)]
    pub fn is_approved_for_all(
        env: Env,
        owner: Address,
        operator: Address,
    ) -> Result<bool, ContractError> {
        let result = env
            .storage()
            .persistent()
            .get(&DataKey::OperatorApproval(OperatorApprovalKey {
                owner,
                operator,
            }))
            .unwrap_or(false);

        Ok(result)
    }

    // Transfers `amount` tokens of token type `id` from `from` to `to`
    #[allow(dead_code)]
    pub fn safe_transfer_from(
        env: Env,
        from: Address,
        to: Address,
        id: u64,
        transfer_amount: u64,
        _data: Bytes, // we don't have onERC1155Received in Stellar/Soroban
    ) -> Result<(), ContractError> {
        from.require_auth();
        // TODO: check if `to` is not zero address

        let sender_balance = get_balance_of(&env, &from, id)?;
        let rcpt_balance = get_balance_of(&env, &to, id)?;

        if sender_balance < transfer_amount {
            log!(&env, "Collection: Safe transfer from: Insuficient Balance");
            return Err(ContractError::InsuficientBalance);
        }

        //NOTE: checks if we go over the limit of u64::MAX?
        // first we reduce the sender's `from` balance
        update_balance_of(&env, &from, id, sender_balance - transfer_amount)?;

        // next we incrase the recipient's `to` balance
        update_balance_of(&env, &to, id, rcpt_balance + transfer_amount)?;

        Ok(())
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
