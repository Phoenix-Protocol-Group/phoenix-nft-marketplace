use helpers::ttl::{
    INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL, PERSISTENT_RENEWAL_THRESHOLD,
    PERSISTENT_TARGET_TTL,
};
use soroban_sdk::{
    contract, contractevent, contractimpl, log, vec, Address, Bytes, BytesN, Env, String, Vec,
};

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

// ---- Contract Events ----

#[contractevent]
pub struct InitializeEvent {
    pub name: String,
    pub symbol: String,
}

#[contractevent]
pub struct ApprovalForAllEvent {
    pub owner: Address,
    pub operator: Address,
    pub approved: bool,
}

#[contractevent]
pub struct ApprovalForTransferEvent {
    pub owner: Address,
    pub operator: Address,
    pub nft_id: u64,
    pub approved: bool,
}

#[contractevent]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub id: u64,
    pub amount: u64,
}

#[contractevent]
pub struct BatchTransferEvent {
    pub from: Address,
    pub to: Address,
    pub ids: Vec<u64>,
    pub amounts: Vec<u64>,
}

#[contractevent]
pub struct MintEvent {
    pub sender: Address,
    pub to: Address,
    pub id: u64,
    pub amount: u64,
}

#[contractevent]
pub struct MintBatchEvent {
    pub sender: Address,
    pub to: Address,
    pub ids: Vec<u64>,
    pub amounts: Vec<u64>,
}

#[contractevent]
pub struct BurnEvent {
    pub from: Address,
    pub id: u64,
    pub amount: u64,
}

#[contractevent]
pub struct BurnBatchEvent {
    pub from: Address,
    pub ids: Vec<u64>,
    pub amounts: Vec<u64>,
}

#[contractevent]
pub struct SetUriEvent {
    pub sender: Address,
    pub id: u64,
    pub uri: Bytes,
}

#[contractevent]
pub struct SetCollectionUriEvent {
    pub uri: Bytes,
}

// ---- Contract ----

#[contract]
pub struct Collections;

#[contractimpl]
impl Collections {
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

        InitializeEvent { name, symbol }.publish(&env);

        Ok(())
    }

    pub fn balance_of(env: Env, account: Address, id: u64) -> Result<u64, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        Ok(get_balance_of(&env, &account, id))?
    }

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

        ApprovalForAllEvent {
            owner: admin,
            operator,
            approved,
        }
        .publish(&env);

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

        ApprovalForTransferEvent {
            owner: admin,
            operator,
            nft_id,
            approved,
        }
        .publish(&env);

        Ok(())
    }

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

    pub fn safe_transfer_from(
        env: Env,
        sender: Address,
        from: Address,
        to: Address,
        id: u64,
        transfer_amount: u64,
    ) -> Result<(), ContractError> {
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
                "Collection: Safe transfer from: Insufficient Balance",
                "Available balance: ",
                from_balance,
                "Amount to send: ",
                transfer_amount
            );
            return Err(ContractError::InsufficientBalance);
        }

        update_balance_of(&env, &from, id, from_balance - transfer_amount)?;
        update_balance_of(&env, &to, id, rcpt_balance + transfer_amount)?;

        TransferEvent {
            from,
            to,
            id,
            amount: transfer_amount,
        }
        .publish(&env);

        Ok(())
    }

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

            update_balance_of(&env, &from, id, sender_balance - amount)?;
            update_balance_of(&env, &to, id, rcpt_balance + amount)?;
        }

        BatchTransferEvent {
            from,
            to,
            ids,
            amounts,
        }
        .publish(&env);

        Ok(())
    }

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

        MintEvent {
            sender,
            to,
            id,
            amount,
        }
        .publish(&env);

        Ok(())
    }

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

        MintBatchEvent {
            sender,
            to,
            ids,
            amounts,
        }
        .publish(&env);

        Ok(())
    }

    pub fn burn(
        env: Env,
        sender: Address,
        from: Address,
        id: u64,
        amount: u64,
    ) -> Result<(), ContractError> {
        if sender != from && !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Burn: Unauthorized. Sender: ", sender);
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
                "Amount to burn: ",
                amount
            );
            return Err(ContractError::InsufficientBalance);
        }

        update_balance_of(&env, &from, id, current_balance - amount)?;

        BurnEvent { from, id, amount }.publish(&env);

        Ok(())
    }

    pub fn burn_batch(
        env: Env,
        sender: Address,
        from: Address,
        ids: Vec<u64>,
        amounts: Vec<u64>,
    ) -> Result<(), ContractError> {
        if sender != from && !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: Burn: Unauthorized. Sender: ", sender);
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
                    " Amount to burn: ",
                    amount
                );
                return Err(ContractError::InsufficientBalance);
            }
            update_balance_of(&env, &from, id, current_balance - amount)?;
        }

        BurnBatchEvent {
            from,
            ids,
            amounts,
        }
        .publish(&env);

        Ok(())
    }

    pub fn set_uri(env: Env, sender: Address, id: u64, uri: Bytes) -> Result<(), ContractError> {
        if !Self::is_authorized_for_all(&env, &sender) {
            log!(&env, "Collections: SetUri: Unauthorized. Sender: ", sender);
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

        SetUriEvent {
            sender,
            id,
            uri,
        }
        .publish(&env);

        Ok(())
    }

    pub fn set_collection_uri(env: Env, sender: Address, uri: Bytes) -> Result<(), ContractError> {
        if !Self::is_authorized_for_all(&env, &sender) {
            log!(
                &env,
                "Collections: SetCollectionUri: Unauthorized. Sender: ",
                sender
            );
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

        SetCollectionUriEvent { uri }.publish(&env);

        Ok(())
    }

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
