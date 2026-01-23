use soroban_sdk::contracttype;

// 1. Expert Status Enum
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ExpertStatus {
    Unverified = 0,
    Verified = 1,
    Banned = 2,
}

// 2. Expert Record Struct
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpertRecord {
    pub status: ExpertStatus,
    pub updated_at: u64, // Ledger timestamp of the last change
}
