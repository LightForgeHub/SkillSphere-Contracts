#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ReputationScoringContract, ());
    let client = ReputationScoringContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    client.init(&admin, &vault);
}
