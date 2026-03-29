use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReputationError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    ContractPaused = 3,
    InvalidScore = 4,
    BookingNotComplete = 5,
    AlreadyReviewed = 6,
    NotBookingUser = 7,
    InvalidPenalty = 8,
}
