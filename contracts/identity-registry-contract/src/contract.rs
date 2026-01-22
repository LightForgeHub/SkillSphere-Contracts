use soroban_sdk::{Address, Env, Vec};
use crate::storage;
use crate::events;
use crate::{error::RegistryError, types::ExpertStatus};

pub fn initialize_registry(env: &Env, admin: &Address) -> Result<(), RegistryError> {
    if storage::has_admin(env) {
        return Err(RegistryError::AlreadyInitialized);
    }

    storage::set_admin(env, admin);

    Ok(())
}

/// Batch Verification
pub fn batch_add_experts(env:Env, experts: Vec<Address>) -> Result<(), RegistryError> {
    if experts.len() > 20 {
        return Err(RegistryError::ExpertVecMax);
    }

    let admin = storage::get_admin(&env).ok_or(RegistryError::NotInitialized)?;
    admin.require_auth();

    for expert in experts {
        let status = storage::get_expert_status(&env, &expert);
        if status == ExpertStatus::Verified {
            return Err(RegistryError::AlreadyVerified);
        }
        storage::set_expert_record(&env, &expert, ExpertStatus::Verified);
        events::emit_status_change(&env, expert, status, ExpertStatus::Verified, admin.clone());
    }

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