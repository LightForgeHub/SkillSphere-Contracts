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

// Mock Identity Registry contract that returns configurable value for is_verified
mod mock_registry {
    use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

    #[contract]
    pub struct MockRegistry;

    #[contractimpl]
    impl MockRegistry {
        pub fn is_verified(env: Env, _expert: Address) -> bool {
            // Read the verification state from the registry's storage
            // For simplicity, we'll use an internal storage key
            let key = Symbol::new(&env, "is_verified");
            env.storage().instance().get(&key).unwrap_or(true)
        }

        pub fn set_verified(env: Env, verified: bool) {
            let key = Symbol::new(&env, "is_verified");
            env.storage().instance().set(&key, &verified);
        }
    }
}

// Create a mock registry contract that returns true for is_verified
fn create_mock_registry<'a>(env: &'a Env) -> Address {
    let contract_id = env.register(mock_registry::MockRegistry, ());
    contract_id
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let client = create_client(&env);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    // 1. Successful Init
    let res = client.try_init(&admin, &token, &oracle, &registry);
    assert!(res.is_ok());

    // 2. Double Init (Should Fail)
    let res_duplicate = client.try_init(&admin, &token, &oracle, &registry);
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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    assert_eq!(token.balance(&user), 9_000);
    assert_eq!(token.balance(&client.address), 1_000);

    let actual_duration = 50_u64;
    client.finalize_session(&booking_id, &actual_duration);

    assert_eq!(token.balance(&expert), 500);
    assert_eq!(token.balance(&user), 9_500);
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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    let actual_duration = 100_u64;
    client.finalize_session(&booking_id, &actual_duration);

    assert_eq!(token.balance(&expert), 1_000);
    assert_eq!(token.balance(&user), 9_000);
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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    let actual_duration = 50_u64;
    let result = client.try_finalize_session(&booking_id, &actual_duration);
    assert!(result.is_ok());

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    env.set_auths(&[]);

    let result = client.try_finalize_session(&booking_id, &50);
    assert!(result.is_err());

    env.mock_all_auths();
    client.finalize_session(&booking_id, &50);

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    let actual_duration = 0_u64;
    client.finalize_session(&booking_id, &actual_duration);

    assert_eq!(token.balance(&expert), 0);
    assert_eq!(token.balance(&user), 10_000);
    assert_eq!(token.balance(&client.address), 0);
}

#[test]
fn test_booking_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);
    let token = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin, &token, &oracle, &registry);

    let result = client.try_finalize_session(&999, &50);
    assert!(result.is_err());
}

