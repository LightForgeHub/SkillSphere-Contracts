#![no_std]

mod contract;
mod error;
mod events;
mod storage;
#[cfg(test)]
mod test;
mod types;

use crate::error::CalendarError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

#[contract]
pub struct CalendarContract;

#[contractimpl]
impl CalendarContract {
    pub fn init(env: Env, admin: Address, vault_address: Address) -> Result<(), CalendarError> {
        contract::initialize(&env, &admin, &vault_address)
    }

    pub fn pause(env: Env) -> Result<(), CalendarError> {
        contract::pause(&env)
    }

    pub fn unpause(env: Env) -> Result<(), CalendarError> {
        contract::unpause(&env)
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), CalendarError> {
        contract::transfer_admin(&env, &new_admin)
    }

    pub fn upgrade_contract(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), CalendarError> {
        contract::upgrade_contract(&env, new_wasm_hash)
    }
}
