use crate::error::ReputationError;
use crate::events;
use crate::storage;
use crate::types::{BookingRecord, BookingStatus, ExpertStats, ReviewRecord};
use soroban_sdk::{Address, BytesN, Env, IntoVal, Symbol};

pub fn initialize(
    env: &Env,
    admin: &Address,
    vault_address: &Address,
) -> Result<(), ReputationError> {
    if storage::has_admin(env) {
        return Err(ReputationError::AlreadyInitialized);
    }
    storage::set_admin(env, admin);
    storage::set_vault_address(env, vault_address);
    Ok(())
}

pub fn pause(env: &Env) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, true);
    events::contract_paused(env, true);
    Ok(())
}

pub fn unpause(env: &Env) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    storage::set_paused(env, false);
    events::contract_paused(env, false);
    Ok(())
}

pub fn transfer_admin(env: &Env, new_admin: &Address) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    if storage::is_paused(env) {
        return Err(ReputationError::ContractPaused);
    }
    storage::set_admin(env, new_admin);
    events::admin_transferred(env, &admin, new_admin);
    Ok(())
}

pub fn upgrade_contract(env: &Env, new_wasm_hash: BytesN<32>) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
    Ok(())
}

pub fn submit_review(
    env: &Env,
    reviewer: &Address,
    booking_id: u64,
    score: u32,
) -> Result<(), ReputationError> {
    storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;

    if storage::is_paused(env) {
        return Err(ReputationError::ContractPaused);
    }

    reviewer.require_auth();

    // Validate score 1–5
    if score < 1 || score > 5 {
        return Err(ReputationError::InvalidScore);
    }

    // Prevent duplicate reviews
    if storage::has_review(env, booking_id) {
        return Err(ReputationError::AlreadyReviewed);
    }

    // Cross-contract call to vault: get_booking
    let vault_address = storage::get_vault_address(env)
        .ok_or(ReputationError::NotInitialized)?;

    let booking: BookingRecord = env.invoke_contract(
        &vault_address,
        &Symbol::new(env, "get_booking"),
        soroban_sdk::vec![env, booking_id.into_val(env)],
    );

    // Check booking is Complete
    if booking.status != BookingStatus::Complete {
        return Err(ReputationError::BookingNotComplete);
    }

    // Verify reviewer is the booking user
    if *reviewer != booking.user {
        return Err(ReputationError::NotBookingUser);
    }

    // Store review
    let review = ReviewRecord {
        booking_id,
        reviewer: reviewer.clone(),
        expert: booking.expert.clone(),
        score,
        timestamp: env.ledger().timestamp(),
    };
    storage::set_review(env, booking_id, &review);

    // Update expert stats
    let mut stats = storage::get_expert_stats(env, &booking.expert);
    stats.total_score += score as u64;
    stats.review_count += 1;
    storage::set_expert_stats(env, &booking.expert, &stats);

    events::review_submitted(env, booking_id, reviewer, &booking.expert, score);

    Ok(())
}

pub fn get_review(env: &Env, booking_id: u64) -> Option<ReviewRecord> {
    storage::get_review(env, booking_id)
}

pub fn penalize_expert(
    env: &Env,
    expert: &Address,
    amount: u64,
) -> Result<(), ReputationError> {
    let admin = storage::get_admin(env).ok_or(ReputationError::NotInitialized)?;
    admin.require_auth();

    if storage::is_paused(env) {
        return Err(ReputationError::ContractPaused);
    }

    if amount == 0 {
        return Err(ReputationError::InvalidPenalty);
    }

    let mut stats = storage::get_expert_stats(env, expert);
    // Saturating subtraction to prevent underflow
    stats.total_score = stats.total_score.saturating_sub(amount);
    storage::set_expert_stats(env, expert, &stats);

    events::expert_penalized(env, expert, amount);

    Ok(())
}

pub fn get_expert_stats(env: &Env, expert: &Address) -> ExpertStats {
    storage::get_expert_stats(env, expert)
}
