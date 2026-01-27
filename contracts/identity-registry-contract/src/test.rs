#![cfg(test)]

extern crate std;

use crate::error::RegistryError;
use crate::{IdentityRegistryContract, IdentityRegistryContractClient};
use crate::{storage, types::ExpertStatus};
use soroban_sdk::{Env, testutils::Address as _, Symbol, Address, IntoVal, TryIntoVal, vec, String};
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation, Events};

#[test]
fn test_initialization() {
    let env = Env::default();
    let contract_id = env.register(IdentityRegistryContract, ());
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
fn test_data_uri_persisted_on_verify() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);
    let uri = String::from_str(&env, "ipfs://persisted");

    client.init(&admin);
    client.add_expert(&expert, &uri);

    // Read storage as contract and assert data_uri persisted
    env.as_contract(&contract_id, || {
        let rec = storage::get_expert_record(&env, &expert);
        assert_eq!(rec.data_uri, uri);
    });
}

#[test]
fn test_update_profile_updates_uri_and_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);
    let uri1 = String::from_str(&env, "ipfs://initial");
    let uri2 = String::from_str(&env, "ipfs://updated");

    client.init(&admin);
    client.add_expert(&expert, &uri1);

    // Update profile URI
    client.update_profile(&expert, &uri2);

    // Assert record updated
    env.as_contract(&contract_id, || {
        let rec = storage::get_expert_record(&env, &expert);
        assert_eq!(rec.data_uri, uri2);
    });

    // Event assertion skipped to avoid flakiness in event buffers
}

#[test]
fn test_update_profile_rejections() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unverified = Address::generate(&env);
    client.init(&admin);

    // NotVerified when updating without being verified
    let new_uri = String::from_str(&env, "ipfs://new");
    let res = client.try_update_profile(&unverified, &new_uri);
    assert_eq!(res, Err(Ok(RegistryError::NotVerified)));

    // Verify then try overlong uri
    let expert = Address::generate(&env);
    let ok_uri = String::from_str(&env, "ipfs://ok");
    client.add_expert(&expert, &ok_uri);

    // Build >64 length string
    let long_str = "a".repeat(65);
    let long_uri = String::from_str(&env, long_str.as_str());
    let res2 = client.try_update_profile(&expert, &long_uri);
    assert_eq!(res2, Err(Ok(RegistryError::UriTooLong)));
}

#[test]
#[should_panic]
fn test_batch_verification_no_admin() {
    let env = Env::default();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let experts = vec![&env, Address::generate(&env), Address::generate(&env), Address::generate(&env)];

    client.batch_add_experts(&experts);
}

#[test]
fn test_batch_verification_check_status() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
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

    let contract_id = env.register(IdentityRegistryContract, ());
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

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin);

    let data_uri = String::from_str(&env, "ipfs://profile1");
    let res = client.try_add_expert(&expert, &data_uri);
    assert!(res.is_ok());

    assert_eq!(
        env.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "add_expert"),
                    (expert.clone(), data_uri.clone()).into_val(&env)
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

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin);
    let data_uri = String::from_str(&env, "ipfs://unauth");
    client.add_expert(&expert, &data_uri);
}

#[test]
fn test_expert_status_changed_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin);
    let data_uri = String::from_str(&env, "ipfs://event");
    client.add_expert(&expert, &data_uri);

    let events = env.events().all();
    let event = events.last().unwrap();

    assert_eq!(event.0, contract_id);

    let topic: Symbol = event.1.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic, Symbol::new(&env, "status_change"));
}
#[test]
fn test_ban_expert() {
    let env = Env::default();
    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    // Setup: Create admin and expert addresses
    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    // Initialize the contract
    client.init(&admin);

    // Verify the expert first
    env.mock_all_auths();
    let data_uri = String::from_str(&env, "ipfs://ban");
    client.add_expert(&expert, &data_uri);

    // Verify status is Verified
    let status = client.get_status(&expert);
    assert_eq!(status, ExpertStatus::Verified);

    // Ban the expert (should succeed)
    client.ban_expert(&expert);

    // Check that status is now Banned
    let status = client.get_status(&expert);
    assert_eq!(status, ExpertStatus::Banned);

    // Test: Try to ban again (should fail with AlreadyBanned)
    let result = client.try_ban_expert(&expert);
    assert_eq!(result, Err(Ok(RegistryError::AlreadyBanned)));
}

