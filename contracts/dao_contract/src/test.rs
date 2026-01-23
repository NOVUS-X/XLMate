#![cfg(test)]
extern crate std;

use crate::{
    DaoConfig, DaoContract, DaoContractClient, DaoError, ProposalAction, Status, String,
    VoteAction, token,
};
use soroban_sdk::{
    Address, Env,
    testutils::{Address as _, Ledger},
};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(env, &sac.address()),
        token::StellarAssetClient::new(env, &sac.address()),
    )
}

pub struct TestDaoConfig<'a> {
    dao_config: DaoConfig,
    token_client: token::Client<'a>,
    token_admin_client: token::StellarAssetClient<'a>,
    dao_client: DaoContractClient<'a>,
}

fn create_dao_config<'a>(env: &Env) -> TestDaoConfig {
    let contract_id = env.register(DaoContract, ());
    let client = DaoContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    let dao_token = create_token_contract(&env, &admin);
    let dao_token_client = dao_token.0;
    let dao_token_admin_client = dao_token.1;

    let config = DaoConfig {
        dao_token: dao_token_client.address.clone(),
        min_threshold: 10,
        voting_period: 100,
        quorum: 30,
        protocol_fee: 1,
    };

    TestDaoConfig {
        dao_client: client,
        token_client: dao_token_client,
        token_admin_client: dao_token_admin_client,
        dao_config: config,
    }
}

#[test]
fn test_initialize_success() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client,
        token_admin_client: _,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let stored = dao_client.get_dao_config();

    assert_eq!(stored.min_threshold, 10);
    assert_eq!(stored.quorum, 30);
    assert_eq!(stored.protocol_fee, 1);
    assert_eq!(stored.dao_token, token_client.address)
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client: _,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let result = dao_client.try_initialize(&dao_config);

    assert_eq!(result, Err(Ok(DaoError::AlreadyInitialized)));
}

#[test]
fn test_initialize_with_invalid_configuraion_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DaoContract, ());
    let dao_client = DaoContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let dao_token = create_token_contract(&env, &admin);
    let dao_token_client = dao_token.0;

    let dao_config = DaoConfig {
        dao_token: dao_token_client.address.clone(),
        min_threshold: 100,
        voting_period: 100,
        quorum: 0,
        protocol_fee: 1,
    };

    let result = dao_client.try_initialize(&dao_config);

    assert_eq!(result, Err(Ok(DaoError::InvalidDaoConfiguration)));
}

#[test]
fn test_create_proposal_success() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint min balance to proposer address
    token_admin_client.mint(&proposer, &100);

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let proposal = dao_client.get_proposal(&0);
    assert_eq!(proposal.proposer, proposer);
    assert_eq!(proposal.status, Status::Open);
    assert_eq!(proposal.votes_abstain, 0);
    assert_eq!(proposal.votes_for, 0);
    assert_eq!(proposal.votes_against, 0);
}

#[test]
fn test_create_proposal_fails_if_threshold_not_met() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client: _,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    let result = dao_client.try_create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    assert_eq!(result, Err(Ok(DaoError::MinThresholdNotMet)));
}

#[test]
fn test_create_proposal_fails_if_dao_is_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config: _,
        token_client: _,
        token_admin_client: _,
    } = create_dao_config(&env);

    let proposer = Address::generate(&env);

    let result = dao_client.try_create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    assert_eq!(result, Err(Ok(DaoError::NotInitialized)));
}

#[test]
fn test_create_multiple_proposal_success() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint min balance to proposer address
    token_admin_client.mint(&proposer, &100);

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let proposal_id = dao_client.get_proposal_count();

    let proposal = dao_client.get_proposal(&(proposal_id - 1));
    assert_eq!(proposal.proposer, proposer);
    assert_eq!(proposal.status, Status::Open);
    assert_eq!(proposal.action, ProposalAction::UpdateFee(5));

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update min threshold"),
        &ProposalAction::UpdateMinThreshold(5000),
    );

    let proposal_id = dao_client.get_proposal_count();

    assert_eq!(proposal_id, 2);

    let proposal = dao_client.get_proposal(&(proposal_id - 1));
    assert_eq!(proposal.proposer, proposer);
    assert_eq!(proposal.status, Status::Open);
    assert_eq!(proposal.action, ProposalAction::UpdateMinThreshold(5000));
}

#[test]
fn test_vote_for_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint min balance to proposer address
    token_admin_client.mint(&proposer, &100);

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let voter = Address::generate(&env);

    // Mint tokens to voter
    token_admin_client.mint(&voter, &50);

    dao_client.vote_proposal(&0, &voter, &VoteAction::For);

    let proposal = dao_client.get_proposal(&0);
    assert_eq!(proposal.votes_for, 50);
}

