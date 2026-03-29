#![cfg(test)]

use super::*;
use crate::error::CalendarError;
use soroban_sdk::{testutils::Address as _, testutils::Events, Address, Env, Symbol, TryIntoVal};

fn setup() -> (Env, Address, Address, CalendarContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CalendarContract, ());
    let client = CalendarContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    (env, admin, vault, client)
}

#[test]
fn test_initialize() {
    let (_env, admin, vault, client) = setup();
    let res = client.try_init(&admin, &vault);
    assert!(res.is_ok());
}

#[test]
fn test_initialize_twice_fails() {
    let (_env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let res = client.try_init(&admin, &vault);
    assert_eq!(res, Err(Ok(CalendarError::AlreadyInitialized)));
}

#[test]
fn test_pause() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    client.pause();

    let events = env.events().all();
    let last = events.last().unwrap();
    let topic: Symbol = last.1.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic, Symbol::new(&env, "paused"));
}

#[test]
fn test_unpause() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    client.pause();
    client.unpause();

    let events = env.events().all();
    let last = events.last().unwrap();
    let topic: Symbol = last.1.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic, Symbol::new(&env, "paused"));
}

#[test]
fn test_pause_not_initialized() {
    let (_env, _admin, _vault, client) = setup();
    let res = client.try_pause();
    assert_eq!(res, Err(Ok(CalendarError::NotInitialized)));
}

#[test]
fn test_transfer_admin() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let new_admin = Address::generate(&env);
    let res = client.try_transfer_admin(&new_admin);
    assert!(res.is_ok());
}

#[test]
fn test_transfer_admin_emits_event() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    let new_admin = Address::generate(&env);
    client.transfer_admin(&new_admin);

    let events = env.events().all();
    let last = events.last().unwrap();
    let topic: Symbol = last.1.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic, Symbol::new(&env, "adm_xfer"));
}

#[test]
fn test_pause_blocks_transfer_admin() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    client.pause();
    let new_admin = Address::generate(&env);
    let res = client.try_transfer_admin(&new_admin);
    assert_eq!(res, Err(Ok(CalendarError::ContractPaused)));
}

#[test]
fn test_unpause_restores_transfer_admin() {
    let (env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    client.pause();
    client.unpause();
    let new_admin = Address::generate(&env);
    let res = client.try_transfer_admin(&new_admin);
    assert!(res.is_ok());
}

#[test]
#[should_panic]
fn test_pause_requires_auth() {
    let env = Env::default();
    let contract_id = env.register(CalendarContract, ());
    let client = CalendarContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let vault = Address::generate(&env);

    // Init with mocked auth
    env.mock_all_auths();
    client.init(&admin, &vault);

    // Clear auth — pause should panic
    env.mock_auths(&[]);
    client.pause();
}

#[test]
#[should_panic]
fn test_transfer_admin_requires_auth() {
    let env = Env::default();
    let contract_id = env.register(CalendarContract, ());
    let client = CalendarContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    let new_admin = Address::generate(&env);

    env.mock_all_auths();
    client.init(&admin, &vault);

    env.mock_auths(&[]);
    client.transfer_admin(&new_admin);
}
