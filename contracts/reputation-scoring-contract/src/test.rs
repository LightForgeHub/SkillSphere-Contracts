#![cfg(test)]

extern crate std;

use super::*;
use crate::error::ReputationError;
use crate::types::BookingStatus;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, testutils::Events, Address, Env, Symbol,
    TryIntoVal,
};

// ── Mock Vault Contract ──────────────────────────────────────────────────

/// A minimal mock of the PaymentVault that returns a canned BookingRecord.
/// It stores a single booking at id=1 with configurable user, expert, and status.
#[contract]
pub struct MockVault;

#[contractimpl]
impl MockVault {
    /// Store a mock booking. status: 0=Pending, 1=Complete, etc.
    pub fn set_booking(env: Env, user: Address, expert: Address, status: u32) {
        use crate::types::BookingRecord;
        let booking = BookingRecord {
            id: 1,
            user,
            expert,
            rate_per_second: 100,
            max_duration: 3600,
            total_deposit: 360_000,
            status: match status {
                0 => BookingStatus::Pending,
                1 => BookingStatus::Complete,
                2 => BookingStatus::Rejected,
                3 => BookingStatus::Reclaimed,
                _ => BookingStatus::Cancelled,
            },
            created_at: 1000,
            started_at: None,
        };
        env.storage().persistent().set(&1u64, &booking);
    }

    /// Matches the vault's get_booking(booking_id) → BookingRecord
    pub fn get_booking(env: Env, booking_id: u64) -> crate::types::BookingRecord {
        env.storage()
            .persistent()
            .get(&booking_id)
            .expect("booking not found")
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn setup() -> (Env, Address, Address, ReputationScoringContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ReputationScoringContract, ());
    let client = ReputationScoringContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    (env, admin, vault, client)
}

fn setup_with_vault() -> (
    Env,
    Address,
    Address,
    Address,
    Address,
    ReputationScoringContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Register mock vault
    let vault_id = env.register(MockVault, ());
    let vault_client = MockVaultClient::new(&env, &vault_id);

    // Register reputation contract
    let contract_id = env.register(ReputationScoringContract, ());
    let client = ReputationScoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);

    // Initialize reputation contract pointing at mock vault
    client.init(&admin, &vault_id);

    // Set up a completed booking (id=1) with user and expert
    vault_client.set_booking(&user, &expert, &1u32);

    (env, admin, user, expert, vault_id, client)
}

// ── Existing tests ───────────────────────────────────────────────────────

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

// ── submit_review tests ──────────────────────────────────────────────────

#[test]
fn test_submit_review_success() {
    let (_env, _admin, user, expert, _vault_id, client) = setup_with_vault();

    let res = client.try_submit_review(&user, &1u64, &4u32);
    assert!(res.is_ok());

    // Verify review stored
    let review = client.get_review(&1u64).unwrap();
    assert_eq!(review.booking_id, 1);
    assert_eq!(review.reviewer, user);
    assert_eq!(review.expert, expert);
    assert_eq!(review.score, 4);

    // Verify expert stats updated
    let stats = client.get_expert_stats(&expert);
    assert_eq!(stats.total_score, 4);
    assert_eq!(stats.review_count, 1);
}

#[test]
fn test_submit_review_emits_event() {
    let (_env, _admin, user, _expert, _vault_id, client) = setup_with_vault();

    client.submit_review(&user, &1u64, &5u32);

    let events = _env.events().all();
    let last = events.last().unwrap();

    let topic: Symbol = last.1.get(0).unwrap().try_into_val(&_env).unwrap();
    assert_eq!(topic, Symbol::new(&_env, "review"));
}

#[test]
fn test_submit_review_invalid_score_zero() {
    let (_env, _admin, user, _expert, _vault_id, client) = setup_with_vault();

    let res = client.try_submit_review(&user, &1u64, &0u32);
    assert_eq!(res, Err(Ok(ReputationError::InvalidScore)));
}

#[test]
fn test_submit_review_invalid_score_six() {
    let (_env, _admin, user, _expert, _vault_id, client) = setup_with_vault();

    let res = client.try_submit_review(&user, &1u64, &6u32);
    assert_eq!(res, Err(Ok(ReputationError::InvalidScore)));
}

#[test]
fn test_submit_review_duplicate() {
    let (_env, _admin, user, _expert, _vault_id, client) = setup_with_vault();

    client.submit_review(&user, &1u64, &3u32);
    let res = client.try_submit_review(&user, &1u64, &5u32);
    assert_eq!(res, Err(Ok(ReputationError::AlreadyReviewed)));
}

#[test]
fn test_submit_review_booking_not_complete() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(MockVault, ());
    let vault_client = MockVaultClient::new(&env, &vault_id);

    let contract_id = env.register(ReputationScoringContract, ());
    let client = ReputationScoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin, &vault_id);

    // Set booking as Pending (status=0)
    vault_client.set_booking(&user, &expert, &0u32);

    let res = client.try_submit_review(&user, &1u64, &4u32);
    assert_eq!(res, Err(Ok(ReputationError::BookingNotComplete)));
}

