#![cfg(test)]
use crate::{IdentityRegistryContract, IdentityRegistryContractClient};
use soroban_sdk::{Env, testutils::Address as _};

#[test]
fn test_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    // 1. Generate a fake admin address
    let admin = soroban_sdk::Address::generate(&env);

    // 2. Call init (Should succeed)
    let res = client.try_init(&admin);
    assert!(res.is_ok());

    // 3. Call init again (Should fail)
    let res_duplicate = client.try_init(&admin);
    assert!(res_duplicate.is_err());
}