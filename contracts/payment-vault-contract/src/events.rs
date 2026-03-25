#![allow(deprecated)]
use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a new booking is created
pub fn booking_created(
    env: &Env,
    booking_id: u64,
    user: &Address,
    expert: &Address,
    deposit: i128,
) {
    let topics = (symbol_short!("booked"), booking_id);
    env.events()
        .publish(topics, (user.clone(), expert.clone(), deposit));
}

/// Emitted when a session is finalized
pub fn session_finalized(env: &Env, booking_id: u64, actual_duration: u64, expert_pay: i128, fee_amount: i128) {
    let topics = (symbol_short!("finalized"), booking_id);
    env.events().publish(topics, (actual_duration, expert_pay, fee_amount));
}

pub fn session_reclaimed(env: &Env, booking_id: u64, amount: i128) {
    let topics = (symbol_short!("reclaim"), booking_id);
    env.events().publish(topics, amount);
}

/// Emitted when the contract is paused or unpaused
pub fn contract_paused(env: &Env, paused: bool) {
    let topics = (symbol_short!("paused"),);
    env.events().publish(topics, paused);
}

/// Emitted when an expert rejects a pending session
pub fn session_rejected(env: &Env, booking_id: u64, reason: &str) {
    let topics = (symbol_short!("reject"), booking_id);
    env.events().publish(topics, reason);
}

/// Emitted when an expert updates their rate
pub fn expert_rate_updated(env: &Env, expert: &Address, rate: i128) {
    let topics = (symbol_short!("rate_upd"), expert.clone());
    env.events().publish(topics, rate);
}
