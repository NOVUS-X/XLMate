# DAO Governance Contract

This contract implements a **stake-weighted on-chain DAO governance system** using the Soroban SDK. It allows token holders to stake tokens, create proposals, vote on them, and execute approved changes to DAO configuration parameters in a decentralized and upgrade-safe way.

The design prioritizes:

* determinism and auditability
* explicit lifecycle transitions for proposals
* stake-weighted voting (1 token = 1 share = 1 vote)
* safe handling of edge cases (e.g. zero-vote proposals, quorum not met)

## Overview

1. **Initialization**

   * The DAO is initialized once with a `DaoConfig`.
   * Configuration parameters define governance rules.

2. **Staking**

   * Users must stake DAO tokens to participate in governance.
   * Shares are minted 1:1 with staked tokens.
   * Staked tokens determine voting power and proposal creation eligibility.
   * Users can unstake at any time to reclaim their tokens.

3. **Proposal Creation**

   * Users with staked shares meeting the minimum threshold can create proposals.
   * Each proposal specifies a concrete on-chain action.

4. **Voting**

   * Votes are weighted by user's staked shares.
   * Each address may vote once per proposal.

5. **Execution**

   * After the voting period ends, proposals can be executed.
   * Quorum is checked based on participation vs total staked.
   * Passing proposals update DAO configuration.
   * Proposals with no participation, insufficient quorum, or insufficient support are marked as failed.

## Storage Layout

```rust
pub enum DataKey {
    Initialized,
    Config,
    ProposalsCount,
    Proposals(u32),
    VoteRecord(u32, Address),
    Shares(Address),
    TotalStaked,
}
```

### Storage Keys Explained

* **Initialized**

  * Boolean marker preventing re-initialization.

* **Config**

  * Stores the active `DaoConfig` struct.

* **ProposalsCount**

  * Auto-incrementing counter used to assign proposal IDs.

* **Proposals(u32)**

  * Stores a `Proposal` by its unique ID.

* **VoteRecord(u32, Address)**

  * Tracks whether a specific address has voted on a proposal.
  * Enforces one-vote-per-user-per-proposal.

* **Shares(Address)**

  * Tracks user's staked shares (1:1 with staked tokens).

* **TotalStaked**

  * Tracks total tokens staked in the DAO (used for quorum calculation).

## DAO Configuration (`DaoConfig`)

```rust
pub struct DaoConfig {
    pub quorum: u32,
    pub voting_period: u64,
    pub protocol_fee: i128,
    pub dao_token: Address,
    pub min_threshold: i128,
}
```

### Field-by-Field Explanation

#### `quorum: u32`

* Represents the minimum percentage of total staked tokens that must participate in a proposal for it to be valid.

* A proposal is considered valid only if the total voting power that participated meets or exceeds the configured quorum threshold.

* This is evaluated using total participation across **all vote types** (`FOR`, `AGAINST`, and `ABSTAIN`) against total staked:

    ```rust
    quorum_met = total_votes * 100 >= quorum * total_staked
    ```

#### `voting_period: u64`

* Duration (in ledger timestamps / seconds) that a proposal remains open for voting.
* Calculated as:

  * `end_date = start_date + voting_period`

#### `protocol_fee: i128`

* Governance-controlled protocol fee value.
* Can represent basis points, flat fees, or other protocol-specific units.
* Updated via governance proposals only.

#### `dao_token: Address`

* Address of the Soroban token contract used for governance.
* Token must be staked to participate in governance.

#### `min_threshold: i128`

* Minimum staked shares required to create a proposal.
* Prevents spam and low-signal proposals.

## Contract Functions

### Initialization

```rust
fn initialize(env: Env, config: DaoConfig) -> Result<(), DaoError>
```

Initialize the DAO with configuration parameters.

### Staking

```rust
fn stake(env: Env, user: Address, amount: i128) -> Result<(), DaoError>
```

