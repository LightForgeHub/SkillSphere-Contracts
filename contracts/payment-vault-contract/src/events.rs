use soroban_sdk::{Env, symbol_short};

pub fn session_finalized(env: &Env, booking_id: u64, actual_duration: u64, total_cost: i128) {
    let topics = (symbol_short!("finalized"), booking_id);
    env.events().publish(topics, (actual_duration, total_cost));
}
