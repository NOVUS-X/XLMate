use soroban_sdk::{Address, String, contracttype};

pub const PRECISION: u32 = 100_000;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Initialized,
    Config,
    ProposalsCount,
    Proposals(u32),
    VoteRecord(u32, Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DaoConfig {
    pub quorum: u32, // minimum level of voter participation (FOR + AGAINST + ABSTAIN)
    pub voting_period: u64,
    pub protocol_fee: i128,
    pub dao_token: Address,
    pub min_threshold: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalAction {
    UpdateFee(i128),
    UpdateQuorum(u32),
    UpdateVotingPeriod(u64),
    UpdateDaoToken(Address),
    UpdateMinThreshold(i128),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VoteAction {
    For,
    Against,
    Abstain,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Open,
    Successful,
    Failed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub description: String,
    pub action: ProposalAction,
    pub votes_for: i128,
    pub votes_against: i128,
    pub votes_abstain: i128,
    pub start_date: u64,
    pub end_date: u64,
    pub status: Status,
}
