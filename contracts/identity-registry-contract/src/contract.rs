use soroban_sdk::{Address, Env};
use crate::storage;
use crate::error::RegistryError;

pub fn initialize_registry(env: &Env, admin: &Address) -> Result<(), RegistryError> {
    if storage::has_admin(env) {
        return Err(RegistryError::AlreadyInitialized);
    }

    storage::set_admin(env, admin);

    Ok(())
}