use crate::storage_types::DataKey;
use soroban_sdk::{Address, Env};

pub fn read_minted(env: &Env, owner: Address) -> bool {
    let key = DataKey::Minted(owner);
    match env.storage().persistent().get::<_, bool>(&key) {
        Some(minted) => minted,
        None => false,
    }
}

pub fn write_minted(env: &Env, owner: Address) {
    let key = DataKey::Minted(owner);
    env.storage().persistent().set(&key, &true);
}

pub fn check_minted(env: &Env, owner: Address) {
    assert!(!read_minted(&env, owner), "already minted");
}
