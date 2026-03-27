use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAuthorized = 3,
    BookingNotFound = 4,
    BookingNotPending = 5,
    InvalidAmount = 6,
    ReclaimTooEarly = 7,
    ContractPaused = 8,
    ExpertRateNotSet = 9,
    ExpertNotVerified = 10,
    SessionAlreadyStarted = 11,
    Overflow = 12,
}
