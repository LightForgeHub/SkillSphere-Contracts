use soroban_sdk::{Address, Env, token};
use crate::storage::{self, Booking, BookingStatus};
use crate::error::VaultError;
use crate::events;

pub fn initialize_vault(
    env: &Env,
    admin: &Address,
    token: &Address,
    oracle: &Address
) -> Result<(), VaultError> {
    // 1. Check if already initialized
    if storage::has_admin(env) {
        return Err(VaultError::AlreadyInitialized);
    }

    // 2. Save State
    storage::set_admin(env, admin);
    storage::set_token(env, token);
    storage::set_oracle(env, oracle);

    Ok(())
}

pub fn create_booking(
    env: &Env,
    user: &Address,
    expert: &Address,
    rate: i128,
    booked_duration: u64,
) -> Result<u64, VaultError> {
    // Require authorization from the user creating the booking
    user.require_auth();

    // Validate rate
    if rate <= 0 {
        return Err(VaultError::InvalidAmount);
    }

    // Calculate total deposit
    let total_deposit = rate * (booked_duration as i128);

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
    let booking = Booking {
        id: booking_id,
        expert: expert.clone(),
        user: user.clone(),
        rate,
        total_deposit,
        booked_duration,
        status: BookingStatus::Pending,
    };

    // Save booking
    storage::save_booking(env, &booking);

    // Add booking to user and expert lists
    storage::add_booking_to_user_list(env, user, booking_id);
    storage::add_booking_to_expert_list(env, expert, booking_id);

    Ok(booking_id)
}

pub fn finalize_session(
    env: &Env,
    booking_id: u64,
    actual_duration: u64,
) -> Result<(), VaultError> {
    // 1. Require Oracle authorization
    let oracle = storage::get_oracle(env);
    oracle.require_auth();

    // 2. Get booking and verify it exists
    let booking = storage::get_booking(env, booking_id)
        .ok_or(VaultError::BookingNotFound)?;

    // 3. Verify booking is in Pending status
    if booking.status != BookingStatus::Pending {
        return Err(VaultError::BookingNotPending);
    }

    // 4. Calculate payments
    let expert_pay = booking.rate * (actual_duration as i128);
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