#[test]
fn test_book_session_balance_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    let initial_balance = 5_000_i128;
    token.mint(&user, &initial_balance);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 5_i128;
    let max_duration = 200_u64;
    let expected_deposit = rate_per_second * (max_duration as i128);

    assert_eq!(token.balance(&user), initial_balance);
    assert_eq!(token.balance(&client.address), 0);

    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    assert_eq!(token.balance(&user), initial_balance - expected_deposit);
    assert_eq!(token.balance(&client.address), expected_deposit);
    assert_eq!(booking_id, 1);

    token.mint(&user, &expected_deposit);
    let booking_id_2 = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &100_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id_1 = {
        client.set_my_rate(&expert1, &rate_per_second);
        client.book_session(&user, &expert1, &max_duration)
    };
    let booking_id_2 = {
        client.set_my_rate(&expert2, &rate_per_second);
        client.book_session(&user, &expert2, &max_duration)
    };

    // Paginated: fetch all 2 user bookings starting at index 0
    let user_bookings = client.get_user_bookings(&user, &0, &10);
    assert_eq!(user_bookings.len(), 2);
    assert_eq!(user_bookings.get(0).unwrap(), booking_id_1);
    assert_eq!(user_bookings.get(1).unwrap(), booking_id_2);

    // Count
    assert_eq!(client.get_user_booking_count(&user), 2);

    // Expert1 has 1 booking
    let expert1_bookings = client.get_expert_bookings(&expert1, &0, &10);
    assert_eq!(expert1_bookings.len(), 1);
    assert_eq!(expert1_bookings.get(0).unwrap(), booking_id_1);
    assert_eq!(client.get_expert_booking_count(&expert1), 1);

    // Expert2 has 1 booking
    let expert2_bookings = client.get_expert_bookings(&expert2, &0, &10);
    assert_eq!(expert2_bookings.len(), 1);
    assert_eq!(expert2_bookings.get(0).unwrap(), booking_id_2);
    assert_eq!(client.get_expert_booking_count(&expert2), 1);

    // get_booking works
    let booking_1 = client.get_booking(&booking_id_1);
    assert!(booking_1.is_some());
    let booking_1 = booking_1.unwrap();
    assert_eq!(booking_1.id, booking_id_1);
    assert_eq!(booking_1.user, user);
    assert_eq!(booking_1.expert, expert1);
    assert_eq!(booking_1.rate_per_second, rate_per_second);

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    let result = client.try_reclaim_stale_session(&user, &booking_id);
    assert!(result.is_err());

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 90_000);

    let result = client.try_reclaim_stale_session(&user, &booking_id);
    assert!(result.is_ok());

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 90_000);

    let result = client.try_reclaim_stale_session(&other_user, &booking_id);
    assert!(result.is_err());

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    client.finalize_session(&booking_id, &50);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 90_000);

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    assert_eq!(token.balance(&user), 9_000);
    assert_eq!(token.balance(&client.address), 1_000);

    let result = client.try_reject_session(&expert, &booking_id);
    assert!(result.is_ok());

    assert_eq!(token.balance(&user), 10_000);
    assert_eq!(token.balance(&client.address), 0);
    assert_eq!(token.balance(&expert), 0);

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    let result = client.try_reject_session(&user, &booking_id);
    assert!(result.is_err());

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    client.finalize_session(&booking_id, &50);

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 90_000);
    client.reclaim_stale_session(&user, &booking_id);

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
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let rate_per_second = 10_i128;
    let max_duration = 100_u64;
    let booking_id = {
        client.set_my_rate(&expert, &rate_per_second);
        client.book_session(&user, &expert, &max_duration)
    };

    let result = client.try_reject_session(&wrong_expert, &booking_id);
    assert!(result.is_err());

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
    let registry = create_mock_registry(&env);

    let client = create_client(&env);
    client.init(&admin, &token, &oracle, &registry);

    let result = client.try_reject_session(&expert, &999);
    assert!(result.is_err());
}

// ==================== Key Rotation Tests ====================

#[test]
fn test_transfer_admin_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin_a = Address::generate(&env);
    let admin_b = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin_a, &token, &oracle, &registry);

    // Admin A transfers to Admin B
    let result = client.try_transfer_admin(&admin_b);
    assert!(result.is_ok());
}

#[test]
fn test_new_admin_can_pause_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin_a = Address::generate(&env);
    let admin_b = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin_a, &token, &oracle, &registry);
    client.transfer_admin(&admin_b);

    // New admin B can pause and unpause
    assert!(client.try_pause().is_ok());
    assert!(client.try_unpause().is_ok());
}

#[test]
fn test_old_admin_loses_privileges_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin_a = Address::generate(&env);
    let admin_b = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin_a, &token, &oracle, &registry);
    client.transfer_admin(&admin_b);

    // Remove all mocked auths — now only explicit auth will pass
    env.set_auths(&[]);

    // Without any valid auth for admin_b, pause should fail
    // (admin_b is now the required auth, but no auth is mocked)
    let result = client.try_pause();
    assert!(result.is_err());
}

#[test]
fn test_set_oracle_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle_old = Address::generate(&env);
    let oracle_new = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token_contract = create_token_contract(&env, &token_admin);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    token_contract.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token_contract.address, &oracle_old, &registry);

    // Book a session
    client.set_my_rate(&expert, &10_i128);
    let booking_id = client.book_session(&user, &expert, &100);

    // Rotate oracle to new address
    let result = client.try_set_oracle(&oracle_new);
    assert!(result.is_ok());

    // New oracle can finalize
    let result = client.try_finalize_session(&booking_id, &50);
    assert!(result.is_ok());
}

