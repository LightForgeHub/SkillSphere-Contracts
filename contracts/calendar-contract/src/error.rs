use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CalendarError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    ContractPaused = 3,
}
