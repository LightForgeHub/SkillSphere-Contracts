use soroban_sdk::{Address, Env, symbol_short};

/// Emitted when a new booking is created
pub fn booking_created(env: &Env, booking_id: u64, user: &Address, expert: &Address, deposit: i128) {
    let topics = (symbol_short!("booked"), booking_id);
    env.events().publish(topics, (user.clone(), expert.clone(), deposit));
}

/// Emitted when a session is finalized
pub fn session_finalized(env: &Env, booking_id: u64, actual_duration: u64, total_cost: i128) {
    let topics = (symbol_short!("finalized"), booking_id);
    env.events().publish(topics, (actual_duration, total_cost));
}

pub fn session_reclaimed(env: &Env, booking_id: u64, amount: i128) {
    let topics = (symbol_short!("reclaim"), booking_id);
    env.events().publish(topics, amount);
}
