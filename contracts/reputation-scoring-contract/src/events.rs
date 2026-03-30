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
pub fn expert_penalized(env: &Env, expert: &Address, penalty_points: u64, new_score: u64) {
    let topics = (symbol_short!("penalized"),);
    env.events().publish(
        topics,
        (expert.clone(), penalty_points, new_score),
    );
}
