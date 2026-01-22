#![cfg(test)]

extern crate std;

use crate::{IdentityRegistryContract, IdentityRegistryContractClient};
use crate::{storage, types::ExpertStatus};
use soroban_sdk::{Env, testutils::Address as _, Symbol, Address, IntoVal, TryIntoVal, Vec, vec};
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation, Events};

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

#[test]
#[should_panic]
fn test_batch_verification_no_admin() {
    let env = Env::default();

    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let experts = vec![&env, Address::generate(&env), Address::generate(&env), Address::generate(&env)];

    client.batch_add_experts(&experts);
}

#[test]
fn test_batch_verification_check_status() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::generate(&env);
    client.init(&admin);

    let e1: Address = Address::generate(&env);
    let e2 = Address::generate(&env);
    let e3 = Address::generate(&env);
    let e4 = Address::generate(&env);
    let e5 = Address::generate(&env);

    let experts = vec![&env, e1.clone(), e2.clone(), e3.clone(), e4.clone(), e5.clone()];

    client.batch_add_experts(&experts);

    env.as_contract(&contract_id, ||{
        assert_eq!(storage::get_expert_status(&env, &e1), ExpertStatus::Verified);
        assert_eq!(storage::get_expert_status(&env, &e2), ExpertStatus::Verified);
        assert_eq!(storage::get_expert_status(&env, &e3), ExpertStatus::Verified);
        assert_eq!(storage::get_expert_status(&env, &e4), ExpertStatus::Verified);
        assert_eq!(storage::get_expert_status(&env, &e5), ExpertStatus::Verified);
    })
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_batch_verification_max_vec() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = soroban_sdk::Address::generate(&env);
    client.init(&admin);

    let e1 = Address::generate(&env);
    let e2 = Address::generate(&env);
    let e3 = Address::generate(&env);
    let e4 = Address::generate(&env);

    let experts = vec![&env, e1.clone(), e2.clone(), e3.clone(), e4.clone(), 
        e1.clone(), e2.clone(), e3.clone(), e4.clone(), 
        e1.clone(), e2.clone(), e3.clone(), e4.clone(), 
        e1.clone(), e2.clone(), e3.clone(), e4.clone(),
        e1.clone(), e2.clone(), e3.clone(), e4.clone(),
        e1.clone(), e2.clone(), e3.clone(), e4.clone()
    ];

    client.batch_add_experts(&experts);
}

#[test]
fn test_add_expert() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin);

    let res = client.try_add_expert(&expert);
    assert!(res.is_ok());

    assert_eq!(
        env.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "add_expert"),
                    (expert.clone(),).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )
    );
}

#[test]
#[should_panic]
fn test_add_expert_unauthorized() {
    let env = Env::default();
    
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin);

    client.add_expert(&expert);
}

#[test]
fn test_expert_status_changed_event() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, IdentityRegistryContract);
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin);
    client.add_expert(&expert);

    let events = env.events().all();
    let event = events.last().unwrap();

    assert_eq!(event.0, contract_id);
    
    let topic: Symbol = event.1.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic, Symbol::new(&env, "status_change"));
}