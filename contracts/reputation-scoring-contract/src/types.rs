use soroban_sdk::{contracttype, Address};

/// A single review left by a user for a completed booking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewRecord {
    pub booking_id: u64,
    pub reviewer: Address,
    pub expert: Address,
    pub score: u32,     // 1–5
    pub timestamp: u64, // ledger timestamp
}

/// Aggregate reputation stats for an expert
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpertStats {
    pub total_score: u64,
    pub review_count: u32,
}

/// Mirror of PaymentVault's BookingStatus for cross-contract deserialization
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum BookingStatus {
    Pending = 0,
    Complete = 1,
    Rejected = 2,
    Reclaimed = 3,
    Cancelled = 5,
}

/// Mirror of PaymentVault's BookingRecord for cross-contract deserialization
#[contracttype]
#[derive(Clone, Debug)]
pub struct BookingRecord {
    pub id: u64,
    pub user: Address,
    pub expert: Address,
    pub rate_per_second: i128,
    pub max_duration: u64,
    pub total_deposit: i128,
    pub status: BookingStatus,
    pub created_at: u64,
    pub started_at: Option<u64>,
}
