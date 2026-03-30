#![no_std]

mod contract;
mod error;
mod events;
mod storage;
#[cfg(test)]
mod test;
mod types;

use crate::error::ReputationError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

#[contract]
pub struct ReputationScoringContract;

#[contractimpl]
impl ReputationScoringContract {
    pub fn init(env: Env, admin: Address, vault_address: Address) -> Result<(), ReputationError> {
        contract::initialize(&env, &admin, &vault_address)
    }

    pub fn pause(env: Env) -> Result<(), ReputationError> {
        contract::pause(&env)
    }

    pub fn unpause(env: Env) -> Result<(), ReputationError> {
        contract::unpause(&env)
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), ReputationError> {
        contract::transfer_admin(&env, &new_admin)
    }

    pub fn upgrade_contract(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ReputationError> {
        contract::upgrade_contract(&env, new_wasm_hash)
    }

    pub fn penalize_expert(
        env: Env,
        expert: Address,
        penalty_points: u64,
    ) -> Result<(), ReputationError> {
        contract::penalize_expert(&env, &expert, penalty_points)
    }
}
