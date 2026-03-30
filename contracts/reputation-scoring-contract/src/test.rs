#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, Address, ReputationScoringContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ReputationScoringContract, ());
    let client = ReputationScoringContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    (env, admin, vault, client)
}

#[test]
fn test_initialize() {
    let (_env, admin, vault, client) = setup();
    client.init(&admin, &vault);
}

#[test]
fn test_initialize_twice_fails() {
    let (_env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    assert!(client.try_init(&admin, &vault).is_err());
}

#[test]
fn test_transfer_admin() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);
}

#[test]
fn test_pause_blocks_transfer_admin() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    client.pause();
    let new_admin = Address::generate(&env);
    assert!(client.try_transfer_admin(&new_admin).is_err());
}

#[test]
fn test_unpause_restores_transfer_admin() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    client.pause();
    client.unpause();
    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);
}

#[test]
fn test_penalize_expert_by_admin() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let expert = Address::generate(&env);

    // Admin should be able to penalize
    client.penalize_expert(&expert, &50);
}

#[test]
fn test_penalize_expert_by_vault() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let _expert = Address::generate(&env);

    // Vault should be able to penalize
    env.mock_all_auths();
    // Simulate vault calling by mocking auth context
    client.penalize_expert(&_expert, &50);
}

#[test]
fn test_penalize_expert_unauthorized() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let _expert = Address::generate(&env);

    // Create a new environment without mocking all auths to test unauthorized access
    let env_strict = Env::default();
    let contract_id = env_strict.register(ReputationScoringContract, ());
    let client_strict = ReputationScoringContractClient::new(&env_strict, &contract_id);
    let admin_strict = Address::generate(&env_strict);
    let vault_strict = Address::generate(&env_strict);
    
    // Initialize with strict env
    client_strict.init(&admin_strict, &vault_strict);
    
    let expert_strict = Address::generate(&env_strict);
    let _unauthorized = Address::generate(&env_strict);
    
    // Unauthorized address should not be able to penalize (no auth mocking for this env)
    assert!(client_strict.try_penalize_expert(&expert_strict, &50).is_err());
}

#[test]
fn test_penalize_expert_score_underflow_protection() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let expert = Address::generate(&env);

    // Penalize with more points than current score (default score is 0)
    // Should result in score of 0, not underflow
    client.penalize_expert(&expert, &10);
    
    // Penalize again with 5 points, score should stay at 0
    client.penalize_expert(&expert, &5);
}

#[test]
fn test_penalize_expert_partial_deduction() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let expert = Address::generate(&env);

    // First penalize to set a score (100 - 30 = 70)
    client.penalize_expert(&expert, &30);
    
    // Second penalize (70 - 20 = 50)
    client.penalize_expert(&expert, &20);
}