#[test]
fn test_non_admin_cannot_transfer_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin, &token, &oracle, &registry);

    // Clear auths so attacker has no authorization
    env.set_auths(&[]);

    let result = client.try_transfer_admin(&attacker);
    assert!(result.is_err());
}

#[test]
fn test_non_admin_cannot_set_oracle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin, &token, &oracle, &registry);

    env.set_auths(&[]);

    let result = client.try_set_oracle(&attacker);
    assert!(result.is_err());
}

// ==================== Expert Rate Tests ====================

#[test]
fn test_expert_can_set_and_update_rate() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);
    let token = Address::generate(&env);

    let client = create_client(&env);
    client.init(&admin, &token, &oracle, &registry);

    let res1 = client.try_set_my_rate(&expert, &10_i128);
    assert!(res1.is_ok());

    let res2 = client.try_set_my_rate(&expert, &25_i128);
    assert!(res2.is_ok());

    let res3 = client.try_set_my_rate(&expert, &0_i128);
    assert!(res3.is_err());
}

#[test]
fn test_book_session_calculates_correct_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let initial_balance = 5_000_i128;
    token.mint(&user, &initial_balance);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let stored_rate = 15_i128;
    client.set_my_rate(&expert, &stored_rate);

    let max_duration = 100_u64;
    let expected_deposit = stored_rate * (max_duration as i128);

    let _booking_id = client.book_session(&user, &expert, &max_duration);

    assert_eq!(token.balance(&user), initial_balance - expected_deposit);
    assert_eq!(token.balance(&client.address), expected_deposit);
}

#[test]
fn test_book_session_fails_if_expert_rate_not_set() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &5_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let max_duration = 100_u64;
    let res = client.try_book_session(&user, &expert, &max_duration);

    assert!(res.is_err());
}

#[test]
fn test_book_session_fails_if_expert_not_verified() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    // Set the mock registry to return false (expert not verified)
    mock_registry::MockRegistryClient::new(&env, &registry).set_verified(&false);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &5_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    // Set expert's rate
    client.set_my_rate(&expert, &10_i128);

    // Book session should fail with ExpertNotVerified error
    let max_duration = 100_u64;
    let res = client.try_book_session(&user, &expert, &max_duration);

    assert!(res.is_err());
}

// ==================== Pausability (Circuit Breaker) Tests ====================

#[test]
fn test_pause_blocks_book_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    client.set_my_rate(&expert, &10_i128);

    let result = client.try_pause();
    assert!(result.is_ok());

    let result = client.try_book_session(&user, &expert, &100);
    assert!(result.is_err());

    assert_eq!(token.balance(&user), 10_000);
}

#[test]
fn test_pause_blocks_finalize_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let booking_id = {
        client.set_my_rate(&expert, &10_i128);
        client.book_session(&user, &expert, &100)
    };

    client.pause();

    let result = client.try_finalize_session(&booking_id, &50);
    assert!(result.is_err());
}

#[test]
fn test_pause_blocks_reclaim_stale_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let booking_id = {
        client.set_my_rate(&expert, &10_i128);
        client.book_session(&user, &expert, &100)
    };

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 90_000);

    client.pause();

    let result = client.try_reclaim_stale_session(&user, &booking_id);
    assert!(result.is_err());
}

#[test]
fn test_pause_blocks_reject_session() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let booking_id = {
        client.set_my_rate(&expert, &10_i128);
        client.book_session(&user, &expert, &100)
    };

    client.pause();

    let result = client.try_reject_session(&expert, &booking_id);
    assert!(result.is_err());
}

#[test]
fn test_unpause_resumes_operations() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    client.set_my_rate(&expert, &10_i128);
    client.pause();

    let result = client.try_book_session(&user, &expert, &100);
    assert!(result.is_err());

    let result = client.try_unpause();
    assert!(result.is_ok());

    let booking_id = client.book_session(&user, &expert, &100);
    assert_eq!(booking_id, 1);
    assert_eq!(token.balance(&user), 9_000);
    assert_eq!(token.balance(&client.address), 1_000);
}

