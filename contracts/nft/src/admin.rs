use crate::storage_types::DataKey;
use soroban_sdk::{Address, Env};

pub fn has_administrator(env: &Env) -> bool {
    let key = DataKey::Admin;
    env.storage().persistent().has(&key)
}

pub fn read_administrator(env: &Env) -> Address {
    let key = DataKey::Admin;
    env.storage().persistent().get(&key).unwrap()
}

pub fn write_administrator(env: &Env, id: Address) {
    let key = DataKey::Admin;
    env.storage().persistent().set(&key, &id);
}

pub fn check_admin(env: &Env, auth: &Address) {
    assert!(auth == &read_administrator(env), "not authorized by admin");
}
