use crate::types::{BookingRecord, BookingStatus};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Oracle,
    RegistryAddress,        
    Booking(u64),            // Booking ID -> BookingRecord
    BookingCounter,          // Counter for generating unique booking IDs
    UserBookings(Address),   // User Address -> Vec<u64> of booking IDs
    ExpertBookings(Address), // Expert Address -> Vec<u64> of booking IDs
    IsPaused,                // Circuit breaker flag
    // ── Indexed User Booking List ──────────────────────────────────────────
    // Replaces the old Vec<u64> approach with O(1) per-write composite keys.
    UserBooking(Address, u32), // (user, index) -> booking_id
    UserBookingCount(Address), // user -> total count (u32)
    // ── Indexed Expert Booking List ────────────────────────────────────────
    ExpertBooking(Address, u32), // (expert, index) -> booking_id
    ExpertBookingCount(Address), // expert -> total count (u32)
    ExpertRate(Address),     // Expert Address -> rate per second (i128)
}

// --- Admin ---
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

#[allow(dead_code)]
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

// --- Token (USDC/XLM) ---
pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}

pub fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

// --- Oracle (Backend) ---
pub fn set_oracle(env: &Env, oracle: &Address) {
    env.storage().instance().set(&DataKey::Oracle, oracle);
}

pub fn get_oracle(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Oracle).unwrap()
}

// --- Registry (Identity) ---
pub fn set_registry_address(env: &Env, registry: &Address) {
    env.storage().instance().set(&DataKey::RegistryAddress, registry);
}

pub fn get_registry_address(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::RegistryAddress)
}

// --- Pause (Circuit Breaker) ---
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&DataKey::IsPaused, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::IsPaused)
        .unwrap_or(false)
}

// --- Booking Counter ---
pub fn get_next_booking_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::BookingCounter)
        .unwrap_or(0);
    let next = current + 1;
    env.storage()
        .instance()
        .set(&DataKey::BookingCounter, &next);
    next
}

// --- Bookings ---
pub fn save_booking(env: &Env, booking: &BookingRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Booking(booking.id), booking);
}

pub fn get_booking(env: &Env, booking_id: u64) -> Option<BookingRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Booking(booking_id))
}

pub fn update_booking_status(env: &Env, booking_id: u64, status: BookingStatus) {
    if let Some(mut booking) = get_booking(env, booking_id) {
        booking.status = status;
        save_booking(env, &booking);
    }
}

pub fn update_booking_started_at(env: &Env, booking_id: u64, started_at: u64) {
    if let Some(mut booking) = get_booking(env, booking_id) {
        booking.started_at = Some(started_at);
        save_booking(env, &booking);
    }
}

// --- User Booking List (O(1) indexed storage) ---

/// Returns how many bookings a user has booked in total.
pub fn get_user_booking_count(env: &Env, user: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::UserBookingCount(user.clone()))
        .unwrap_or(0u32)
}

/// Appends a booking_id to the user's list in O(1) — no Vec load/save.
pub fn add_booking_to_user_list(env: &Env, user: &Address, booking_id: u64) {
    let count = get_user_booking_count(env, user);
    // Store the new booking_id at slot `count` (0-indexed)
    env.storage()
        .persistent()
        .set(&DataKey::UserBooking(user.clone(), count), &booking_id);
    // Increment the counter
    env.storage()
        .persistent()
        .set(&DataKey::UserBookingCount(user.clone()), &(count + 1));
}

/// Returns a paginated slice of booking IDs for a user.
/// `start_index` is 0-based; returns at most `limit` items.
pub fn get_user_bookings_paginated(
    env: &Env,
    user: &Address,
    start_index: u32,
    limit: u32,
) -> soroban_sdk::Vec<u64> {
    let count = get_user_booking_count(env, user);
    let mut result = soroban_sdk::Vec::new(env);

    let end = (start_index + limit).min(count);
    let mut i = start_index;
    while i < end {
        if let Some(booking_id) = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::UserBooking(user.clone(), i))
        {
            result.push_back(booking_id);
        }
        i += 1;
    }

    result
}

// --- Expert Booking List (O(1) indexed storage) ---

/// Returns how many bookings an expert has in total.
pub fn get_expert_booking_count(env: &Env, expert: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::ExpertBookingCount(expert.clone()))
        .unwrap_or(0u32)
}

/// Appends a booking_id to the expert's list in O(1) — no Vec load/save.
pub fn add_booking_to_expert_list(env: &Env, expert: &Address, booking_id: u64) {
    let count = get_expert_booking_count(env, expert);
    env.storage()
        .persistent()
        .set(&DataKey::ExpertBooking(expert.clone(), count), &booking_id);
    env.storage()
        .persistent()
        .set(&DataKey::ExpertBookingCount(expert.clone()), &(count + 1));
}

/// Returns a paginated slice of booking IDs for an expert.
/// `start_index` is 0-based; returns at most `limit` items.
pub fn get_expert_bookings_paginated(
    env: &Env,
    expert: &Address,
    start_index: u32,
    limit: u32,
) -> soroban_sdk::Vec<u64> {
    let count = get_expert_booking_count(env, expert);
    let mut result = soroban_sdk::Vec::new(env);

    let end = (start_index + limit).min(count);
    let mut i = start_index;
    while i < end {
        if let Some(booking_id) = env
            .storage()
            .persistent()
            .get::<DataKey, u64>(&DataKey::ExpertBooking(expert.clone(), i))
        {
            result.push_back(booking_id);
        }
        i += 1;
    }

    result
}

// --- Expert Rates ---
pub fn set_expert_rate(env: &Env, expert: &Address, rate: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::ExpertRate(expert.clone()), &rate);
}

pub fn get_expert_rate(env: &Env, expert: &Address) -> Option<i128> {
    env.storage()
        .persistent()
        .get(&DataKey::ExpertRate(expert.clone()))
}
