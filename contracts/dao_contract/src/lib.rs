#![no_std]

mod error;
mod events;
mod storage;

use soroban_sdk::{Address, Env, String, contract, contractimpl, token};

pub use error::DaoError;
pub use storage::{DaoConfig, DataKey, Proposal, ProposalAction, Status, VoteAction};

#[contract]
pub struct DaoContract;

#[contractimpl]
impl DaoContract {
    pub fn initialize(env: Env, config: DaoConfig) -> Result<(), DaoError> {
        let is_initialized = Self::_check_initialization(&env);

        if is_initialized {
            return Err(DaoError::AlreadyInitialized);
        }

        if config.min_threshold == 0 || config.voting_period == 0 || config.protocol_fee == 0 {
            return Err(DaoError::InvalidDaoConfiguration);
        }

        Self::_save_config(&env, &config);

        Self::_initialize(&env);

        events::emit_initialized(&env, config, env.ledger().timestamp());

        Ok(())
    }

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        description: String,
        action: ProposalAction,
    ) -> Result<(), DaoError> {
        let is_initialized = Self::_check_initialization(&env);

        if !is_initialized {
            return Err(DaoError::NotInitialized);
        }

        proposer.require_auth();

        let next_proposal_id = Self::_get_proposal_count(&env);

        let config = Self::_get_config(&env).ok_or(DaoError::ConfigNotFound)?;

        let is_valid = Self::_validate_proposal_action(&action, &config);

        if !is_valid {
            return Err(DaoError::InvalidDaoConfiguration);
        }

        let dao_token_client = token::Client::new(&env, &config.dao_token);

        let user_balance = dao_token_client.balance(&proposer);

        if user_balance < config.min_threshold {
            return Err(DaoError::MinThresholdNotMet);
        }

        let timestamp = env.ledger().timestamp();

        let proposal = Proposal {
            id: next_proposal_id,
            proposer: proposer.clone(),
            description: description.clone(),
            start_date: timestamp,
            end_date: timestamp + config.voting_period,
            action,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            status: Status::Open,
        };

        Self::_save_proposal(&env, next_proposal_id, &proposal);

        Self::_save_proposal_count(&env, next_proposal_id + 1);

        events::emit_proposal_created(&env, next_proposal_id, &proposer, description, timestamp);

        Ok(())
    }

    pub fn vote_proposal(
        env: Env,
        proposal_id: u32,
        user: Address,
        vote_action: VoteAction,
    ) -> Result<(), DaoError> {
        let is_initialized = Self::_check_initialization(&env);

        if !is_initialized {
            return Err(DaoError::NotInitialized);
        }

        user.require_auth();

        let mut proposal =
            Self::_get_proposal(&env, proposal_id).ok_or(DaoError::ProposalNotFound)?;

        let timestamp = env.ledger().timestamp();

        if proposal.end_date <= timestamp {
            return Err(DaoError::ProposalVotingEnded);
        }

        if Self::_check_vote_record(&env, proposal_id, &user) {
            return Err(DaoError::AlreadyVotedForProposal);
        }

        let config = Self::_get_config(&env).ok_or(DaoError::ConfigNotFound)?;

        let dao_token_client = token::Client::new(&env, &config.dao_token);

        let user_balance = dao_token_client.balance(&user);

        if user_balance == 0 {
            return Err(DaoError::UserBalanceIsEmpty);
        }

        match vote_action {
            VoteAction::Abstain => proposal.votes_abstain += user_balance,
            VoteAction::Against => proposal.votes_against += user_balance,
            VoteAction::For => proposal.votes_for += user_balance,
        }

        Self::_save_proposal(&env, proposal_id, &proposal);

        Self::_save_vote_record(&env, proposal_id, &user);

        events::emit_voted(&env, proposal_id, user, timestamp);

        Ok(())
    }

    pub fn execute_proposal(env: Env, proposal_id: u32, user: Address) -> Result<(), DaoError> {
        let is_initialized = Self::_check_initialization(&env);

        if !is_initialized {
            return Err(DaoError::NotInitialized);
        }

        user.require_auth();

        let mut proposal =
            Self::_get_proposal(&env, proposal_id).ok_or(DaoError::ProposalNotFound)?;

        let timestamp = env.ledger().timestamp();

        if proposal.end_date > timestamp {
            return Err(DaoError::ProposalVotingNotEnded);
        }

        if proposal.status != Status::Open {
            return Err(DaoError::ProposalAlreadyExecuted);
        }

        let mut config = Self::_get_config(&env).ok_or(DaoError::ConfigNotFound)?;

        let total_votes = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;

        if total_votes == 0 {
            proposal.status = Status::Failed;

            Self::_save_proposal(&env, proposal_id, &proposal);

            events::emit_proposal_executed(&env, proposal_id, proposal.status, timestamp);

            return Ok(());
        }

        if proposal.votes_for > proposal.votes_against {
            match proposal.action.clone() {
                ProposalAction::UpdateDaoToken(new_token) => config.dao_token = new_token,
                ProposalAction::UpdateFee(new_fee) => config.protocol_fee = new_fee,
                ProposalAction::UpdateMinThreshold(new_threshold) => {
                    config.min_threshold = new_threshold
                }
                ProposalAction::UpdateVotingPeriod(new_voting_period) => {
                    config.voting_period = new_voting_period
                }
            }

            proposal.status = Status::Successful
        } else {
            proposal.status = Status::Failed
        }

        Self::_save_proposal(&env, proposal_id, &proposal);

        Self::_save_config(&env, &config);

        events::emit_proposal_executed(&env, proposal_id, proposal.status, timestamp);

        Ok(())
    }

    pub fn get_dao_config(env: Env) -> Result<DaoConfig, DaoError> {
        let config = Self::_get_config(&env).ok_or(DaoError::ConfigNotFound)?;

        Ok(config)
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, DaoError> {
        let proposal = Self::_get_proposal(&env, proposal_id).ok_or(DaoError::ProposalNotFound)?;

        Ok(proposal)
    }

    pub fn check_user_vote_status(
        env: Env,
        proposal_id: u32,
        user: Address,
    ) -> Result<bool, DaoError> {
        Ok(Self::_check_vote_record(&env, proposal_id, &user))
    }

    pub fn get_proposal_count(env: Env) -> Result<u32, DaoError> {
        Ok(Self::_get_proposal_count(&env))
    }
}

