#![no_std]

mod contract;
mod error;
mod events;
mod storage;
mod types;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use crate::error::RegistryError;

#[contract]
pub struct IdentityRegistryContract;

#[contractimpl]
impl IdentityRegistryContract {
    /// Initialize the contract with an admin
    pub fn init(env: Env, admin: Address) -> Result<(), RegistryError> {
        contract::initialize_registry(&env, &admin)
    }

    /// Add an expert to the whitelist (Admin only)
    pub fn add_expert(env: Env, expert: Address) -> Result<(), RegistryError> {
        contract::verify_expert(&env, &expert)
    }
}