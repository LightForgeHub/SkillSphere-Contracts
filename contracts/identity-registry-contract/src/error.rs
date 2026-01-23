use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RegistryError {
    // Initialization Errors
    AlreadyInitialized = 1,
    NotInitialized = 2,

    // Authorization Errors
    AdminOnly = 3,

    // Logic Errors
    ExpertNotFound = 4,
    AlreadyVerified = 5,
    AlreadyBanned = 6,
    ExpertVecMax = 7,
}