#[test]
fn test_read_only_functions_work_while_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let booking_id = {
        client.set_my_rate(&expert, &10_i128);
        client.book_session(&user, &expert, &100)
    };

    client.pause();

    let booking = client.get_booking(&booking_id);
    assert!(booking.is_some());
    assert_eq!(booking.unwrap().id, booking_id);

    // Paginated reads work while paused
    let user_bookings = client.get_user_bookings(&user, &0, &10);
    assert_eq!(user_bookings.len(), 1);

    let expert_bookings = client.get_expert_bookings(&expert, &0, &10);
    assert_eq!(expert_bookings.len(), 1);

    assert_eq!(client.get_user_booking_count(&user), 1);
    assert_eq!(client.get_expert_booking_count(&expert), 1);
}

// ==================== Scale & Pagination Tests ====================

/// Verifies that 50 bookings can be added to a single user without O(N) Vec growth.
/// Asserts count == 50, then uses pagination to fetch the first 10 and validates them.
#[test]
fn test_scale_50_bookings_single_user_with_pagination() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    // Mint enough tokens: rate=1, duration=1 per booking, 50 bookings = 50 tokens
    token.mint(&user, &50_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    // Expert sets rate
    let rate_per_second = 1_i128;
    let max_duration = 1_u64; // 1 token per booking to keep it cheap
    client.set_my_rate(&expert, &rate_per_second);

    // Book 50 sessions
    let mut booking_ids = std::vec::Vec::new();
    for _ in 0..50 {
        let id = client.book_session(&user, &expert, &max_duration);
        booking_ids.push(id);
    }

    // Assert count is correct — O(1) counter, no Vec load
    assert_eq!(client.get_user_booking_count(&user), 50);
    assert_eq!(client.get_expert_booking_count(&expert), 50);

    // Fetch first page: start=0, limit=10
    let page1 = client.get_user_bookings(&user, &0, &10);
    assert_eq!(page1.len(), 10);

    // Validate that each returned ID matches what was booked (IDs are 1-indexed globally)
    for i in 0..10u32 {
        let expected_id = booking_ids[i as usize];
        assert_eq!(page1.get(i).unwrap(), expected_id);
    }

    // Fetch second page: start=10, limit=10
    let page2 = client.get_user_bookings(&user, &10, &10);
    assert_eq!(page2.len(), 10);
    for i in 0..10u32 {
        let expected_id = booking_ids[(10 + i) as usize];
        assert_eq!(page2.get(i).unwrap(), expected_id);
    }

    // Fetch last page: start=45, limit=10 → should return 5 items
    let last_page = client.get_user_bookings(&user, &45, &10);
    assert_eq!(last_page.len(), 5);

    // Fetch out-of-range page: start=50, limit=10 → should return 0 items
    let empty_page = client.get_user_bookings(&user, &50, &10);
    assert_eq!(empty_page.len(), 0);
}

/// Verifies pagination is independent per user — two users with 25 bookings each
/// don't interfere with each other's indices.
#[test]
fn test_pagination_isolation_between_users() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user_a, &25_000);
    token.mint(&user_b, &25_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    client.set_my_rate(&expert, &1_i128);

    // 25 bookings for user_a then 25 for user_b (interleaved global booking IDs)
    for _ in 0..25 {
        client.book_session(&user_a, &expert, &1);
        client.book_session(&user_b, &expert, &1);
    }

    assert_eq!(client.get_user_booking_count(&user_a), 25);
    assert_eq!(client.get_user_booking_count(&user_b), 25);

    // Each user's first page should be 10 items, distinct from the other
    let page_a = client.get_user_bookings(&user_a, &0, &10);
    let page_b = client.get_user_bookings(&user_b, &0, &10);

    assert_eq!(page_a.len(), 10);
    assert_eq!(page_b.len(), 10);

    // user_a gets odd global IDs (1,3,5,...), user_b gets even (2,4,6,...)
    // Just assert they don't overlap
    for i in 0..10u32 {
        let id_a = page_a.get(i).unwrap();
        let id_b = page_b.get(i).unwrap();
        assert_ne!(
            id_a, id_b,
            "user_a and user_b share a booking_id — isolation broken"
        );
    }
}