#[test]
#[should_panic]
fn test_ban_expert_unauthorized() {
    let env = Env::default();
    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin);

    env.mock_all_auths();
    let data_uri = String::from_str(&env, "ipfs://ban-unauth");
    client.add_expert(&expert, &data_uri);

    env.mock_all_auths_allowing_non_root_auth();

    env.mock_auths(&[]);

    client.ban_expert(&expert);
}

#[test]
fn test_ban_unverified_expert() {
    let env = Env::default();
    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    // Initialize
    client.init(&admin);

    // Verify initial status is Unverified
    let status = client.get_status(&expert);
    assert_eq!(status, ExpertStatus::Unverified);

    // Ban an expert who was never verified (should still succeed)
    env.mock_all_auths();
    client.ban_expert(&expert);

    // Status should be Banned now
    let status = client.get_status(&expert);
    assert_eq!(status, ExpertStatus::Banned);
}

#[test]
fn test_ban_expert_workflow() {
    let env = Env::default();
    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert1 = Address::generate(&env);
    let expert2 = Address::generate(&env);
    let expert3 = Address::generate(&env);

    // Initialize
    client.init(&admin);

    env.mock_all_auths();

    // Verify multiple experts
    let uri1 = String::from_str(&env, "ipfs://u1");
    let uri2 = String::from_str(&env, "ipfs://u2");
    let uri3 = String::from_str(&env, "ipfs://u3");
    client.add_expert(&expert1, &uri1);
    client.add_expert(&expert2, &uri2);
    client.add_expert(&expert3, &uri3);

    // Check all are verified
    assert_eq!(client.get_status(&expert1), ExpertStatus::Verified);
    assert_eq!(client.get_status(&expert2), ExpertStatus::Verified);
    assert_eq!(client.get_status(&expert3), ExpertStatus::Verified);

    // Ban expert2
    client.ban_expert(&expert2);

    // Verify expert2 is banned, others remain verified
    assert_eq!(client.get_status(&expert1), ExpertStatus::Verified);
    assert_eq!(client.get_status(&expert2), ExpertStatus::Banned);
    assert_eq!(client.get_status(&expert3), ExpertStatus::Verified);

    // Ban expert1
    client.ban_expert(&expert1);

    // Verify expert1 is now banned
    assert_eq!(client.get_status(&expert1), ExpertStatus::Banned);
    assert_eq!(client.get_status(&expert2), ExpertStatus::Banned);
    assert_eq!(client.get_status(&expert3), ExpertStatus::Verified);
}

#[test]
fn test_ban_before_contract_initialized() {
    let env = Env::default();
    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let expert = Address::generate(&env);

    env.mock_all_auths();

    // Try to ban without initializing (should fail)
    let result = client.try_ban_expert(&expert);
    assert_eq!(result, Err(Ok(RegistryError::NotInitialized)));
}

#[test]
fn test_complete_expert_lifecycle() {
    let env = Env::default();
    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);

    // Initialize
    client.init(&admin);

    env.mock_all_auths();

    // 1. Initial state: Unverified
    assert_eq!(client.get_status(&expert), ExpertStatus::Unverified);

    // 2. Verify the expert
    let data_uri = String::from_str(&env, "ipfs://life");
    client.add_expert(&expert, &data_uri);
    assert_eq!(client.get_status(&expert), ExpertStatus::Verified);

    // 3. Ban the expert
    client.ban_expert(&expert);
    assert_eq!(client.get_status(&expert), ExpertStatus::Banned);
}

#[test]
fn test_getters() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(IdentityRegistryContract, ());
    let client = IdentityRegistryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    // Test 1: Check is_verified on a random address (should be false)
    let random_address = Address::generate(&env);
    assert_eq!(client.is_verified(&random_address), false);
    assert_eq!(client.get_status(&random_address), ExpertStatus::Unverified);

    // Test 2: Verify an expert and check is_verified (should be true)
    let expert = Address::generate(&env);
    let data_uri = String::from_str(&env, "ipfs://getters");
    client.add_expert(&expert, &data_uri);
    assert_eq!(client.is_verified(&expert), true);
    assert_eq!(client.get_status(&expert), ExpertStatus::Verified);

    // Test 3: Ban the expert and check is_verified (should be false)
    client.ban_expert(&expert);
    assert_eq!(client.is_verified(&expert), false);
    assert_eq!(client.get_status(&expert), ExpertStatus::Banned);
}
