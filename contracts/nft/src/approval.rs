use crate::owner::zero_address;
use crate::storage_types::{ApprovalAll, ApprovalKey, DataKey};
use soroban_sdk::{Address, Env};

pub fn read_approval(env: &Env, id: i128) -> Address {
    let key = DataKey::Approval(ApprovalKey::ID(id));
    if let Some(approval) = env.storage().persistent().get::<_, Address>(&key) {
        approval
    } else {
        zero_address(&env)
    }
}

pub fn read_approval_all(env: &Env, owner: Address, operator: Address) -> bool {
    let key = DataKey::Approval(ApprovalKey::All(ApprovalAll { operator, owner }));
    if let Some(approval) = env.storage().persistent().get::<_, bool>(&key) {
        approval
    } else {
        false
    }
}

pub fn write_approval(env: &Env, id: i128, operator: Address) {
    let key = DataKey::Approval(ApprovalKey::ID(id));
    env.storage().persistent().set(&key, &operator);
}

pub fn write_approval_all(env: &Env, owner: Address, operator: Address, approved: bool) {
    let key = DataKey::Approval(ApprovalKey::All(ApprovalAll { operator, owner }));
    env.storage().persistent().set(&key, &approved);
}
