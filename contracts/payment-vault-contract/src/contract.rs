use crate::error::VaultError;
use crate::events;
use crate::storage;
use crate::types::{BookingRecord, BookingStatus};
use soroban_sdk::{token, Address, Env, Symbol};

pub fn initialize_vault(
    env: &Env,
    admin: &Address,
    token: &Address,
    oracle: &Address,
    registry: &Address,
) -> Result<(), VaultError> {
    // 1. Check if already initialized
    if storage::has_admin(env) {
        return Err(VaultError::AlreadyInitialized);
    }

    // 2. Save State
    storage::set_admin(env, admin);
    storage::set_token(env, token);
    storage::set_oracle(env, oracle);
    storage::set_registry_address(env, registry);

    Ok(())
}

pub fn pause(env: &Env) -> Result<(), VaultError> {
    let admin = storage::get_admin(env).ok_or(VaultError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, true);
    events::contract_paused(env, true);
    Ok(())
}

pub fn unpause(env: &Env) -> Result<(), VaultError> {
    let admin = storage::get_admin(env).ok_or(VaultError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, false);
    events::contract_paused(env, false);
    Ok(())
}

pub fn set_my_rate(env: &Env, expert: &Address, rate_per_second: i128) -> Result<(), VaultError> {
    expert.require_auth();

    if rate_per_second <= 0 {
        return Err(VaultError::InvalidAmount);
    }

    storage::set_expert_rate(env, expert, rate_per_second);
    events::expert_rate_updated(env, expert, rate_per_second);

    Ok(())
}

pub fn book_session(
    env: &Env,
    user: &Address,
    expert: &Address,
    max_duration: u64,
) -> Result<u64, VaultError> {
    if storage::is_paused(env) {
        return Err(VaultError::ContractPaused);
    }

    // Require authorization from the user creating the booking
    user.require_auth();

    // Verify expert is verified via Identity Registry cross-contract call
    let registry_address = storage::get_registry_address(env).ok_or(VaultError::NotInitialized)?;
    let is_verified: bool = env.invoke_contract(
        &registry_address,
        &Symbol::new(env, "is_verified"),
        soroban_sdk::vec![env, expert.to_val()],
    );

    if !is_verified {
        return Err(VaultError::ExpertNotVerified);
    }

    // Fetch the expert's rate
    let rate_per_second =
        storage::get_expert_rate(env, expert).ok_or(VaultError::ExpertRateNotSet)?;

    // Validate rate
    if rate_per_second <= 0 {
        return Err(VaultError::InvalidAmount);
    }

    // Calculate total deposit.
    // rate_per_second must be expressed in atomic units of the payment token
    // (e.g., stroops for XLM with 7 decimals, or 10^18 base units for 18-decimal tokens).
    let total_deposit = rate_per_second
        .checked_mul(max_duration as i128)
        .ok_or(VaultError::Overflow)?;

    if total_deposit <= 0 {
        return Err(VaultError::InvalidAmount);
    }

    // Get the token contract
    let token_address = storage::get_token(env);
    let token_client = token::Client::new(env, &token_address);

    // Transfer tokens from user to this contract
    let contract_address = env.current_contract_address();
    token_client.transfer(user, &contract_address, &total_deposit);

    // Generate booking ID and create booking
    let booking_id = storage::get_next_booking_id(env);
    let booking = BookingRecord {
        id: booking_id,
        user: user.clone(),
        expert: expert.clone(),
        rate_per_second,
        max_duration,
        total_deposit,
        status: BookingStatus::Pending,
        created_at: env.ledger().timestamp(),
        started_at: None,
    };

    // Save booking
    storage::save_booking(env, &booking);

    // Add booking to user and expert lists
    storage::add_booking_to_user_list(env, user, booking_id);
    storage::add_booking_to_expert_list(env, expert, booking_id);

    // Emit event for booking creation
    events::booking_created(env, booking_id, user, expert, total_deposit);

    Ok(booking_id)
}

pub fn finalize_session(
    env: &Env,
    booking_id: u64,
    actual_duration: u64,
) -> Result<(), VaultError> {
    if storage::is_paused(env) {
        return Err(VaultError::ContractPaused);
    }

    // 1. Require Oracle authorization
    let oracle = storage::get_oracle(env);
    oracle.require_auth();

    // 2. Get booking and verify it exists
    let booking = storage::get_booking(env, booking_id).ok_or(VaultError::BookingNotFound)?;

    // 3. Verify booking is in Pending status
    if booking.status != BookingStatus::Pending {
        return Err(VaultError::BookingNotPending);
    }

    // 4. Calculate payments.
    // rate_per_second is stored in atomic units of the payment token, so this
    // multiplication is safe for any token precision as long as the product fits i128.
    let expert_pay = booking
        .rate_per_second
        .checked_mul(actual_duration as i128)
        .ok_or(VaultError::Overflow)?;
    let refund = booking.total_deposit - expert_pay;

    // Ensure calculations are valid
    if expert_pay < 0 || refund < 0 {
        return Err(VaultError::InvalidAmount);
    }

    // 5. Get token contract
    let token_address = storage::get_token(env);
    let token_client = token::Client::new(env, &token_address);
    let contract_address = env.current_contract_address();

    // 6. Execute transfers
    // Pay expert
    if expert_pay > 0 {
        token_client.transfer(&contract_address, &booking.expert, &expert_pay);
    }

    // Refund user
    if refund > 0 {
        token_client.transfer(&contract_address, &booking.user, &refund);
    }

    // 7. Update booking status to Complete
    storage::update_booking_status(env, booking_id, BookingStatus::Complete);

    // 8. Emit SessionFinalized event
    events::session_finalized(env, booking_id, actual_duration, expert_pay);

    Ok(())
}

