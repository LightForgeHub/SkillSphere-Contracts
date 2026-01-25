use soroban_sdk::{Address, Env, Vec};
use crate::storage;
use crate::events;
use crate::{error::RegistryError, types::ExpertStatus};

/// Initialize the registry with an admin address
pub fn initialize_registry(env: &Env, admin: &Address) -> Result<(), RegistryError> {
    if storage::has_admin(env) {
        return Err(RegistryError::AlreadyInitialized);
    }

    storage::set_admin(env, admin);

    Ok(())
}

/// Verify an expert by setting their status to Verified (Admin only)
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

/// Batch ban experts by setting their status to Banned (Admin only)
pub fn batch_ban_experts(env: Env, experts: Vec<Address>) -> Result<(), RegistryError> {
    if experts.len() > 20 {
        return Err(RegistryError::ExpertVecMax);
    }

    let admin = storage::get_admin(&env).ok_or(RegistryError::NotInitialized)?;
    admin.require_auth();

    for expert in experts {
        let status = storage::get_expert_status(&env, &expert);
        if status == ExpertStatus::Banned {
            return Err(RegistryError::AlreadyBanned);
        }
        storage::set_expert_record(&env, &expert, ExpertStatus::Banned);
        events::emit_status_change(&env, expert, status, ExpertStatus::Banned, admin.clone());
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

    events::emit_status_change(
        env,
        expert.clone(),
        current_status,
        ExpertStatus::Verified,
        admin,
    );

    Ok(())
}

/// Ban an expert by setting their status to Banned (Admin only)
pub fn ban_expert(env: &Env, expert: &Address) -> Result<(), RegistryError> {
    let admin = storage::get_admin(env).ok_or(RegistryError::NotInitialized)?;
    admin.require_auth();

    let current_status = storage::get_expert_status(env, expert);

    if current_status == ExpertStatus::Banned {
        return Err(RegistryError::AlreadyBanned);
    }

    storage::set_expert_record(env, expert, ExpertStatus::Banned);

    events::emit_status_change(
        env,
        expert.clone(),
        current_status,
        ExpertStatus::Banned,
        admin,
    );

    Ok(())
}

/// Get the current status of an expert
pub fn get_expert_status(env: &Env, expert: &Address) -> ExpertStatus {
    storage::get_expert_status(env, expert)
}

/// Check if an expert is verified
/// Returns true only if the expert's status is Verified
pub fn is_verified(env: &Env, expert: &Address) -> bool {
    storage::get_expert_status(env, expert) == ExpertStatus::Verified
}
