use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpertReputation {
    pub total_sessions: u64,
    pub cumulative_score: u64,
    pub updated_at: u64,
}
