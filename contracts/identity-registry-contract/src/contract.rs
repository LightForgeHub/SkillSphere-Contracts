use soroban_sdk::{Address, Env};
use crate::storage;
use crate::error::RegistryError;
use crate::types::ExpertStatus;
use crate::events;

pub fn initialize_registry(env: &Env, admin: &Address) -> Result<(), RegistryError> {
    if storage::has_admin(env) {
        return Err(RegistryError::AlreadyInitialized);
    }

    storage::set_admin(env, admin);

    Ok(())
}

pub fn verify_expert(env: &Env, expert: &Address) -> Result<(), RegistryError> {
    let admin = storage::get_admin(env).ok_or(RegistryError::NotInitialized)?;
    
    admin.require_auth();
    
    let current_status = storage::get_expert_status(env, expert);
    
    if current_status == ExpertStatus::Verified {
        return Err(RegistryError::AlreadyVerified);
    }
    
    storage::set_expert_record(env, expert, ExpertStatus::Verified);
    
    events::emit_status_change(env, expert.clone(), current_status, ExpertStatus::Verified, admin);
    
    Ok(())
}