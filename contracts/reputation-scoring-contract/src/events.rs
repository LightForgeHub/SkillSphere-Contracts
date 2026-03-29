#![allow(deprecated)]
use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when the contract is paused or unpaused.
pub fn contract_paused(env: &Env, paused: bool) {
    let topics = (symbol_short!("paused"),);
    env.events().publish(topics, paused);
}

/// Emitted when admin authority is transferred to a new address.
pub fn admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    let topics = (symbol_short!("adm_xfer"),);
    env.events()
        .publish(topics, (old_admin.clone(), new_admin.clone()));
}

/// Emitted when a review is submitted.
pub fn review_submitted(env: &Env, booking_id: u64, reviewer: &Address, expert: &Address, score: u32) {
    let topics = (symbol_short!("review"),);
    env.events()
        .publish(topics, (booking_id, reviewer.clone(), expert.clone(), score));
}
