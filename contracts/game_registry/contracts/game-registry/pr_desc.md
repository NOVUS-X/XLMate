# Description

Closes #96

## Changes proposed

### What were you told to do?

I was tasked with implementing the **Game Registry - Immutable Game Record** contract with the following requirements:

- Provide a verifiable on-chain history of all ranked games for decentralized trust.
- Implement a contract that the backend can call to "finalize" a game result.
- **Storage**: Map `GameId` -> `GameResult { winner, white, black, timestamp }`.
- **Authorization**: Only an authorized "Server" address can call `record_game`.
- **Events**: Emit a `GameFinalized` event for indexers.
- **Storage Type**: Use `Persistent` storage for permanent records.
- **Performance**: Optimize for gas (minimal data storage) and extend TTL as per Soroban best practices.
- **Testing**: Develop tests for authorized/unauthorized calls and double initialization.

### What did I do?

**Implemented Game Registry Contract**:

- Developed the `GameRegistry` contract using the Soroban SDK.
- Defined the `GameResult` struct and `DataKey` enum for organized state management.
- Implemented `record_game` with custom logic to prevent duplicate record entries.
- Added `get_game` to allow easy retrieval of historical game data.

**Robust Authorization & Governance**:

- Implemented `initialize` function to set initial Admin and Server addresses.
- Added `require_auth` checks to ensure only the authorized Server can record games and only the Admin can modify sensitive settings.
- Created `set_server` and `set_admin` functions for smooth governance transitions.

**Optimized Storage & Events**:

- Utilized `env.storage().persistent()` for long-term data retention.
- Implemented `extend_ttl` on game records to ensure they stay active in the ledger.
- Implemented event emission using `Symbol::new(&env, "GameFinalized")` with structured topics and data for efficient indexing.

**Comprehensive Testing**:

- Wrote unit tests covering the happy path (successful recording).
- Implemented `should_panic` tests for unauthorized server calls.
- Added tests for preventing double initialization and verifying admin governance functions.
- Verified that all tests pass and the contract builds correctly.

## Check List (Check all the applicable boxes)

🚨Please review the contribution guideline for this repository.

- [x] My code follows the code style of this project.
- [x] This PR does not contain plagiarized content.
- [x] The title and description of the PR is clear and explains the approach.
- [x] I am making a pull request against the dev branch (left side).
- [x] My commit messages styles matches our requested structure.
- [x] My code additions will fail neither code linting checks nor unit test.
- [x] I am only making changes to files I was requested to.

## Screenshots/Videos

(Tests passing output)

```bash
running 4 tests
test test::test_unauthorized_record - should panic ... ok
test test::test_double_initialize - should panic ... ok
test test::test_game_registry_success ... ok
test test::test_update_server_and_admin ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```
