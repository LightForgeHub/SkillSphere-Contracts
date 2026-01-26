use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Oracle,
    Booking(u64), // Booking ID -> Booking
    BookingCounter, // Counter for generating unique booking IDs
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BookingStatus {
    Pending,
    Complete,
    Reclaimed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Booking {
    pub id: u64,
    pub expert: Address,
    pub user: Address,
    pub rate: i128,           // Payment per second
    pub total_deposit: i128,  // Total amount deposited by user
    pub booked_duration: u64, // Booked duration in seconds
    pub status: BookingStatus,
    pub created_at: u64,      // Ledger timestamp when booking was created
}

// --- Admin ---
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

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

// --- Booking Counter ---
pub fn get_next_booking_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::BookingCounter)
        .unwrap_or(0);
    let next = current + 1;
    env.storage().instance().set(&DataKey::BookingCounter, &next);
    next
}

// --- Bookings ---
pub fn save_booking(env: &Env, booking: &Booking) {
    env.storage()
        .persistent()
        .set(&DataKey::Booking(booking.id), booking);
}

pub fn get_booking(env: &Env, booking_id: u64) -> Option<Booking> {
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