Stake tokens to receive shares (1:1 ratio).

```rust
fn unstake(env: Env, user: Address, amount: i128) -> Result<(), DaoError>
```

Unstake tokens by burning shares (1:1 ratio).

### Proposals

```rust
fn create_proposal(env: Env, proposer: Address, description: String, action: ProposalAction) -> Result<(), DaoError>
```

Create a new governance proposal. Requires shares >= `min_threshold`.

```rust
fn vote_proposal(env: Env, proposal_id: u32, user: Address, vote_action: VoteAction) -> Result<(), DaoError>
```

Cast a vote on an active proposal.

```rust
fn execute_proposal(env: Env, proposal_id: u32, user: Address) -> Result<(), DaoError>
```

Execute a proposal after voting period ends. Checks quorum and vote outcome.

#### Getters

```rust
fn get_dao_config(env: Env) -> Result<DaoConfig, DaoError>
fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, DaoError>
fn get_proposal_count(env: Env) -> Result<u32, DaoError>
fn check_user_vote_status(env: Env, proposal_id: u32, user: Address) -> Result<bool, DaoError>
fn get_user_shares(env: Env, user: Address) -> Result<i128, DaoError>
fn get_total_staked(env: Env) -> Result<i128, DaoError>
```

## Proposal Actions

```rust
pub enum ProposalAction {
    UpdateFee(i128),
    UpdateQuorum(u32),
    UpdateVotingPeriod(u64),
    UpdateDaoToken(Address),
    UpdateMinThreshold(i128),
}
```

Each proposal encodes **exactly one executable governance action**.

### Supported Actions

* **UpdateFee** - change the protocol fee
* **UpdateQuorum** - adjust governance quorum requirements
* **UpdateVotingPeriod** - modify voting duration
* **UpdateDaoToken** - migrate to a new governance token
* **UpdateMinThreshold** - update proposal creation threshold

All actions are executed atomically during proposal execution.

## Voting Model

```rust
pub enum VoteAction {
    For,
    Against,
    Abstain,
}
```

* Voting power is proportional to staked shares at vote time.
* Abstain votes count toward quorum participation but not approval.
* A voter may only vote once per proposal.

## Proposal Lifecycle

```rust
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
```

### Lifecycle States

```rust
pub enum Status {
    Open,
    Successful,
    Failed,
}
```

1. **Open** - voting is active
2. **Successful** - quorum met and votes_for > votes_against, action executed
3. **Failed** - quorum not met, votes_for <= votes_against, or no votes cast

> Proposals with **zero votes** or **insufficient quorum** are marked as failed when executed.

## Error Codes

| Code | Error                   | Description                                            |
| ---- | ----------------------- | ------------------------------------------------------ |
| 1    | InvalidDaoConfiguration | Invalid config parameters                              |
| 2    | AlreadyInitialized      | DAO already initialized                                |
| 3    | NotInitialized          | DAO not initialized                                    |
| 4    | Unauthorized            | Unauthorized action                                    |
| 5    | ConfigNotFound          | Configuration not found                                |
| 6    | MinThresholdNotMet      | User shares below min threshold                        |
| 7    | ProposalNotFound        | Proposal does not exist                                |
| 8    | AlreadyVotedForProposal | User already voted                                     |
| 9    | ProposalVotingEnded     | Voting period has ended                                |
| 10   | ProposalVotingNotEnded  | Voting period not ended                                |
| 11   | NoStakesFoundForUser    | User has no staked shares                              |
| 12   | ProposalAlreadyExecuted | Proposal already executed                              |
| 13   | InsufficientStake       | Not enough staked tokens to unstake                    |
| 14   | InvalidAmount           | Invalid stake/unstake amount (must be > 0)             |
| 15   | QuorumNotMet            | Reserved (quorum failures mark proposal as failed)     |
| 16   | NoStakesFoundInDao      | No tokens staked in DAO (required to create proposals) |
