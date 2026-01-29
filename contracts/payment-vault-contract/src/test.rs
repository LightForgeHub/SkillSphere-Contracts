#![cfg(test)]
use crate::{PaymentVaultContract, PaymentVaultContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

extern crate std;

fn create_client<'a>(env: &'a Env) -> PaymentVaultContractClient<'a> {
    let contract_id = env.register(PaymentVaultContract, ());
    PaymentVaultContractClient::new(env, &contract_id)
}

fn create_token_contract<'a>(env: &'a Env, admin: &Address) -> token::StellarAssetClient<'a> {
    let contract = env.register_stellar_asset_contract_v2(admin.clone());
    token::StellarAssetClient::new(env, &contract.address())
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let client = create_client(&env);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);

    // 1. Successful Init
    let res = client.try_init(&admin, &token, &oracle);
    assert!(res.is_ok());

    // 2. Double Init (Should Fail)
    let res_duplicate = client.try_init(&admin, &token, &oracle);
    assert!(res_duplicate.is_err());
}

#[test]
fn test_partial_duration_scenario() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    // Create token contract and mint tokens to user
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    // Initialize vault
    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Book session: rate = 10 tokens/second, max_duration = 100 seconds
    // Total deposit = 10 * 100 = 1000 tokens
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Verify user's balance decreased
    assert_eq!(token.balance(&user), 9_000);
    assert_eq!(token.balance(&client.address), 1_000);

    // Oracle finalizes with 50% of booked time (50 seconds)
    let actual_duration = 50_u64;
    client.finalize_session(&booking_id, &actual_duration);

    // Expected: expert_pay = 10 * 50 = 500, refund = 1000 - 500 = 500
    assert_eq!(token.balance(&expert), 500);
    assert_eq!(token.balance(&user), 9_500); // 9000 + 500 refund
    assert_eq!(token.balance(&client.address), 0);
}

#[test]
fn test_full_duration_no_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Book session
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Oracle finalizes with full duration (100 seconds)
    let actual_duration = 100_u64;
    client.finalize_session(&booking_id, &actual_duration);

    // Expected: expert_pay = 10 * 100 = 1000, refund = 0
    assert_eq!(token.balance(&expert), 1_000);
    assert_eq!(token.balance(&user), 9_000); // No refund
    assert_eq!(token.balance(&client.address), 0);
}

#[test]
fn test_double_finalization_protection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // First finalization succeeds
    let actual_duration = 50_u64;
    let result = client.try_finalize_session(&booking_id, &actual_duration);
    assert!(result.is_ok());

    // Second finalization should fail (booking no longer Pending)
    let result_duplicate = client.try_finalize_session(&booking_id, &actual_duration);
    assert!(result_duplicate.is_err());
}

#[test]
fn test_oracle_authorization_enforcement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Clear all mocked auths to test Oracle authorization
    env.set_auths(&[]);

    // Try to finalize without any auth (should fail with auth error)
    let result = client.try_finalize_session(&booking_id, &50);
    assert!(result.is_err());

    // Finalize with Oracle auth (should succeed)
    env.mock_all_auths();
    client.finalize_session(&booking_id, &50);

    // Verify finalization succeeded
    assert_eq!(token.balance(&expert), 500);
}

#[test]
fn test_zero_duration_finalization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Oracle finalizes with 0 duration (session cancelled)
    let actual_duration = 0_u64;
    client.finalize_session(&booking_id, &actual_duration);

    // Expected: expert_pay = 0, full refund to user
    assert_eq!(token.balance(&expert), 0);
    assert_eq!(token.balance(&user), 10_000); // Full refund
    assert_eq!(token.balance(&client.address), 0);
}

#[test]
fn test_booking_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin, &token, &oracle);

    // Try to finalize non-existent booking
    let result = client.try_finalize_session(&999, &50);
    assert!(result.is_err());
}

#[test]
fn test_book_session_balance_transfer() {
    // This test specifically verifies the acceptance criteria from Issue #6:
    // - User's balance decreases
    // - Contract's balance increases
    // - Booking ID is unique and retrievable
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    // Initial balance
    let initial_balance = 5_000_i128;
    token.mint(&user, &initial_balance);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Book session with specific deposit
    let rate_per_second = 5_i128;
    let max_duration = 200_u64;
    let expected_deposit = rate_per_second * (max_duration as i128); // 1000 tokens

    // Verify initial state
    assert_eq!(token.balance(&user), initial_balance);
    assert_eq!(token.balance(&client.address), 0);

    // Book session
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Acceptance Criteria #1: User's balance decreases
    assert_eq!(token.balance(&user), initial_balance - expected_deposit);

    // Acceptance Criteria #2: Contract's balance increases
    assert_eq!(token.balance(&client.address), expected_deposit);

    // Acceptance Criteria #3: Booking ID is unique (first booking should be ID 1)
    assert_eq!(booking_id, 1);

    // Create another booking to verify uniqueness
    token.mint(&user, &expected_deposit); // Mint more tokens for second booking
    let booking_id_2 = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Second booking should have different ID
    assert_eq!(booking_id_2, 2);
    assert_ne!(booking_id, booking_id_2);
}

