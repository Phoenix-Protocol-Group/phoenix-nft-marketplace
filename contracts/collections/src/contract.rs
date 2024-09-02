use soroban_sdk::{contract, contractimpl, log, vec, Address, Bytes, BytesN, Env, String, Vec};

use crate::{
    error::ContractError,
    storage::{
        utils::{
            get_admin, get_balance_of, is_initialized, save_admin, save_config, set_initialized,
            update_balance_of,
        },
        Config, DataKey, OperatorApprovalKey, TransferApprovalKey, URIValue,
    },
    ttl::{BUMP_AMOUNT, LIFETIME_THRESHOLD},
};

#[contract]
pub struct Collections;

#[contractimpl]
impl Collections {
    // takes an address and uses it as an administrator
    #[allow(dead_code)]
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
    ) -> Result<(), ContractError> {
        let config = Config {
            name: name.clone(),
            symbol: symbol.clone(),
        };

        if is_initialized(&env) {
            log!(&env, "Collections: Initialize: Already initialized");
            return Err(ContractError::AlreadyInitialized);
        }

        save_config(&env, config)?;
        save_admin(&env, &admin)?;

        set_initialized(&env);

        env.events()
            .publish(("initialize", "collection name: "), name);
        env.events()
            .publish(("initialize", "collectoin symbol: "), symbol);

        Ok(())
    }

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
            let account = accounts
                .get(idx)
                .ok_or(ContractError::InvalidAccountIndex)?;
            let id = ids.get(idx).ok_or(ContractError::InvalidIdIndex)?;

            let current = get_balance_of(&env, &account, id)?;
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

        let data_key = DataKey::OperatorApproval(OperatorApprovalKey {
            owner: sender.clone(),
            operator: operator.clone(),
        });

        env.storage().persistent().set(&data_key, &approved);
        env.storage()
            .persistent()
            .extend_ttl(&data_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);

        env.events()
            .publish(("Set approval for", "Sender: "), sender);
        env.events().publish(
            ("Set approval for", "Set approval for operator: "),
            operator,
        );
        env.events()
            .publish(("Set approval for", "New approval: "), approved);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_approval_for_transfer(
        env: Env,
        mp_address: Address,
        approved: bool,
    ) -> Result<(), ContractError> {
        let admin = get_admin(&env)?;
        admin.require_auth();

        if admin == mp_address {
            log!(
                &env,
                "Collection: Set approval for transfer: Trying to authorize self"
            );
            return Err(ContractError::CannotApproveSelf);
        }

        let data_key = DataKey::TransferApproval(TransferApprovalKey {
            owner: admin.clone(),
            mp_address: mp_address.clone(),
        });

        env.storage().persistent().set(&data_key, &approved);
        env.storage()
            .persistent()
            .extend_ttl(&data_key, LIFETIME_THRESHOLD, BUMP_AMOUNT);

        env.events()
            .publish(("Set approval for transfer", "Sender: "), admin);
        env.events().publish(
            (
                "Set approval for transfer",
                "Set approval for market place addr: ",
            ),
            mp_address,
        );
        env.events()
            .publish(("Set approval for", "New approval: "), approved);

        Ok(())
    }

    // Returns true if `operator` is approved to manage `owner`'s tokens
    #[allow(dead_code)]
    pub fn is_approved_for_all(
        env: Env,
        owner: Address,
        operator: Address,
    ) -> Result<bool, ContractError> {
        let data_key = DataKey::OperatorApproval(OperatorApprovalKey { owner, operator });

        let result = env.storage().persistent().get(&data_key).unwrap_or(false);

        env.storage().persistent().has(&data_key).then(|| {
            env.storage()
                .persistent()
                .extend_ttl(&data_key, LIFETIME_THRESHOLD, BUMP_AMOUNT)
        });

        Ok(result)
    }

    // Transfers `amount` tokens of token type `id` from `from` to `to`
    #[allow(dead_code)]
    pub fn safe_transfer_from(
        env: Env,
        sender: Address,
        from: Address,
        to: Address,
        id: u64,
        transfer_amount: u64,
    ) -> Result<(), ContractError> {
        let operator =  
        // TODO: check if `to` is not zero address

        let sender_balance = get_balance_of(&env, &from, id)?;
        let rcpt_balance = get_balance_of(&env, &to, id)?;

        if sender_balance < transfer_amount {
            log!(&env, "Collection: Safe transfer from: Insuficient Balance");
            return Err(ContractError::InsufficientBalance);
        }

        //NOTE: checks if we go over the limit of u64::MAX?
        // first we reduce the sender's `from` balance
        update_balance_of(&env, &from, id, sender_balance - transfer_amount)?;

        // next we incrase the recipient's `to` balance
        update_balance_of(&env, &to, id, rcpt_balance + transfer_amount)?;

        env.events().publish(("safe transfer from", "from: "), from);
        env.events().publish(("safe transfer from", "to: "), to);
        env.events().publish(("safe transfer from", "id: "), id);
        env.events()
            .publish(("safe transfer from", "transfer amount: "), transfer_amount);

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
    ) -> Result<(), ContractError> {
        from.require_auth();
        // TODO: check if `to` is not zero address

        if ids.len() != amounts.len() {
            log!(
                &env,
                "Collection: Safe batch transfer from: length mismatch"
            );
            return Err(ContractError::IdsAmountsLengthMismatch);
        }

        for idx in 0..ids.len() {
            let id = ids.get(idx).unwrap();
            let amount = amounts.get(idx).unwrap();

            let sender_balance = get_balance_of(&env, &from, id)?;
            let rcpt_balance = get_balance_of(&env, &to, id)?;

            if sender_balance < amount {
                log!(
                    &env,
                    "Collection: Safe batch transfer from: Insufficient Balance"
                );
                return Err(ContractError::InsufficientBalance);
            }

            // Reduce the sender's balance
            update_balance_of(&env, &from, id, sender_balance - amount)?;

            // Increase the recipient's balance
            update_balance_of(&env, &to, id, rcpt_balance + amount)?;
        }

        env.events()
            .publish(("safe batch transfer from", "from: "), from);
        env.events()
            .publish(("safe batch transfer from", "to: "), to);
        env.events()
            .publish(("safe batch transfer from", "ids: "), ids);
        env.events()
            .publish(("safe batch transfer from", "amounts: "), amounts);

        Ok(())
    }

    // Mints `amount` tokens of token type `id` to `to`
    // FIXME: currently this doesn't check if we have minted the same ID twice.
    #[allow(dead_code)]
    pub fn mint(
        env: Env,
        sender: Address,
        to: Address,
        id: u64,
        amount: u64,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        let admin = get_admin(&env)?;
        if admin != sender {
            log!(&env, "Collections: Mint: Unauthorized");
            return Err(ContractError::Unauthorized);
        }

        update_balance_of(&env, &to, id, amount)?;

        env.events().publish(("mint", "sender: "), sender);
        env.events().publish(("mint", "to: "), to);
        env.events().publish(("mint", "id: "), id);
        env.events().publish(("mint", "amount: "), amount);

        Ok(())
    }

    // Mints multiple types and amounts of tokens to `to`
    #[allow(dead_code)]
    pub fn mint_batch(
        env: Env,
        sender: Address,
        to: Address,
        ids: Vec<u64>,
        amounts: Vec<u64>,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        let admin = get_admin(&env)?;
        if admin != sender {
            log!(&env, "Collections: Mint batch: Unauthorized");
            return Err(ContractError::Unauthorized);
        }

        if ids.len() != amounts.len() {
            log!(&env, "Collection: Mint batch: length mismatch");
            return Err(ContractError::IdsAmountsLengthMismatch);
        }

        for idx in 0..ids.len() {
            let id = ids.get(idx).unwrap();
            let amount = amounts.get(idx).unwrap();

            //TODO: check for overflow?
            update_balance_of(&env, &to, id, amount)?;
        }

        env.events().publish(("mint batch", "sender: "), sender);
        env.events().publish(("mint batch", "to: "), to);
        env.events().publish(("mint batch", "ids: "), ids);
        env.events().publish(("mint batch", "amounts: "), amounts);

        Ok(())
    }

    // Destroys `amount` tokens of token type `id` from `from`
    #[allow(dead_code)]
    pub fn burn(env: Env, from: Address, id: u64, amount: u64) -> Result<(), ContractError> {
        let admin = get_admin(&env)?;
        admin.require_auth();

        let current_balance = get_balance_of(&env, &from, id)?;

        if current_balance < amount {
            log!(&env, "Collection: Burn: Insufficient Balance");
            return Err(ContractError::InsufficientBalance);
        }

        update_balance_of(&env, &from, id, current_balance - amount)?;

        env.events().publish(("burn", "from: "), from);
        env.events().publish(("burn", "id: "), id);
        env.events().publish(("burn", "amount: "), amount);

        Ok(())
    }

    // Destroys multiple types and amounts of tokens from `from`
    #[allow(dead_code)]
    pub fn burn_batch(
        env: Env,
        from: Address,
        ids: Vec<u64>,
        amounts: Vec<u64>,
    ) -> Result<(), ContractError> {
        let admin = get_admin(&env)?;
        admin.require_auth();

        if ids.len() != amounts.len() {
            log!(&env, "Collection: Burn batch: length mismatch");
            return Err(ContractError::IdsAmountsLengthMismatch);
        }

        for idx in 0..ids.len() {
            let id = ids.get(idx).unwrap();
            let amount = amounts.get(idx).unwrap();

            let current_balance = get_balance_of(&env, &from, id)?;
            if current_balance < amount {
                log!(&env, "Collection: Burn batch: Insufficient Balance");
                return Err(ContractError::InsufficientBalance);
            }
            update_balance_of(&env, &from, id, current_balance - amount)?;
        }

        env.events().publish(("burn batch", "from: "), from);
        env.events().publish(("burn batch", "ids: "), ids);
        env.events().publish(("burn batch", "amounts: "), amounts);

        Ok(())
    }

    // Sets a new URI for a token type `id`
    #[allow(dead_code)]
    pub fn set_uri(env: Env, sender: Address, id: u64, uri: Bytes) -> Result<(), ContractError> {
        sender.require_auth();
        let admin = get_admin(&env)?;
        if admin != sender {
            log!(&env, "Collections: Set uri: Unauthorized");
            return Err(ContractError::Unauthorized);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Uri(id), &URIValue { uri: uri.clone() });
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Uri(id), LIFETIME_THRESHOLD, BUMP_AMOUNT);

        env.events().publish(("set uri", "sender: "), sender);
        env.events().publish(("set uri", "id: "), id);
        env.events().publish(("set uri", "uri: "), uri);

        Ok(())
    }

    // Sets the main image(logo) for the collection
    #[allow(dead_code)]
    pub fn set_collection_uri(env: Env, uri: Bytes) -> Result<(), ContractError> {
        get_admin(&env)?.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::CollectionUri, &URIValue { uri: uri.clone() });
        env.storage().persistent().extend_ttl(
            &DataKey::CollectionUri,
            LIFETIME_THRESHOLD,
            BUMP_AMOUNT,
        );

        env.events().publish(("set collection uri", "uri: "), uri);

        Ok(())
    }

    // Returns the URI for a token type `id`
    #[allow(dead_code)]
    pub fn uri(env: Env, id: u64) -> Result<URIValue, ContractError> {
        if let Some(uri) = env.storage().persistent().get(&DataKey::Uri(id)) {
            env.storage().persistent().extend_ttl(
                &DataKey::Uri(id),
                LIFETIME_THRESHOLD,
                BUMP_AMOUNT,
            );
            Ok(uri)
        } else {
            log!(&env, "Collections: Uri: No uri set for the given id");
            Err(ContractError::NoUriSet)
        }
    }

    // Returns the URI for a token type `id`
    #[allow(dead_code)]
    pub fn collection_uri(env: Env) -> Result<URIValue, ContractError> {
        if let Some(uri) = env.storage().persistent().get(&DataKey::CollectionUri) {
            env.storage().persistent().extend_ttl(
                &DataKey::CollectionUri,
                LIFETIME_THRESHOLD,
                BUMP_AMOUNT,
            );
            Ok(uri)
        } else {
            log!(&env, "Collections: Uri: No collection uri set");
            Err(ContractError::NoUriSet)
        }
    }

    #[allow(dead_code)]
    #[cfg(not(tarpaulin_include))]
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        let admin: Address = get_admin(&env)?;
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn show_admin(env: &Env) -> Result<Address, ContractError> {
        let maybe_admin = crate::storage::utils::get_admin(env)?;
        Ok(maybe_admin)
    }
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn show_config(env: &Env) -> Result<Config, ContractError> {
        let mabye_config = crate::storage::utils::get_config(env)?;
        Ok(mabye_config)
    }
}
