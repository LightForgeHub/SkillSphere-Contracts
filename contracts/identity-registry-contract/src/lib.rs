#![no_std]

mod contract;
mod error;
mod events;
mod storage;
#[cfg(test)]
mod test;
mod types;

use crate::error::RegistryError;
use crate::types::ExpertStatus;
use soroban_sdk::{contract, contractimpl, Address, Env,Vec};

#[contract]
pub struct IdentityRegistryContract;

#[contractimpl]
impl IdentityRegistryContract {
    /// Initialize the contract with an admin
    pub fn init(env: Env, admin: Address) -> Result<(), RegistryError> {
        contract::initialize_registry(&env, &admin)
    }

  /// Batch Add an expert to the whitelist (Admin only)
    pub fn batch_add_experts(env: Env, experts: Vec<Address>) -> Result<(), RegistryError> {
        contract::batch_add_experts(env, experts)
    }
    
    /// Add an expert to the whitelist (Admin only)
    pub fn add_expert(env: Env, expert: Address) -> Result<(), RegistryError> {
        contract::verify_expert(&env, &expert)
    }

    /// Ban an expert and revoke their verification status (Admin only)
    pub fn ban_expert(env: Env, expert: Address) -> Result<(), RegistryError> {
        contract::ban_expert(&env, &expert)
    }

    /// Get the current status of an expert
    pub fn get_status(env: Env, expert: Address) -> ExpertStatus {
        contract::get_expert_status(&env, &expert)
    }
}