#[test]
fn test_get_user_and_expert_bookings() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert1 = Address::generate(&env);
    let expert2 = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &100_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Create 2 bookings for the same user with different experts
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id_1 = client.book_session(&user, &expert1, &rate_per_second, &max_duration);
    let booking_id_2 = client.book_session(&user, &expert2, &rate_per_second, &max_duration);

    // Test get_user_bookings - should return 2 bookings
    let user_bookings = client.get_user_bookings(&user);
    assert_eq!(user_bookings.len(), 2);
    assert_eq!(user_bookings.get(0).unwrap(), booking_id_1);
    assert_eq!(user_bookings.get(1).unwrap(), booking_id_2);

    // Test get_expert_bookings - expert1 should have 1 booking
    let expert1_bookings = client.get_expert_bookings(&expert1);
    assert_eq!(expert1_bookings.len(), 1);
    assert_eq!(expert1_bookings.get(0).unwrap(), booking_id_1);

    // Test get_expert_bookings - expert2 should have 1 booking
    let expert2_bookings = client.get_expert_bookings(&expert2);
    assert_eq!(expert2_bookings.len(), 1);
    assert_eq!(expert2_bookings.get(0).unwrap(), booking_id_2);

    // Test get_booking - verify we can retrieve booking details
    let booking_1 = client.get_booking(&booking_id_1);
    assert!(booking_1.is_some());
    let booking_1 = booking_1.unwrap();
    assert_eq!(booking_1.id, booking_id_1);
    assert_eq!(booking_1.user, user);
    assert_eq!(booking_1.expert, expert1);
    assert_eq!(booking_1.rate_per_second, rate_per_second);

    // Test get_booking for non-existent booking
    let non_existent = client.get_booking(&999);
    assert!(non_existent.is_none());
}

#[test]
fn test_reclaim_stale_session_too_early() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Create booking
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // User tries to reclaim immediately (should fail - too early)
    let result = client.try_reclaim_stale_session(&user, &booking_id);
    assert!(result.is_err());

    // Verify funds are still in contract
    assert_eq!(token.balance(&client.address), 1_000);
    assert_eq!(token.balance(&user), 9_000);
}

#[test]
fn test_reclaim_stale_session_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Create booking
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Advance ledger timestamp by 25 hours (90000 seconds)
    env.ledger().set_timestamp(env.ledger().timestamp() + 90_000);

    // User tries to reclaim after 25 hours (should succeed)
    let result = client.try_reclaim_stale_session(&user, &booking_id);
    assert!(result.is_ok());

    // Verify funds returned to user
    assert_eq!(token.balance(&client.address), 0);
    assert_eq!(token.balance(&user), 10_000);
    assert_eq!(token.balance(&expert), 0);
}

#[test]
fn test_reclaim_stale_session_wrong_user() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let other_user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Create booking
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Advance ledger timestamp by 25 hours
    env.ledger().set_timestamp(env.ledger().timestamp() + 90_000);

    // Other user tries to reclaim (should fail - not authorized)
    let result = client.try_reclaim_stale_session(&other_user, &booking_id);
    assert!(result.is_err());

    // Verify funds still in contract
    assert_eq!(token.balance(&client.address), 1_000);
}

#[test]
fn test_reclaim_already_finalized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Create booking
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Oracle finalizes the session
    client.finalize_session(&booking_id, &50);

    // Advance ledger timestamp by 25 hours
    env.ledger().set_timestamp(env.ledger().timestamp() + 90_000);

    // User tries to reclaim after finalization (should fail - not pending)
    let result = client.try_reclaim_stale_session(&user, &booking_id);
    assert!(result.is_err());
}

#[test]
fn test_expert_rejects_pending_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    // Create booking
    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Verify initial state
    assert_eq!(token.balance(&user), 9_000);
    assert_eq!(token.balance(&client.address), 1_000);

    // Expert rejects the session
    let result = client.try_reject_session(&expert, &booking_id);
    assert!(result.is_ok());

    // Verify user balance increased (full refund)
    assert_eq!(token.balance(&user), 10_000);
    assert_eq!(token.balance(&client.address), 0);
    assert_eq!(token.balance(&expert), 0);

    // Verify booking status is Rejected
    let booking = client.get_booking(&booking_id).unwrap();
    use crate::types::BookingStatus;
    assert_eq!(booking.status, BookingStatus::Rejected);
}

#[test]
fn test_user_cannot_reject_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // User tries to reject their own session (should fail - not authorized)
    let result = client.try_reject_session(&user, &booking_id);
    assert!(result.is_err());

    // Verify funds still in contract
    assert_eq!(token.balance(&client.address), 1_000);
}

#[test]
fn test_reject_already_complete_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Oracle finalizes the session
    client.finalize_session(&booking_id, &50);

    // Expert tries to reject after completion (should fail - not pending)
    let result = client.try_reject_session(&expert, &booking_id);
    assert!(result.is_err());
}

#[test]
fn test_reject_already_reclaimed_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Advance time and user reclaims
    env.ledger().set_timestamp(env.ledger().timestamp() + 90_000);
    client.reclaim_stale_session(&user, &booking_id);

    // Expert tries to reject after reclamation (should fail - not pending)
    let result = client.try_reject_session(&expert, &booking_id);
    assert!(result.is_err());
}

#[test]
fn test_wrong_expert_cannot_reject() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let wrong_expert = Address::generate(&env);
    let oracle = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = client.book_session(&user, &expert, &rate_per_second, &max_duration);

    // Different expert tries to reject (should fail - not authorized)
    let result = client.try_reject_session(&wrong_expert, &booking_id);
    assert!(result.is_err());

    // Verify funds still in contract
    assert_eq!(token.balance(&client.address), 1_000);
}

#[test]
fn test_reject_nonexistent_booking() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin, &token, &oracle);

    // Expert tries to reject non-existent booking (should fail - not found)
    let result = client.try_reject_session(&expert, &999);
    assert!(result.is_err());
}

