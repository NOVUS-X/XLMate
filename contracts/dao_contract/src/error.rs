use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DaoError {
    InvalidDaoConfiguration = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    Unauthorized = 4,
    ConfigNotFound = 5,
    MinThresholdNotMet = 6,
    ProposalNotFound = 7,
    AlreadyVotedForProposal = 8,
    ProposalVotingEnded = 9,
    ProposalVotingNotEnded = 10,
    UserBalanceIsEmpty = 11,
}
