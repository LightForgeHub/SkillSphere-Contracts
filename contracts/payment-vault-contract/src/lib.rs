#![no_std]

mod contract;
mod error;
mod events;
mod storage;
mod types;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, Vec};
use crate::error::VaultError;
use crate::types::BookingRecord;

#[contract]
pub struct PaymentVaultContract;

#[contractimpl]
impl PaymentVaultContract {
    /// Initialize the vault with the Admin, the Payment Token, and the Oracle (Backend)
    pub fn init(
        env: Env,
        admin: Address,
        token: Address,
        oracle: Address
    ) -> Result<(), VaultError> {
        contract::initialize_vault(&env, &admin, &token, &oracle)
    }

    /// Book a session with an expert
    /// User deposits tokens upfront based on rate_per_second * max_duration
    pub fn book_session(
        env: Env,
        user: Address,
        expert: Address,
        rate_per_second: i128,
        max_duration: u64,
    ) -> Result<u64, VaultError> {
        contract::book_session(&env, &user, &expert, rate_per_second, max_duration)
    }

    /// Finalize a session (Oracle-only)
    /// Calculates payments based on actual duration and processes refunds
    pub fn finalize_session(
        env: Env,
        booking_id: u64,
        actual_duration: u64,
    ) -> Result<(), VaultError> {
        contract::finalize_session(&env, booking_id, actual_duration)
    }

    /// Reclaim funds from a stale booking (User-only)
    /// Users can reclaim their deposit if the booking has been pending for more than 24 hours
    pub fn reclaim_stale_session(
        env: Env,
        user: Address,
        booking_id: u64,
    ) -> Result<(), VaultError> {
        contract::reclaim_stale_session(&env, &user, booking_id)
    }

    /// Get all booking IDs for a specific user
    pub fn get_user_bookings(env: Env, user: Address) -> Vec<u64> {
        storage::get_user_bookings(&env, &user)
    }

    /// Get all booking IDs for a specific expert
    pub fn get_expert_bookings(env: Env, expert: Address) -> Vec<u64> {
        storage::get_expert_bookings(&env, &expert)
    }

    /// Get booking details by booking ID (read-only)
    pub fn get_booking(env: Env, booking_id: u64) -> Option<BookingRecord> {
        storage::get_booking(&env, booking_id)
    }
}
