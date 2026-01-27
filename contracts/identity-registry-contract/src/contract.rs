use soroban_sdk::{Address, Env, Vec, String};
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
        // Default empty URI for batch adds
        let empty_uri = String::from_str(&env, "");
        storage::set_expert_record(&env, &expert, ExpertStatus::Verified, empty_uri);
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
        let existing = storage::get_expert_record(&env, &expert);
        storage::set_expert_record(&env, &expert, ExpertStatus::Banned, existing.data_uri);
        events::emit_status_change(&env, expert, status, ExpertStatus::Banned, admin.clone());
    }

    Ok(())
}

pub fn verify_expert(env: &Env, expert: &Address, data_uri: String) -> Result<(), RegistryError> {
    let admin = storage::get_admin(env).ok_or(RegistryError::NotInitialized)?;

    admin.require_auth();

    let current_status = storage::get_expert_status(env, expert);

    if current_status == ExpertStatus::Verified {
        return Err(RegistryError::AlreadyVerified);
    }

    // Validate URI length (limit ~64 chars)
    if data_uri.len() > 64 {
        return Err(RegistryError::UriTooLong);
    }

    storage::set_expert_record(env, expert, ExpertStatus::Verified, data_uri);

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

    // Preserve existing data_uri when banning
    let existing = storage::get_expert_record(env, expert);
    storage::set_expert_record(env, expert, ExpertStatus::Banned, existing.data_uri);

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

/// Allow a verified expert to update their own profile URI
pub fn update_profile(env: &Env, expert: &Address, new_uri: String) -> Result<(), RegistryError> {
    expert.require_auth();

    // Validate URI length
    if new_uri.len() > 64 {
        return Err(RegistryError::UriTooLong);
    }

    let status = storage::get_expert_status(env, expert);
    if status != ExpertStatus::Verified {
        return Err(RegistryError::NotVerified);
    }

    // Update record preserving status
    storage::set_expert_record(env, expert, status, new_uri.clone());
    events::emit_profile_updated(env, expert.clone(), new_uri);
    Ok(())
}
