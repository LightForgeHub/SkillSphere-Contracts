#![no_std]

mod contract;
mod error;
mod storage;
#[cfg(test)]
mod test;
mod types;

use crate::error::ReputationError;
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct ReputationScoringContract;

#[contractimpl]
impl ReputationScoringContract {
    pub fn init(env: Env, admin: Address, vault_address: Address) -> Result<(), ReputationError> {
        contract::initialize(&env, &admin, &vault_address)
    }
}
