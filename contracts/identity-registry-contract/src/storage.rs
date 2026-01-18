use soroban_sdk::{contracttype, Address, Env};
use crate::types::{ExpertStatus, ExpertRecord};

// 1. Data Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,              
    Expert(Address),    // Mapping: Address -> ExpertRecord
}

// 2. Helper Functions

// --- Admin (Instance Storage) ---
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

// --- Expert (Persistent Storage) ---

pub fn set_expert_record(env: &Env, expert: &Address, status: ExpertStatus) {
    let record = ExpertRecord {
        status,
        updated_at: env.ledger().timestamp(),
    };
    env.storage().persistent().set(&DataKey::Expert(expert.clone()), &record);
}

pub fn get_expert_record(env: &Env, expert: &Address) -> ExpertRecord {
    env.storage()
        .persistent()
        .get(&DataKey::Expert(expert.clone()))
        .unwrap_or(ExpertRecord {
            status: ExpertStatus::Unverified,
            updated_at: 0,
        })
}

pub fn get_expert_status(env: &Env, expert: &Address) -> ExpertStatus {
    get_expert_record(env, expert).status
}