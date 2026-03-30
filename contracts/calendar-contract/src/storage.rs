use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    VaultAddress,
    IsPaused,
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
