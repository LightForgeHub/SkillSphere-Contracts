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
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

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

    /// Batch ban experts and revoke their verification status (Admin only)
    pub fn batch_ban_experts(env: Env, experts: Vec<Address>) -> Result<(), RegistryError> {
        contract::batch_ban_experts(env, experts)
    }

    /// Add an expert to the whitelist (Admin only)
    /// Also saves a profile data_uri reference (e.g., ipfs://...)
    pub fn add_expert(env: Env, expert: Address, data_uri: String) -> Result<(), RegistryError> {
        contract::verify_expert(&env, &expert, data_uri)
    }

    /// Ban an expert and revoke their verification status (Admin only)
    pub fn ban_expert(env: Env, expert: Address) -> Result<(), RegistryError> {
        contract::ban_expert(&env, &expert)
    }

    /// Get the total number of verified experts ever added to the directory
    pub fn get_total_experts(env: Env) -> u64 {
        contract::get_total_experts(&env)
    }

    /// Get the expert address at the given index in the directory
    pub fn get_expert_by_index(env: Env, index: u64) -> Address {
        contract::get_expert_by_index(&env, index)
    }

    /// Get the current status of an expert
    pub fn get_status(env: Env, expert: Address) -> ExpertStatus {
        contract::get_expert_status(&env, &expert)
    }

    /// Check if an expert is verified
    /// Returns true only if the expert's status is Verified
    pub fn is_verified(env: Env, expert: Address) -> bool {
        contract::is_verified(&env, &expert)
    }

    /// Allow a verified expert to update their own profile URI
    pub fn update_profile(env: Env, expert: Address, new_uri: String) -> Result<(), RegistryError> {
        contract::update_profile(&env, &expert, new_uri)
    }

    /// Batch update expert profiles (Admin only)
    /// Allows admins to update multiple expert metadata URIs in a single transaction
    pub fn batch_update_profiles(
        env: Env,
        updates: Vec<(Address, String, u32)>,
    ) -> Result<(), RegistryError> {
        contract::batch_update_profiles(&env, updates)
    }

    /// Get a paginated list of experts
    /// Returns a vector of expert addresses from start_index to start_index + limit
    pub fn get_experts_paginated(env: Env, start_index: u64, limit: u64) -> Vec<Address> {
        contract::get_experts_paginated(&env, start_index, limit)
    }

}
