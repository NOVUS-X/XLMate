use crate::{DaoConfig, Status};
use soroban_sdk::{Address, Env, String, contractevent};

#[contractevent(topics = ["XLMateDao", "Init"])]
struct InitEvent {
    config: DaoConfig,
    timestamp: u64,
}

#[contractevent(topics = ["XLMateDao", "NewProposal"])]
struct NewProposalEvent {
    proposal_id: u32,
    creator: Address,
    description: String,
    timestamp: u64,
}

#[contractevent(topics = ["XLMateDao", "NewVote"])]
struct NewVoteEvent {
    proposal_id: u32,
    voter: Address,
    timestamp: u64,
}

#[contractevent(topics = ["XLMateDao", "ProposalExecuted"])]
struct ProposalExecutedEvent {
    proposal_id: u32,
    status: Status,
    timestamp: u64,
}

#[contractevent(topics = ["XLMateDao", "Staked"])]
struct StakedEvent {
    user: Address,
    amount: i128,
    total_shares: i128,
    timestamp: u64,
}

#[contractevent(topics = ["XLMateDao", "Unstaked"])]
struct UnstakedEvent {
    user: Address,
    amount: i128,
    remaining_shares: i128,
    timestamp: u64,
}

pub fn emit_initialized(env: &Env, config: DaoConfig, timestamp: u64) {
    InitEvent { config, timestamp }.publish(env);
}

pub fn emit_proposal_created(
    env: &Env,
    proposal_id: u32,
    creator: &Address,
    description: String,
    timestamp: u64,
) {
    NewProposalEvent {
        proposal_id,
        creator: creator.clone(),
        description: description.clone(),
        timestamp,
    }
    .publish(env);
}

pub fn emit_voted(env: &Env, proposal_id: u32, voter: Address, timestamp: u64) {
    NewVoteEvent {
        proposal_id,
        voter: voter.clone(),
        timestamp,
    }
    .publish(env);
}

pub fn emit_proposal_executed(env: &Env, proposal_id: u32, status: Status, timestamp: u64) {
    ProposalExecutedEvent {
        proposal_id,
        status,
        timestamp,
    }
    .publish(env);
}

pub fn emit_staked(env: &Env, user: &Address, amount: i128, total_shares: i128, timestamp: u64) {
    StakedEvent {
        user: user.clone(),
        amount,
        total_shares,
        timestamp,
    }
    .publish(env);
}

pub fn emit_unstaked(
    env: &Env,
    user: &Address,
    amount: i128,
    remaining_shares: i128,
    timestamp: u64,
) {
    UnstakedEvent {
        user: user.clone(),
        amount,
        remaining_shares,
        timestamp,
    }
    .publish(env);
}
