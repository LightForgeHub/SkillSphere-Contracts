#![cfg(test)]
use crate::{PaymentVaultContract, PaymentVaultContractClient};
use soroban_sdk::{
    testutils::Address as _,
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

    // Create booking: rate = 10 tokens/second, duration = 100 seconds
    // Total deposit = 10 * 100 = 1000 tokens
    let rate = 10_i128;
    let booked_duration = 100_u64;
    let booking_id = client.create_booking(&user, &expert, &rate, &booked_duration);

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

    // Create booking
    let rate = 10_i128;
    let booked_duration = 100_u64;
    let booking_id = client.create_booking(&user, &expert, &rate, &booked_duration);

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

    let rate = 10_i128;
    let booked_duration = 100_u64;
    let booking_id = client.create_booking(&user, &expert, &rate, &booked_duration);

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

    let rate = 10_i128;
    let booked_duration = 100_u64;
    let booking_id = client.create_booking(&user, &expert, &rate, &booked_duration);

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

    let rate = 10_i128;
    let booked_duration = 100_u64;
    let booking_id = client.create_booking(&user, &expert, &rate, &booked_duration);

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