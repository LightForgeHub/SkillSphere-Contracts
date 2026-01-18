use soroban_sdk::{contracttype, Address, Env, Symbol};
use crate::types::ExpertStatus;

// The Event Data Structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpertStatusChangedEvent {
    pub expert: Address,
    pub old_status: ExpertStatus,
    pub new_status: ExpertStatus,
    pub admin: Address,
}

// Helper function to emit the event
pub fn emit_status_change(
    env: &Env, 
    expert: Address, 
    old_status: ExpertStatus, 
    new_status: ExpertStatus, 
    admin: Address
) {
    let event = ExpertStatusChangedEvent {
        expert,
        old_status,
        new_status,
        admin,
    };
    
    // We publish with the topic "status_change" so indexers can find it easily
    env.events().publish((Symbol::new(env, "status_change"),), event);
}