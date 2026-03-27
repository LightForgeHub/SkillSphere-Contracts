#![no_std]
mod contract;
mod error;
mod events;
mod storage;
#[cfg(test)]
mod test;
mod types;

use crate::error::VaultError;
use crate::types::BookingRecord;
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct PaymentVaultContract;

#[contractimpl]
impl PaymentVaultContract {
    /// Initialize the vault with the Admin, the Payment Token, the Oracle (Backend), and the Identity Registry
    pub fn init(
        env: Env,
        admin: Address,
        token: Address,
        oracle: Address,
        registry: Address,
    ) -> Result<(), VaultError> {
        contract::initialize_vault(&env, &admin, &token, &oracle, &registry)
    }

    /// Pause the contract (Admin-only)
    /// Halts all state-changing operations in an emergency
    pub fn pause(env: Env) -> Result<(), VaultError> {
        contract::pause(&env)
    }

    /// Unpause the contract (Admin-only)
    /// Resumes normal contract operations
    pub fn unpause(env: Env) -> Result<(), VaultError> {
        contract::unpause(&env)
    }

    /// Transfer admin rights to a new address (Admin-only)
    /// Old admin instantly loses all privileges
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), VaultError> {
        contract::transfer_admin(&env, &new_admin)
    }

    /// Update the oracle address (Admin-only)
    /// Old oracle instantly loses authorization to finalize sessions
    pub fn set_oracle(env: Env, new_oracle: Address) -> Result<(), VaultError> {
        contract::set_oracle(&env, &new_oracle)
    }

    /// Set an expert's own rate per second.
    /// `rate_per_second` MUST be expressed in atomic units of the payment token
    /// (e.g., 1 XLM = 10_000_000 stroops; 1 18-decimal token = 10^18 base units).
    pub fn set_my_rate(env: Env, expert: Address, rate_per_second: i128) -> Result<(), VaultError> {
        contract::set_my_rate(&env, &expert, rate_per_second)
    }

    /// Book a session with an expert.
    /// User deposits tokens upfront based on rate_per_second * max_duration.
    /// Both `rate_per_second` and the resulting `total_deposit` are denominated in
    /// atomic units of the configured payment token to correctly handle any token precision.
    pub fn book_session(
        env: Env,
        user: Address,
        expert: Address,
        max_duration: u64,
    ) -> Result<u64, VaultError> {
        contract::book_session(&env, &user, &expert, max_duration)
    }

    /// Finalize a session (Oracle-only).
    /// Calculates payments based on actual duration and processes refunds.
    pub fn finalize_session(
        env: Env,
        booking_id: u64,
        actual_duration: u64,
    ) -> Result<(), VaultError> {
        contract::finalize_session(&env, booking_id, actual_duration)
    }

    /// Reclaim funds from a stale booking (User-only).
    /// Users can reclaim their deposit if the booking has been pending for more than 24 hours.
    pub fn reclaim_stale_session(
        env: Env,
        user: Address,
        booking_id: u64,
    ) -> Result<(), VaultError> {
        contract::reclaim_stale_session(&env, &user, booking_id)
    }

    /// Reject a pending session (Expert-only).
    /// Experts can reject a pending booking, instantly refunding the user.
    pub fn reject_session(env: Env, expert: Address, booking_id: u64) -> Result<(), VaultError> {
        contract::reject_session(&env, &expert, booking_id)
    }

    /// Mark a session as started (Oracle-only).
    /// Once called, the user can no longer cancel the booking.
    pub fn mark_session_started(env: Env, booking_id: u64) -> Result<(), VaultError> {
        contract::mark_session_started(&env, booking_id)
    }

    /// Cancel a pending booking and receive a full refund (User-only).
    /// Cancellation is only allowed if the Oracle has not yet marked the session as started.
    /// `rate_per_second` and `total_deposit` must always be expressed in atomic units
    /// of the payment token (e.g., stroops for XLM, or 10^18 base units for 18-decimal tokens).
    pub fn cancel_booking(env: Env, user: Address, booking_id: u64) -> Result<(), VaultError> {
        contract::cancel_booking(&env, &user, booking_id)
    }

    /// Get a paginated list of booking IDs for a specific user.
    /// `start_index` is 0-based. Returns at most `limit` booking IDs.
    pub fn get_user_bookings(env: Env, user: Address, start_index: u32, limit: u32) -> Vec<u64> {
        storage::get_user_bookings_paginated(&env, &user, start_index, limit)
    }

    /// Get the total number of bookings a user has ever made.
    pub fn get_user_booking_count(env: Env, user: Address) -> u32 {
        storage::get_user_booking_count(&env, &user)
    }

    /// Get a paginated list of booking IDs for a specific expert.
    /// `start_index` is 0-based. Returns at most `limit` booking IDs.
    pub fn get_expert_bookings(
        env: Env,
        expert: Address,
        start_index: u32,
        limit: u32,
    ) -> Vec<u64> {
        storage::get_expert_bookings_paginated(&env, &expert, start_index, limit)
    }

    /// Get the total number of bookings an expert has ever received.
    pub fn get_expert_booking_count(env: Env, expert: Address) -> u32 {
        storage::get_expert_booking_count(&env, &expert)
    }

    /// Get booking details by booking ID (read-only).
    pub fn get_booking(env: Env, booking_id: u64) -> Option<BookingRecord> {
        storage::get_booking(&env, booking_id)
    }
}