#[test]
fn test_submit_review_wrong_user() {
    let (env, _admin, _user, _expert, _vault_id, client) = setup_with_vault();

    let stranger = Address::generate(&env);
    let res = client.try_submit_review(&stranger, &1u64, &4u32);
    assert_eq!(res, Err(Ok(ReputationError::NotBookingUser)));
}

#[test]
fn test_submit_review_paused() {
    let (_env, _admin, user, _expert, _vault_id, client) = setup_with_vault();

    client.pause();
    let res = client.try_submit_review(&user, &1u64, &4u32);
    assert_eq!(res, Err(Ok(ReputationError::ContractPaused)));
}

#[test]
fn test_expert_stats_accumulate() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(MockVault, ());
    let vault_client = MockVaultClient::new(&env, &vault_id);

    let contract_id = env.register(ReputationScoringContract, ());
    let client = ReputationScoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin, &vault_id);

    // Booking 1: user1 → expert, Complete
    vault_client.set_booking(&user1, &expert, &1u32);
    client.submit_review(&user1, &1u64, &5u32);

    // Booking 2: user2 → expert, Complete (store at id=2)
    // We need a second booking. Override storage for id=2.
    env.as_contract(&vault_id, || {
        use crate::types::BookingRecord;
        let booking = BookingRecord {
            id: 2,
            user: user2.clone(),
            expert: expert.clone(),
            rate_per_second: 100,
            max_duration: 3600,
            total_deposit: 360_000,
            status: BookingStatus::Complete,
            created_at: 2000,
            started_at: None,
        };
        env.storage().persistent().set(&2u64, &booking);
    });

    client.submit_review(&user2, &2u64, &3u32);

    let stats = client.get_expert_stats(&expert);
    assert_eq!(stats.total_score, 8); // 5 + 3
    assert_eq!(stats.review_count, 2);
}

#[test]
fn test_score_boundary_values() {
    let (_env, _admin, user, expert, _vault_id, client) = setup_with_vault();

    // Score 1 (minimum valid)
    let res = client.try_submit_review(&user, &1u64, &1u32);
    assert!(res.is_ok());

    let stats = client.get_expert_stats(&expert);
    assert_eq!(stats.total_score, 1);
    assert_eq!(stats.review_count, 1);
}

// ── penalize_expert tests ────────────────────────────────────────────────

#[test]
fn test_penalize_expert_success() {
    let (_env, _admin, user, expert, _vault_id, client) = setup_with_vault();

    // Give the expert some score first
    client.submit_review(&user, &1u64, &5u32);
    assert_eq!(client.get_expert_stats(&expert).total_score, 5);

    // Penalize by 3
    let res = client.try_penalize_expert(&expert, &3u64);
    assert!(res.is_ok());

    let stats = client.get_expert_stats(&expert);
    assert_eq!(stats.total_score, 2);
    assert_eq!(stats.review_count, 1); // review_count unchanged
}

#[test]
fn test_penalize_expert_no_underflow() {
    let (_env, _admin, user, expert, _vault_id, client) = setup_with_vault();

    // Score is 5
    client.submit_review(&user, &1u64, &5u32);

    // Penalize by 100 — should saturate to 0
    client.penalize_expert(&expert, &100u64);

    let stats = client.get_expert_stats(&expert);
    assert_eq!(stats.total_score, 0);
}

#[test]
fn test_penalize_expert_zero_score() {
    let (_env, admin, _vault, client) = setup();
    client.init(&admin, &_vault);

    let expert = Address::generate(&_env);

    // Expert has no stats (total_score=0), penalize should saturate to 0
    client.penalize_expert(&expert, &5u64);

    let stats = client.get_expert_stats(&expert);
    assert_eq!(stats.total_score, 0);
}

#[test]
fn test_penalize_expert_zero_amount_fails() {
    let (_env, admin, vault, client) = setup();
    client.init(&admin, &vault);

    let expert = Address::generate(&_env);
    let res = client.try_penalize_expert(&expert, &0u64);
    assert_eq!(res, Err(Ok(ReputationError::InvalidPenalty)));
}

#[test]
fn test_penalize_expert_emits_event() {
    let (_env, _admin, user, expert, _vault_id, client) = setup_with_vault();

    client.submit_review(&user, &1u64, &5u32);
    client.penalize_expert(&expert, &2u64);

    let events = _env.events().all();
    let last = events.last().unwrap();

    let topic: Symbol = last.1.get(0).unwrap().try_into_val(&_env).unwrap();
    assert_eq!(topic, Symbol::new(&_env, "penalize"));
}

#[test]
fn test_penalize_expert_paused() {
    let (_env, admin, vault, client) = setup();
    client.init(&admin, &vault);
    client.pause();

    let expert = Address::generate(&_env);
    let res = client.try_penalize_expert(&expert, &1u64);
    assert_eq!(res, Err(Ok(ReputationError::ContractPaused)));
}

#[test]
#[should_panic]
fn test_penalize_expert_not_admin() {
    let env = Env::default();

    let contract_id = env.register(ReputationScoringContract, ());
    let client = ReputationScoringContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vault = Address::generate(&env);
    let expert = Address::generate(&env);

    client.init(&admin, &vault);

    // No auth mocked — should panic on admin.require_auth()
    client.penalize_expert(&expert, &1u64);
}
