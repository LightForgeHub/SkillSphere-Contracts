use crate::error::CalendarError;
use crate::events;
use crate::storage;
use soroban_sdk::{Address, BytesN, Env};

pub fn initialize(
    env: &Env,
    admin: &Address,
    vault_address: &Address,
) -> Result<(), CalendarError> {
    if storage::has_admin(env) {
        return Err(CalendarError::AlreadyInitialized);
    }
    storage::set_admin(env, admin);
    storage::set_vault_address(env, vault_address);
    Ok(())
}

pub fn pause(env: &Env) -> Result<(), CalendarError> {
    let admin = storage::get_admin(env).ok_or(CalendarError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, true);
    events::contract_paused(env, true);
    Ok(())
}

pub fn unpause(env: &Env) -> Result<(), CalendarError> {
    let admin = storage::get_admin(env).ok_or(CalendarError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, false);
    events::contract_paused(env, false);
    Ok(())
}

pub fn transfer_admin(env: &Env, new_admin: &Address) -> Result<(), CalendarError> {
    let admin = storage::get_admin(env).ok_or(CalendarError::NotInitialized)?;
    admin.require_auth();
    if storage::is_paused(env) {
        return Err(CalendarError::ContractPaused);
    }
    storage::set_admin(env, new_admin);
    events::admin_transferred(env, &admin, new_admin);
    Ok(())
}

pub fn upgrade_contract(env: &Env, new_wasm_hash: BytesN<32>) -> Result<(), CalendarError> {
    let admin = storage::get_admin(env).ok_or(CalendarError::NotInitialized)?;
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
    Ok(())
}