// ==================== Booking Cancellation Tests (Issue #36) ====================

#[test]
fn test_user_cancels_before_session_starts_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let booking_id = {
        client.set_my_rate(&expert, &10_i128);
        client.book_session(&user, &expert, &100)
    };

    assert_eq!(token.balance(&user), 9_000);
    assert_eq!(token.balance(&client.address), 1_000);

    // User cancels immediately — session not yet started
    let result = client.try_cancel_booking(&user, &booking_id);
    assert!(result.is_ok());

    // Full refund returned to user
    assert_eq!(token.balance(&user), 10_000);
    assert_eq!(token.balance(&client.address), 0);

    let booking = client.get_booking(&booking_id).unwrap();
    use crate::types::BookingStatus;
    assert_eq!(booking.status, BookingStatus::Cancelled);
}

#[test]
fn test_user_cannot_cancel_after_oracle_marks_started() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    token.mint(&user, &10_000);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    let booking_id = {
        client.set_my_rate(&expert, &10_i128);
        client.book_session(&user, &expert, &100)
    };

    // Oracle marks session as started
    let result = client.try_mark_session_started(&booking_id);
    assert!(result.is_ok());

    // User tries to cancel — should fail because session has started
    let result = client.try_cancel_booking(&user, &booking_id);
    assert!(result.is_err());

    // Funds remain locked in contract
    assert_eq!(token.balance(&client.address), 1_000);
    assert_eq!(token.balance(&user), 9_000);
}

// ==================== Dynamic Precision Tests (Issue #38) ====================

#[test]
fn test_booking_with_18_decimal_token_scale_no_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    // Simulate a token with 18 decimals:
    // 1 token/second = 1_000_000_000_000_000_000 atomic units/second
    let rate_per_second: i128 = 1_000_000_000_000_000_000_i128; // 10^18
    let max_duration: u64 = 100; // 100 seconds
    let expected_deposit: i128 = rate_per_second * max_duration as i128; // 10^20 — well within i128

    // Mint enough tokens to cover deposit
    token.mint(&user, &expected_deposit);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);

    client.set_my_rate(&expert, &rate_per_second);
    let booking_id = client.book_session(&user, &expert, &max_duration);

    assert_eq!(token.balance(&user), 0);
    assert_eq!(token.balance(&client.address), expected_deposit);

    // Finalize for 50 seconds
    client.finalize_session(&booking_id, &50);

    let expert_pay = rate_per_second * 50_i128;
    let refund = expected_deposit - expert_pay;
    assert_eq!(token.balance(&expert), expert_pay);
    assert_eq!(token.balance(&user), refund);
    assert_eq!(token.balance(&client.address), 0);
}

/// Verifies expert pagination works correctly for 50 sessions.
#[test]
fn test_expert_pagination_50_bookings() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let expert = Address::generate(&env);
    let oracle = Address::generate(&env);
    let registry = create_mock_registry(&env);

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    let client = create_client(&env);
    client.init(&admin, &token.address, &oracle, &registry);
    client.set_my_rate(&expert, &1_i128);

    // 50 different users each book 1 session with the same expert
    for _ in 0..50 {
        let user = Address::generate(&env);
        token.mint(&user, &1);
        client.book_session(&user, &expert, &1);
    }

    assert_eq!(client.get_expert_booking_count(&expert), 50);

    let page = client.get_expert_bookings(&expert, &0, &10);
    assert_eq!(page.len(), 10);

    let tail = client.get_expert_bookings(&expert, &40, &20);
    assert_eq!(tail.len(), 10); // only 10 left from index 40 to 49
}