impl DaoContract {
    fn _initialize(env: &Env) {
        env.storage()
            .instance()
            .set::<_, bool>(&DataKey::Initialized, &true);
    }

    fn _check_initialization(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    fn _get_config(env: &Env) -> Option<DaoConfig> {
        env.storage()
            .instance()
            .get::<_, DaoConfig>(&DataKey::Config)
    }

    fn _save_config(env: &Env, config: &DaoConfig) {
        env.storage()
            .instance()
            .set::<_, DaoConfig>(&DataKey::Config, &config);
    }

    fn _get_proposal(env: &Env, proposal_id: u32) -> Option<Proposal> {
        env.storage()
            .instance()
            .get::<_, Proposal>(&DataKey::Proposals(proposal_id))
    }

    fn _save_proposal(env: &Env, proposal_id: u32, proposal: &Proposal) {
        env.storage()
            .instance()
            .set::<_, Proposal>(&DataKey::Proposals(proposal_id), &proposal);
    }

    fn _get_proposal_count(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get::<_, u32>(&DataKey::ProposalsCount)
            .unwrap_or_default()
    }

    fn _save_proposal_count(env: &Env, proposal_count: u32) {
        env.storage()
            .instance()
            .set::<_, u32>(&DataKey::ProposalsCount, &proposal_count)
    }

    fn _save_vote_record(env: &Env, proposal_id: u32, user: &Address) {
        env.storage()
            .instance()
            .set::<_, bool>(&DataKey::VoteRecord(proposal_id, user.clone()), &true);
    }

    fn _check_vote_record(env: &Env, proposal_id: u32, user: &Address) -> bool {
        env.storage()
            .instance()
            .get::<_, bool>(&DataKey::VoteRecord(proposal_id.clone(), user.clone()))
            .unwrap_or_default()
    }

    fn _validate_proposal_action(action: &ProposalAction, config: &DaoConfig) -> bool {
        match action {
            ProposalAction::UpdateDaoToken(addr) => addr != &config.dao_token,
            ProposalAction::UpdateFee(fee) => fee > &0,
            ProposalAction::UpdateMinThreshold(threshold) => threshold > &0,
            ProposalAction::UpdateVotingPeriod(period) => period > &0,
        }
    }
}

mod test;
