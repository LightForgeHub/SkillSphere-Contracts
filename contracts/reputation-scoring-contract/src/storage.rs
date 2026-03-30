use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VaultAddress,
    IsPaused,
    ExpertScore(Address),
    ExpertReviews(Address),
}

pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_vault_address(env: &Env, vault: &Address) {
    env.storage().instance().set(&DataKey::VaultAddress, vault);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::IsPaused, &paused);
}

pub fn get_expert_score(env: &Env, expert: &Address) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ExpertScore(expert.clone()))
        .unwrap_or(0)
}

pub fn set_expert_score(env: &Env, expert: &Address, score: u64) {
    env.storage()
        .instance()
        .set(&DataKey::ExpertScore(expert.clone()), &score);
}

pub fn get_expert_reviews(env: &Env, expert: &Address) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ExpertReviews(expert.clone()))
        .unwrap_or(0)
}

pub fn set_expert_reviews(env: &Env, expert: &Address, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::ExpertReviews(expert.clone()), &count);
}

pub fn get_vault_address(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::VaultAddress)
}
