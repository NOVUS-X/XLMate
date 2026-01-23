# DAO Governance Contract

This contract implements a **token-weighted on-chain DAO governance system** using the Soroban SDK. It allows token holders to create proposals, vote on them, and execute approved changes to DAO configuration parameters in a decentralized and upgrade-safe way.

The design prioritizes:

* determinism and auditability
* explicit lifecycle transitions for proposals
* token-weighted voting
* safe handling of edge cases (e.g. zero-vote proposals)

## Overview

1. **Initialization**

   * The DAO is initialized once with a `DaoConfig`.
   * Configuration parameters define governance rules.

2. **Proposal Creation**

   * Token holders meeting a minimum threshold can create proposals.
   * Each proposal specifies a concrete on-chain action.

3. **Voting**

   * Votes are weighted by User token balance.
   * Each address may vote once per proposal.

4. **Execution**

   * After the voting period ends, proposals can be executed.
   * Passing proposals update DAO configuration.
   * Proposals with no participation or insufficient support fail safely.

## Precision Model

```rust
pub const PRECISION: u32 = 100_000;
```

Governance thresholds (such as quorum) are evaluated using fixed-point arithmetic. Percentages are scaled by `PRECISION` to avoid floating-point math.

Example:

* `quorum = 50` means **50%**
* internally evaluated as `50 * PRECISION`

## Storage Layout

```rust
pub enum DataKey {
    Initialized,
    Config,
    ProposalsCount,
    Proposals(u32),
    VoteRecord(u32, Address),
    Shares(Address),
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

  * Reserved for future extensions (e.g. internal share accounting).

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

* Represents the minimum percentage of total voting power that must participate in a proposal for it to be valid.

* A proposal is considered valid only if the total voting power that participated in the vote meets or exceeds the configured quorum threshold.

* This is evaluated using total participation across **all vote types** (`FOR`, `AGAINST`, and `ABSTAIN`) and is checked by:

    ```rust
    is_valid =
      (votes_for + votes_against + votes_abstain)
        * PRECISION
        / total_available_votes
      >= quorum * PRECISION
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
* Token balances determine:

  * proposal eligibility
  * vote weight

#### `min_threshold: i128`

* Minimum token balance required to create a proposal.
* Prevents spam and low-signal proposals.

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

* **UpdateFee** – change the protocol fee
* **UpdateQuorum** – adjust governance quorum requirements
* **UpdateVotingPeriod** – modify voting duration
* **UpdateDaoToken** – migrate to a new governance token
* **UpdateMinThreshold** – update proposal creation threshold

All actions are executed atomically during proposal execution.

## Voting Model

```rust
pub enum VoteAction {
    For,
    Against,
    Abstain,
}
```

* Voting power is proportional to DAO token balance at vote time.
* Abstain votes count toward participation but not approval.
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

1. **Open** – voting is active
2. **Successful** – votes for is greater and quorum(not in place now) met action executed
3. **Failed** – votes for is less than or quorum(not in place now) not met or no votes cast

> Proposals with **zero votes** are explicitly executable and always fail.
