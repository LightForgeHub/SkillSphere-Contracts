use crate::types::{ExpertStats, ReviewRecord};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VaultAddress,
    IsPaused,
    Review(u64),         // booking_id → ReviewRecord
    ExpertStats(Address), // expert → ExpertStats
}

// --- Admin ---

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

// --- Vault ---

pub fn set_vault_address(env: &Env, vault: &Address) {
    env.storage().instance().set(&DataKey::VaultAddress, vault);
}

pub fn get_vault_address(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::VaultAddress)
}

// --- Pause ---

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::IsPaused, &paused);
}

// --- Reviews ---

pub fn has_review(env: &Env, booking_id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Review(booking_id))
}

pub fn set_review(env: &Env, booking_id: u64, review: &ReviewRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Review(booking_id), review);
}

pub fn get_review(env: &Env, booking_id: u64) -> Option<ReviewRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Review(booking_id))
}

// --- Expert Stats ---

pub fn get_expert_stats(env: &Env, expert: &Address) -> ExpertStats {
    env.storage()
        .persistent()
        .get(&DataKey::ExpertStats(expert.clone()))
        .unwrap_or(ExpertStats {
            total_score: 0,
            review_count: 0,
        })
}

pub fn set_expert_stats(env: &Env, expert: &Address, stats: &ExpertStats) {
    env.storage()
        .persistent()
        .set(&DataKey::ExpertStats(expert.clone()), stats);
}
