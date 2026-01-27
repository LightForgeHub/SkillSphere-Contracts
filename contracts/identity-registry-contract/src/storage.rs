use crate::types::{ExpertRecord, ExpertStatus};
use soroban_sdk::{contracttype, Address, Env, String};

// 1. Data Keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Expert(Address),
}

// Constants for TTL (Time To Live)
// Stellar ledgers close approx every 5 seconds.
// 1 Year in seconds = 31,536,000
// 1 Year in ledgers = ~6,307,200 (approx)
//
// Soroban allows setting TTL logic relative to the current ledger.
// "Threshold": If remaining lifetime is less than this...
// "Extend": ...bump it up to this amount.

const LEDGERS_THRESHOLD: u32 = 1_000_000; // 2 months
const LEDGERS_EXTEND_TO: u32 = 6_300_000; // 1 year

// ... [Admin Helpers] ...

/// Check if the admin has been set
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// Set the admin address
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Get the admin address
pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

// ... [Expert Helpers] ...

/// Set the expert record with status, data_uri and timestamp
pub fn set_expert_record(env: &Env, expert: &Address, status: ExpertStatus, data_uri: String) {
    let key = DataKey::Expert(expert.clone());

    let record = ExpertRecord {
        status,
        updated_at: env.ledger().timestamp(),
        data_uri,
    };

    // 1. Save the data
    env.storage().persistent().set(&key, &record);

    // 2. Extend the TTL
    // This tells the network: "If this data is going to die in less than 2 months,
    // extend its life to 1 full year from now."
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGERS_THRESHOLD, LEDGERS_EXTEND_TO);
}

/// Get the expert record, extending TTL if exists
pub fn get_expert_record(env: &Env, expert: &Address) -> ExpertRecord {
    let key = DataKey::Expert(expert.clone());

    // We also bump TTL on reads
    // If an expert is being checked frequently, they should stay alive.
    if env.storage().persistent().has(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGERS_THRESHOLD, LEDGERS_EXTEND_TO);
    }

    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(ExpertRecord {
            status: ExpertStatus::Unverified,
            updated_at: 0,
            data_uri: String::from_str(env, ""),
        })
}

/// Get the expert status
pub fn get_expert_status(env: &Env, expert: &Address) -> ExpertStatus {
    get_expert_record(env, expert).status
}
