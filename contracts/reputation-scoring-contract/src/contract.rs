use crate::error::ReputationError;
use crate::events;
use crate::storage;
use soroban_sdk::{Address, BytesN, Env};

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

pub fn pause(env: &Env) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, true);
    events::contract_paused(env, true);
    Ok(())
}

pub fn unpause(env: &Env) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, false);
    events::contract_paused(env, false);
    Ok(())
}

pub fn transfer_admin(env: &Env, new_admin: &Address) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    if storage::is_paused(env) {
        return Err(ReputationError::ContractPaused);
    }
    storage::set_admin(env, new_admin);
    events::admin_transferred(env, &admin, new_admin);
    Ok(())
}

pub fn upgrade_contract(env: &Env, new_wasm_hash: BytesN<32>) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
    Ok(())
}

pub fn penalize_expert(
    env: &Env,
    expert: &Address,
    penalty_points: u64,
) -> Result<(), ReputationError> {
    // Verify contract is initialized
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;

    // Require auth from admin (vault authorization is handled through admin)
    admin.require_auth();

    // Get current score
    let current_score = storage::get_expert_score(env, expert);

    // Subtract penalty points, floor at 0 (prevent underflow)
    let new_score = if current_score > penalty_points {
        current_score - penalty_points
    } else {
        0
    };

    // Update score (do not increment review count)
    storage::set_expert_score(env, expert, new_score);

    // Emit event
    events::expert_penalized(env, expert, penalty_points, new_score);

    Ok(())
}
