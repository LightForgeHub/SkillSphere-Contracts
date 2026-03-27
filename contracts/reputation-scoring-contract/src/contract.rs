use crate::error::ReputationError;
use crate::storage;
use soroban_sdk::{Address, Env};

pub fn initialize(
    env: &Env,
    admin: &Address,
    vault_address: &Address,
) -> Result<(), ReputationError> {
    if storage::has_admin(env) {
        return Err(ReputationError::AlreadyInitialized);
    }
    storage::set_admin(env, admin);
    storage::set_vault_address(env, vault_address);
    Ok(())
}
