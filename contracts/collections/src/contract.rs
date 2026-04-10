use helpers::ttl::{
    INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL, PERSISTENT_RENEWAL_THRESHOLD,
    PERSISTENT_TARGET_TTL,
};
use soroban_sdk::{contract, contractimpl, log, vec, Address, Bytes, BytesN, Env, String, Vec};

use crate::{
    error::ContractError,
    storage::{
        utils::{
            get_admin_old, get_balance_of, is_initialized, save_admin_old, save_config,
            set_initialized, update_balance_of,
        },
        Config, DataKey, OperatorApprovalKey, TransferApprovalKey, URIValue, ADMIN,
    },
};

#[contract]
pub struct Collections;

#[contractimpl]
impl Collections {
    // takes an address and uses it as an administrator/owner of the collection
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
        save_admin_old(&env, &admin)?;

        set_initialized(&env);

        env.events()
            .publish(("initialize", "collection name: "), name);
        env.events()
            .publish(("initialize", "collection symbol: "), symbol);

        Ok(())
    }

    // Returns the balance of the `account` for the token `id`
    pub fn balance_of(env: Env, account: Address, id: u64) -> Result<u64, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        Ok(get_balance_of(&env, &account, id))?
    }

    // Returns the balance of multiple `accounts` for multiple `ids`
    pub fn balance_of_batch(
        env: Env,
        accounts: Vec<Address>,
        ids: Vec<u64>,
    ) -> Result<Vec<u64>, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if accounts.len() != ids.len() {
            log!(
                &env,
                "Collections: Balance of batch: length missmatch: ",
                "accounts length: ",
                accounts.len(),
                "ids length: ",
                ids.len()
            );
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

    // Grants or revokes permission to `operator` to manage the caller's assets
    pub fn set_approval_for_all(
        env: Env,
        operator: Address,
        approved: bool,
    ) -> Result<(), ContractError> {
        let admin = get_admin_old(&env)?;
        admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if admin == operator {
            log!(
                &env,
                "Collection: Set approval for all: Cannot set approval for self. Operator: ",
                operator
            );
            return Err(ContractError::CannotApproveSelf);
        }

        let data_key = DataKey::OperatorApproval(OperatorApprovalKey {
            owner: admin.clone(),
            operator: operator.clone(),
        });

        env.storage().persistent().set(&data_key, &approved);
        env.storage().persistent().extend_ttl(
            &data_key,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        env.events()
            .publish(("Set approval for", "Sender: "), admin);
        env.events().publish(
            ("Set approval for", "Set approval for operator: "),
            operator,
        );
        env.events()
            .publish(("Set approval for", "New approval: "), approved);

        Ok(())
    }

    pub fn set_approval_for_transfer(
        env: Env,
        operator: Address,
        nft_id: u64,
        approved: bool,
    ) -> Result<(), ContractError> {
        let admin = get_admin_old(&env)?;
        admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if admin == operator {
            log!(
                &env,
                "Collection: Set approval for transfer: Trying to authorize admin. Operator: ",
                operator
            );
            return Err(ContractError::CannotApproveSelf);
        }

        let data_key = DataKey::TransferApproval(TransferApprovalKey {
            owner: admin.clone(),
            operator: operator.clone(),
            nft_id,
        });

        env.storage().persistent().set(&data_key, &approved);
        env.storage().persistent().extend_ttl(
            &data_key,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        env.events()
            .publish(("Set approval for transfer", "Sender: "), admin);
        env.events().publish(
            (
                "Set approval for transfer",
                "Set approval for operator addr: ",
                "Set approval for nft id: ",
            ),
            (operator, nft_id),
        );
        env.events()
            .publish(("Set approval for", "New approval: "), approved);

        Ok(())
    }

    // Returns true if `operator` is approved to manage `owner`'s tokens
    pub fn is_approved_for_all(env: Env, owner: Address, operator: Address) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let data_key = DataKey::OperatorApproval(OperatorApprovalKey { owner, operator });

        let result: bool = env.storage().persistent().get(&data_key).unwrap_or(false);

        if result {
            env.storage().persistent().extend_ttl(
                &data_key,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
        }

        result
    }

    // Returns true if `operator` is approved to manage `owner`'s tokens
    pub fn is_approved_for_transfer(
        env: Env,
        owner: Address,
        operator: Address,
        nft_id: u64,
    ) -> bool {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let data_key = DataKey::TransferApproval(TransferApprovalKey {
            owner,
            nft_id,
            operator,
        });

        let result: bool = env.storage().persistent().get(&data_key).unwrap_or(false);

        if result {
            env.storage().persistent().extend_ttl(
                &data_key,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
        }

        result
    }

    // Transfers `amount` tokens of token type `id` from `from` to `to`
    pub fn safe_transfer_from(
        env: Env,
        sender: Address,
        from: Address,
        to: Address,
        id: u64,
        transfer_amount: u64,
    ) -> Result<(), ContractError> {
        // if the sender is NOT transferring his own tokens and he's not authorized for transfer then we fail
        if sender != from && !Self::is_authorized_for_transfer(&env, &sender, id) {
            log!(
                &env,
                "Collection: Safe Transfer From: Unauthorized.",
                sender,
                " trying to transfer from ",
                from
            );
            return Err(ContractError::Unauthorized);
        }

        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let from_balance = get_balance_of(&env, &from, id)?;
        let rcpt_balance = get_balance_of(&env, &to, id)?;

        if from_balance < transfer_amount {
            log!(
                &env,
                "Collection: Safe batch transfer from: Insufficient Balance",
                "Available balance: ",
                from_balance,
                "Amount to send: ",
                transfer_amount
            );
            return Err(ContractError::InsufficientBalance);
        }

        // first we reduce `from` balance
        update_balance_of(&env, &from, id, from_balance - transfer_amount)?;

        // next we incrase `to` balance
        update_balance_of(&env, &to, id, rcpt_balance + transfer_amount)?;

        env.events().publish(("safe transfer from", "from: "), from);
        env.events().publish(("safe transfer from", "to: "), to);
        env.events().publish(("safe transfer from", "id: "), id);
        env.events()
            .publish(("safe transfer from", "transfer amount: "), transfer_amount);

        Ok(())
    }

    // Transfers multiple types and amounts of tokens from `from` to `to`
    pub fn safe_batch_transfer_from(
        env: Env,
        sender: Address,
        from: Address,
        to: Address,
        ids: Vec<u64>,
        amounts: Vec<u64>,
    ) -> Result<(), ContractError> {
        if ids.len() != amounts.len() {
            log!(
                &env,
                "Collection: Safe batch transfer from: length mismatch",
                "ids length: ",
                ids.len(),
                "amounts length: ",
                amounts.len()
            );
            return Err(ContractError::IdsAmountsLengthMismatch);
        }

        for idx in 0..ids.len() {
            let nft_id = ids.get(idx).ok_or(ContractError::InvalidIdIndex)?;
            if sender != from && !Self::is_authorized_for_transfer(&env, &sender, nft_id) {
                log!(
                    &env,
                    "Collection: Safe Transfer From: Unauthorized.",
                    sender,
                    " trying to transfer from ",
                    from
                );
                return Err(ContractError::Unauthorized);
            }
        }

        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        for idx in 0..ids.len() {
            let id = ids.get(idx).ok_or(ContractError::InvalidIdIndex)?;
            let amount = amounts.get(idx).ok_or(ContractError::InvalidAmountIndex)?;

            let sender_balance = get_balance_of(&env, &from, id)?;
            let rcpt_balance = get_balance_of(&env, &to, id)?;

            if sender_balance < amount {
                log!(
                    &env,
                    "Collection: Safe batch transfer from: Insufficient balance for id ",
                    id,
                    " Available balance: ",
                    sender_balance,
                    " Amount to send: ",
                    amount
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
    pub fn mint(
        env: Env,
        sender: Address,
        to: Address,
        id: u64,
        amount: u64,
    ) -> Result<(), ContractError> {
        if !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Mint: Unauthorized. Sender: ", sender);
            return Err(ContractError::Unauthorized);
        }

        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let current_balance = get_balance_of(&env, &to, id)?;
        update_balance_of(&env, &to, id, current_balance + amount)?;

        env.events().publish(("mint", "sender: "), sender);
        env.events().publish(("mint", "to: "), to);
        env.events().publish(("mint", "id: "), id);
        env.events().publish(("mint", "amount: "), amount);

        Ok(())
    }

    // Mints multiple types and amounts of tokens to `to`
    pub fn mint_batch(
        env: Env,
        sender: Address,
        to: Address,
        ids: Vec<u64>,
        amounts: Vec<u64>,
    ) -> Result<(), ContractError> {
        if !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Mint: Unauthorized. Sender: ", sender);
            return Err(ContractError::Unauthorized);
        }
        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if ids.len() != amounts.len() {
            log!(
                &env,
                "Collections: Mint Batch: Length missmatch: ",
                "amounts length: ",
                amounts.len(),
                "ids length: ",
                ids.len()
            );
            return Err(ContractError::IdsAmountsLengthMismatch);
        }

        for idx in 0..ids.len() {
            let id = ids.get(idx).ok_or(ContractError::InvalidIdIndex)?;
            let amount = amounts.get(idx).ok_or(ContractError::InvalidAmountIndex)?;

            let current_balance = get_balance_of(&env, &to, id)?;
            update_balance_of(&env, &to, id, current_balance + amount)?;
        }

        env.events().publish(("mint batch", "sender: "), sender);
        env.events().publish(("mint batch", "to: "), to);
        env.events().publish(("mint batch", "ids: "), ids);
        env.events().publish(("mint batch", "amounts: "), amounts);

        Ok(())
    }

    // Destroys `amount` tokens of token type `id` from `from`
    pub fn burn(
        env: Env,
        sender: Address,
        from: Address,
        id: u64,
        amount: u64,
    ) -> Result<(), ContractError> {
        if sender != from && !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Mint: Unauthorized. Sender: ", sender);
            return Err(ContractError::Unauthorized);
        }

        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let current_balance = get_balance_of(&env, &from, id)?;

        if current_balance < amount {
            log!(
                &env,
                "Collection: Burn: Insufficient Balance",
                "Available balance: ",
                current_balance,
                "Amount to transfer: ",
                amount
            );
            return Err(ContractError::InsufficientBalance);
        }

        update_balance_of(&env, &from, id, current_balance - amount)?;

        env.events().publish(("burn", "from: "), from);
        env.events().publish(("burn", "id: "), id);
        env.events().publish(("burn", "amount: "), amount);

        Ok(())
    }

    // Destroys multiple types and amounts of tokens from `from`
    pub fn burn_batch(
        env: Env,
        sender: Address,
        from: Address,
        ids: Vec<u64>,
        amounts: Vec<u64>,
    ) -> Result<(), ContractError> {
        if sender != from && !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Mint: Unauthorized. Sender: ", sender);
            return Err(ContractError::Unauthorized);
        }

        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if ids.len() != amounts.len() {
            log!(
                &env,
                "Collection: Burn batch: length mismatch",
                "ids length: ",
                ids.len(),
                "amounts length: ",
                amounts.len()
            );
            return Err(ContractError::IdsAmountsLengthMismatch);
        }

        for idx in 0..ids.len() {
            let id = ids.get(idx).unwrap();
            let amount = amounts.get(idx).unwrap();

            let current_balance = get_balance_of(&env, &from, id)?;
            if current_balance < amount {
                log!(
                    &env,
                    "Collection: Burn batch: Insufficient balance for id ",
                    id,
                    " Available balance: ",
                    current_balance,
                    " Amount to transfer: ",
                    amount
                );
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
    pub fn set_uri(env: Env, sender: Address, id: u64, uri: Bytes) -> Result<(), ContractError> {
        if !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Mint: Unauthorized. Sender: ", sender);
            return Err(ContractError::Unauthorized);
        }
        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        env.storage()
            .persistent()
            .set(&DataKey::Uri(id), &URIValue { uri: uri.clone() });
        env.storage().persistent().extend_ttl(
            &DataKey::Uri(id),
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        env.events().publish(("set uri", "sender: "), sender);
        env.events().publish(("set uri", "id: "), id);
        env.events().publish(("set uri", "uri: "), uri);

        Ok(())
    }

    // Sets the main image(logo) for the collection
    pub fn set_collection_uri(env: Env, sender: Address, uri: Bytes) -> Result<(), ContractError> {
        if !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Mint: Unauthorized. Sender: ", sender);
            return Err(ContractError::Unauthorized);
        }
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        env.storage()
            .persistent()
            .set(&DataKey::CollectionUri, &URIValue { uri: uri.clone() });
        env.storage().persistent().extend_ttl(
            &DataKey::CollectionUri,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        env.events().publish(("set collection uri", "uri: "), uri);

        Ok(())
    }

    // Returns the URI for a token type `id`
    pub fn uri(env: Env, id: u64) -> Result<URIValue, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if let Some(uri) = env.storage().persistent().get(&DataKey::Uri(id)) {
            env.storage().persistent().extend_ttl(
                &DataKey::Uri(id),
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
            Ok(uri)
        } else {
            log!(&env, "Collections: Uri: No uri set for the given id");
            Err(ContractError::NoUriSet)
        }
    }

    // Returns the URI for a token type `id`
    pub fn collection_uri(env: Env) -> Result<URIValue, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if let Some(uri) = env.storage().persistent().get(&DataKey::CollectionUri) {
            env.storage().persistent().extend_ttl(
                &DataKey::CollectionUri,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            );
            Ok(uri)
        } else {
            log!(&env, "Collections: Uri: No collection uri set");
            Err(ContractError::NoUriSet)
        }
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        let admin: Address = get_admin_old(&env)?;
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    pub fn migrate_admin(env: Env) -> Result<(), ContractError> {
        let admin: Address = get_admin_old(&env)?;
        admin.require_auth();

        env.storage().persistent().set(&ADMIN, &admin);

        Ok(())
    }

    pub fn show_admin(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let maybe_admin = crate::storage::utils::get_admin_old(env)?;
        Ok(maybe_admin)
    }

    pub fn show_config(env: &Env) -> Result<Config, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let mabye_config = crate::storage::utils::get_config(env)?;
        Ok(mabye_config)
    }

    fn is_authorized_for_transfer(env: &Env, sender: &Address, nft_id: u64) -> bool {
        let admin = get_admin_old(env).expect("no admin found");

        admin == sender.clone()
            || Self::is_approved_for_all(env.clone(), admin.clone(), sender.clone())
            || Self::is_approved_for_transfer(env.clone(), admin.clone(), sender.clone(), nft_id)
    }

    fn is_authorized_for_all(env: &Env, sender: &Address) -> bool {
        let admin = get_admin_old(env).expect("no admin found");

        admin == sender.clone() || Self::is_approved_for_all(env.clone(), admin, sender.clone())
    }
}