#[test]
fn test_vote_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint min balance to proposer address
    token_admin_client.mint(&proposer, &100);

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let voter = Address::generate(&env);

    // Mint tokens to voter
    token_admin_client.mint(&voter, &50);

    dao_client.vote_proposal(&0, &voter, &VoteAction::For);

    let result = dao_client.try_vote_proposal(&0, &voter, &VoteAction::Abstain);

    assert_eq!(result, Err(Ok(DaoError::AlreadyVotedForProposal)));
}

#[test]
fn test_vote_without_having_dao_token_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint min balance to proposer address
    token_admin_client.mint(&proposer, &100);

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let voter = Address::generate(&env);

    let result = dao_client.try_vote_proposal(&0, &voter, &VoteAction::Abstain);

    assert_eq!(result, Err(Ok(DaoError::UserBalanceIsEmpty)));
}

#[test]
fn test_vote_after_proposal_ends_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    // Mint min balance to proposer address
    token_admin_client.mint(&proposer, &100);

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let proposal = dao_client.get_proposal(&0);

    // Move ledger to future
    env.ledger().set_timestamp(proposal.end_date + 100);

    let result = dao_client.try_vote_proposal(&0, &voter, &VoteAction::Abstain);

    assert_eq!(result, Err(Ok(DaoError::ProposalVotingEnded)));
}

#[test]
fn test_execute_proposal_success() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint tokens
    token_admin_client.mint(&proposer, &100);

    // token_admin_client

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let proposal = dao_client.get_proposal(&0);

    let voter = Address::generate(&env);

    // Mint tokens to voter
    token_admin_client.mint(&voter, &50);

    dao_client.vote_proposal(&0, &voter, &VoteAction::For);

    // Move ledger to future
    env.ledger().set_timestamp(proposal.end_date + 100);

    // Old config
    assert_eq!(dao_config.protocol_fee, 1);

    dao_client.execute_proposal(&0, &proposer);

    let proposal = dao_client.get_proposal(&0);
    let new_config = dao_client.get_dao_config();

    // New config
    assert_eq!(new_config.protocol_fee, 5);

    assert_eq!(proposal.status, Status::Successful);
}

#[test]
fn test_execute_proposal_resolves_failed_if_votes_are_against() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint tokens
    token_admin_client.mint(&proposer, &100);

    // token_admin_client

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let proposal = dao_client.get_proposal(&0);

    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    // Mint tokens to voter
    token_admin_client.mint(&voter1, &50);
    token_admin_client.mint(&voter2, &100);

    dao_client.vote_proposal(&0, &voter1, &VoteAction::For);
    dao_client.vote_proposal(&0, &voter2, &VoteAction::Against);

    // Move ledger to future
    env.ledger().set_timestamp(proposal.end_date + 100);

    // Old config
    assert_eq!(dao_config.protocol_fee, 1);

    dao_client.execute_proposal(&0, &proposer);

    let proposal = dao_client.get_proposal(&0);
    let new_config = dao_client.get_dao_config();

    // New config still the same
    assert_eq!(new_config.protocol_fee, 1);

    assert_eq!(proposal.status, Status::Failed);
}

#[test]
fn test_execute_proposal_resolves_failed_if_no_votes_are_made() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint tokens
    token_admin_client.mint(&proposer, &100);

    // token_admin_client

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let proposal = dao_client.get_proposal(&0);

    // Move ledger to future
    env.ledger().set_timestamp(proposal.end_date + 100);

    // Old config
    assert_eq!(dao_config.protocol_fee, 1);

    dao_client.execute_proposal(&0, &proposer);

    let proposal = dao_client.get_proposal(&0);
    let new_config = dao_client.get_dao_config();

    // New config still the same
    assert_eq!(new_config.protocol_fee, 1);

    assert_eq!(proposal.status, Status::Failed);
}

#[test]
fn test_execute_proposal_before_end_date_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let TestDaoConfig {
        dao_client,
        dao_config,
        token_client: _,
        token_admin_client,
    } = create_dao_config(&env);

    dao_client.initialize(&dao_config);

    let proposer = Address::generate(&env);

    // Mint tokens
    token_admin_client.mint(&proposer, &100);

    dao_client.create_proposal(
        &proposer,
        &String::from_str(&env, "update fee"),
        &ProposalAction::UpdateFee(5),
    );

    let result = dao_client.try_execute_proposal(&0, &proposer);

    assert_eq!(result, Err(Ok(DaoError::ProposalVotingNotEnded)));
}
