#![no_std]

mod contract;
mod error;
mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use crate::error::VaultError;

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

    /// Create a booking for a consultation session
    /// User deposits tokens upfront based on rate * booked_duration
    pub fn create_booking(
        env: Env,
        user: Address,
        expert: Address,
        rate: i128,
        booked_duration: u64,
    ) -> Result<u64, VaultError> {
        contract::create_booking(&env, &user, &expert, rate, booked_duration)
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
}