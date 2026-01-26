use soroban_sdk::{contracttype, Address};

/// Status of a booking in the payment vault
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum BookingStatus {
    Pending = 0,
    Complete = 1,
    Reclaimed = 2,
}

/// Record of a consultation booking with deposit locked
#[contracttype]
#[derive(Clone, Debug)]
pub struct BookingRecord {
    pub id: u64,                    // Storage key identifier
    pub user: Address,              // User who created the booking
    pub expert: Address,            // Expert providing consultation
    pub rate_per_second: i128,      // Payment rate per second
    pub max_duration: u64,          // Maximum booked duration in seconds
    pub total_deposit: i128,        // Total deposit (rate_per_second * max_duration)
    pub status: BookingStatus,      // Current booking status
    pub created_at: u64,            // Ledger timestamp when booking was created
}