/// 24 hours in seconds
const RECLAIM_TIMEOUT: u64 = 86400;

pub fn reclaim_stale_session(env: &Env, user: &Address, booking_id: u64) -> Result<(), VaultError> {
    if storage::is_paused(env) {
        return Err(VaultError::ContractPaused);
    }

    // 1. Require user authorization
    user.require_auth();

    // 2. Get booking and verify it exists
    let booking = storage::get_booking(env, booking_id).ok_or(VaultError::BookingNotFound)?;

    // 3. Verify the caller is the booking owner
    if booking.user != *user {
        return Err(VaultError::NotAuthorized);
    }

    // 4. Verify booking is in Pending status
    if booking.status != BookingStatus::Pending {
        return Err(VaultError::BookingNotPending);
    }

    // 5. Check if 24 hours have passed since booking creation
    let current_time = env.ledger().timestamp();
    if current_time <= booking.created_at + RECLAIM_TIMEOUT {
        return Err(VaultError::ReclaimTooEarly);
    }

    // 6. Transfer total_deposit back to user
    let token_address = storage::get_token(env);
    let token_client = token::Client::new(env, &token_address);
    let contract_address = env.current_contract_address();
    token_client.transfer(&contract_address, &booking.user, &booking.total_deposit);

    // 7. Update booking status to Reclaimed
    storage::update_booking_status(env, booking_id, BookingStatus::Reclaimed);

    // 8. Emit event
    events::session_reclaimed(env, booking_id, booking.total_deposit);

    Ok(())
}

/// Mark a session as started (Oracle-only).
/// Once started, the user can no longer cancel the booking.
pub fn mark_session_started(env: &Env, booking_id: u64) -> Result<(), VaultError> {
    if storage::is_paused(env) {
        return Err(VaultError::ContractPaused);
    }

    let oracle = storage::get_oracle(env);
    oracle.require_auth();

    let booking = storage::get_booking(env, booking_id).ok_or(VaultError::BookingNotFound)?;

    if booking.status != BookingStatus::Pending {
        return Err(VaultError::BookingNotPending);
    }

    let started_at = env.ledger().timestamp();
    storage::update_booking_started_at(env, booking_id, started_at);
    events::session_started(env, booking_id, started_at);

    Ok(())
}

/// Cancel a pending booking and receive a full refund (User-only).
/// Can only be cancelled if the Oracle has not yet marked it as started.
pub fn cancel_booking(env: &Env, user: &Address, booking_id: u64) -> Result<(), VaultError> {
    if storage::is_paused(env) {
        return Err(VaultError::ContractPaused);
    }

    user.require_auth();

    let booking = storage::get_booking(env, booking_id).ok_or(VaultError::BookingNotFound)?;

    if booking.user != *user {
        return Err(VaultError::NotAuthorized);
    }

    if booking.status != BookingStatus::Pending {
        return Err(VaultError::BookingNotPending);
    }

    if booking.started_at.is_some() {
        return Err(VaultError::SessionAlreadyStarted);
    }

    let token_address = storage::get_token(env);
    let token_client = token::Client::new(env, &token_address);
    let contract_address = env.current_contract_address();
    token_client.transfer(&contract_address, &booking.user, &booking.total_deposit);

    storage::update_booking_status(env, booking_id, BookingStatus::Cancelled);
    events::booking_cancelled(env, booking_id, booking.total_deposit);

    Ok(())
}

pub fn transfer_admin(env: &Env, new_admin: &Address) -> Result<(), VaultError> {
    let current_admin = storage::get_admin(env).ok_or(VaultError::NotInitialized)?;
    current_admin.require_auth();
    storage::set_admin(env, new_admin);
    events::admin_transferred(env, &current_admin, new_admin);
    Ok(())
}

pub fn set_oracle(env: &Env, new_oracle: &Address) -> Result<(), VaultError> {
    let admin = storage::get_admin(env).ok_or(VaultError::NotInitialized)?;
    admin.require_auth();
    let old_oracle = storage::get_oracle(env);
    storage::set_oracle(env, new_oracle);
    events::oracle_updated(env, &old_oracle, new_oracle);
    Ok(())
}

pub fn reject_session(env: &Env, expert: &Address, booking_id: u64) -> Result<(), VaultError> {
    if storage::is_paused(env) {
        return Err(VaultError::ContractPaused);
    }

    // 1. Require expert authorization
    expert.require_auth();

    // 2. Get booking and verify it exists
    let booking = storage::get_booking(env, booking_id).ok_or(VaultError::BookingNotFound)?;

    // 3. Verify the caller is the expert in the booking
    if booking.expert != *expert {
        return Err(VaultError::NotAuthorized);
    }

    // 4. Verify booking is in Pending status
    if booking.status != BookingStatus::Pending {
        return Err(VaultError::BookingNotPending);
    }

    // 5. Transfer total_deposit back to user
    let token_address = storage::get_token(env);
    let token_client = token::Client::new(env, &token_address);
    let contract_address = env.current_contract_address();
    token_client.transfer(&contract_address, &booking.user, &booking.total_deposit);

    // 6. Update booking status to Rejected
    storage::update_booking_status(env, booking_id, BookingStatus::Rejected);

    // 7. Emit event
    events::session_rejected(env, booking_id, "Expert declined session");

    Ok(())
}
