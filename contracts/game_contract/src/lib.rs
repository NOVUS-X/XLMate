#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env,
};

// Constants
const REFUND_TIMEOUT_HOURS: u64 = 24; // 24 hours timeout for refunds
const SECONDS_PER_HOUR: u64 = 3600;

// Escrow state
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowState {
    Pending,  // Deposits received, waiting for resolution
    Resolved, // Game resolved, winner can claim
    Refunded, // Refund claimed due to timeout
}

// Escrow information for each game
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowInfo {
    pub player1: Address,
    pub player2: Option<Address>, // None until second player deposits
    pub token: Address,        // Token contract address (XLM/USDC)
    pub amount: i128,          // Amount per player (in token's smallest unit)
    pub state: EscrowState,
    pub created_at: u64,       // Ledger timestamp when escrow was created
    pub resolved_at: Option<u64>, // Ledger timestamp when resolved
    pub winner: Option<Address>,   // Winner address (if resolved)
}

// Oracle public key (set during contract initialization)
#[contracttype]
#[derive(Clone, Debug)]
pub struct OracleConfig {
    pub public_key: BytesN<32>, // Ed25519 public key (32 bytes)
}

// Storage keys
const ESCROW_KEY: &str = "ESCROW";
const ORACLE_KEY: &str = "ORACLE";
const REFUND_TIMEOUT_KEY: &str = "REFUND_TIMEOUT";

#[contract]
pub struct GameContract;

#[contractimpl]
impl GameContract {
    /// Initialize the contract with Oracle public key
    /// Can only be called once
    pub fn initialize(env: Env, oracle_public_key: BytesN<32>, refund_timeout_hours: Option<u64>) {
        // Check if already initialized
        if env.storage().instance().has(&symbol_short!(ORACLE_KEY)) {
            panic!("Contract already initialized");
        }

        let timeout = refund_timeout_hours.unwrap_or(REFUND_TIMEOUT_HOURS);
        
        env.storage().instance().set(
            &symbol_short!(ORACLE_KEY),
            &OracleConfig {
                public_key: oracle_public_key,
            },
        );
        
        env.storage()
            .instance()
            .set(&symbol_short!(REFUND_TIMEOUT_KEY), &timeout);
    }

    /// Deposit tokens for a game wager
    /// Both players must deposit the same amount of the same token

    pub fn deposit(
        env: Env,
        game_id: BytesN<32>,
        token_address: Address,
        amount: i128,
    ) -> bool {
        // Validate amount
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let caller = env.invoker();

        // Check if escrow already exists
        let escrow_key = (symbol_short!(ESCROW_KEY), game_id.clone());
        if env.storage().persistent().has(&escrow_key) {
            let mut escrow: EscrowInfo = env.storage().persistent().get(&escrow_key).unwrap();

            // Check if already resolved or refunded
            match escrow.state {
                EscrowState::Resolved | EscrowState::Refunded => {
                    panic!("Escrow already finalized");
                }
                EscrowState::Pending => {
                    // Second player depositing
                    if escrow.player1 == caller {
                        panic!("Player already deposited");
                    }
                    if escrow.player2.is_some() {
                        panic!("Both players already deposited");
                    }

                    // Validate same token and amount
                    if escrow.token != token_address {
                        panic!("Token mismatch");
                    }
                    if escrow.amount != amount {
                        panic!("Amount mismatch");
                    }

                    // Transfer tokens from player 2
                    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
                    token_client.transfer(&caller, &env.current_contract_address(), &amount);

                    // Update escrow with player 2
                    escrow.player2 = Some(caller);
                    env.storage().persistent().set(&escrow_key, &escrow);

                    return true;
                }
            }
        } else {
            // First player depositing - create new escrow
            // Transfer tokens from player 1
            let token_client = soroban_sdk::token::Client::new(&env, &token_address);
            token_client.transfer(&caller, &env.current_contract_address(), &amount);

            let escrow = EscrowInfo {
                player1: caller,
                player2: None, // Will be set when second player deposits
                token: token_address,
                amount,
                state: EscrowState::Pending,
                created_at: env.ledger().timestamp(),
                resolved_at: None,
                winner: None,
            };

            env.storage().persistent().set(&escrow_key, &escrow);
            return true;
        }

        false
    }

    /// Resolve the game and set the winner
    /// Can only be called by the Oracle with a valid signature
  
    pub fn resolve(
        env: Env,
        game_id: BytesN<32>,
        winner_address: Address,
        signature: BytesN<64>, // Ed25519 signature (64 bytes)
    ) -> bool {
        // Get Oracle config
        let oracle_config: OracleConfig = env
            .storage()
            .instance()
            .get(&symbol_short!(ORACLE_KEY))
            .unwrap_or_else(|| panic!("Contract not initialized"));

        // Get escrow
        let escrow_key = (symbol_short!(ESCROW_KEY), game_id.clone());
        let mut escrow: EscrowInfo = env
            .storage()
            .persistent()
            .get(&escrow_key)
            .unwrap_or_else(|| panic!("Escrow not found"));

    
        match escrow.state {
            EscrowState::Resolved | EscrowState::Refunded => {
                panic!("Escrow already finalized");
            }
            EscrowState::Pending => {
                // Verify both players deposited
                let player2 = escrow.player2.unwrap_or_else(|| {
                    panic!("Both players must deposit before resolution");
                });

                // Verify winner is one of the players
                if winner_address != escrow.player1 && winner_address != player2 {
                    panic!("Winner must be one of the players");
                }

                // Prepare message to verify: hash(game_id || winner_address)
                // Create deterministic 32-byte message by hashing the concatenation
                let game_id_bytes = game_id.to_array();
                
                // Get deterministic representation of winner address
                // Hash the XDR representation to get consistent 32 bytes
                let winner_xdr = winner_address.to_xdr(&env);
                let winner_hash = env.crypto().sha256(&BytesN::from_array(
                    &env,
                    &{
                        let mut arr = [0u8; 32];
                        let xdr_bytes = winner_xdr.to_array();
                        let len = xdr_bytes.len().min(32);
                        arr[..len].copy_from_slice(&xdr_bytes[..len]);
                        arr
                    },
                ));
                
      
                let mut message_preimage = [0u8; 64];
                message_preimage[..32].copy_from_slice(&game_id_bytes);
                message_preimage[32..].copy_from_slice(&winner_hash.to_array());
                
                let message_hash = env.crypto().sha256(&BytesN::from_array(&env, &message_preimage));

                // Verify signature against the message hash
                // Oracle must sign: sha256(game_id || sha256(winner_address_xdr))
                let valid = env
                    .crypto()
                    .ed25519_verify(
                        &oracle_config.public_key,
                        &message_hash,
                        &signature,
                    );

                if !valid {
                    panic!("Invalid Oracle signature");
                }

                // Update escrow state
                escrow.state = EscrowState::Resolved;
                escrow.winner = Some(winner_address.clone());
                escrow.resolved_at = Some(env.ledger().timestamp());
                env.storage().persistent().set(&escrow_key, &escrow);

                // Transfer winnings to winner (both deposits)
                let total_amount = escrow.amount * 2;
                let token_client = soroban_sdk::token::Client::new(&env, &escrow.token);
                token_client.transfer(
                    &env.current_contract_address(),
                    &winner_address,
                    &total_amount,
                );

                return true;
            }
        }

        false
    }

    /// Claim refund if game not resolved within timeout period
    /// Can be called by either player
   
    pub fn claim_refund(env: Env, game_id: BytesN<32>) -> bool {
        // Get refund timeout
        let timeout_hours: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!(REFUND_TIMEOUT_KEY))
            .unwrap_or(REFUND_TIMEOUT_HOURS);
        let timeout_seconds = timeout_hours * SECONDS_PER_HOUR;

        // Get escrow
        let escrow_key = (symbol_short!(ESCROW_KEY), game_id.clone());
        let mut escrow: EscrowInfo = env
            .storage()
            .persistent()
            .get(&escrow_key)
            .unwrap_or_else(|| panic!("Escrow not found"));

        // Check state
        match escrow.state {
            EscrowState::Resolved | EscrowState::Refunded => {
                panic!("Escrow already finalized");
            }
            EscrowState::Pending => {
                // Check if timeout has elapsed
                let current_time = env.ledger().timestamp();
                let elapsed = current_time - escrow.created_at;
                
                if elapsed < timeout_seconds {
                    panic!("Refund timeout not yet elapsed");
                }

                // Verify caller is one of the players
                let caller = env.invoker();
                if caller != escrow.player1 && caller != escrow.player2 {
                    panic!("Only players can claim refund");
                }

                // Update state
                escrow.state = EscrowState::Refunded;
                escrow.resolved_at = Some(current_time);
                env.storage().persistent().set(&escrow_key, &escrow);

                // Refund both players
                let token_client = soroban_sdk::token::Client::new(&env, &escrow.token);
                
                // Refund player 1
                token_client.transfer(
                    &env.current_contract_address(),
                    &escrow.player1,
                    &escrow.amount,
                );

                // Refund player 2 if they deposited
                if let Some(player2) = escrow.player2 {
                    token_client.transfer(
                        &env.current_contract_address(),
                        &player2,
                        &escrow.amount,
                    );
                }

                return true;
            }
        }

        false
    }

    /// Get escrow information for a game
    pub fn get_escrow(env: Env, game_id: BytesN<32>) -> Option<EscrowInfo> {
        let escrow_key = (symbol_short!(ESCROW_KEY), game_id);
        env.storage().persistent().get(&escrow_key)
    }

    /// Get Oracle public key
    pub fn get_oracle(env: Env) -> Option<BytesN<32>> {
        let oracle_config: Option<OracleConfig> =
            env.storage().instance().get(&symbol_short!(ORACLE_KEY));
        oracle_config.map(|config| config.public_key)
    }

    /// Get refund timeout in hours
    pub fn get_refund_timeout(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&symbol_short!(REFUND_TIMEOUT_KEY))
            .unwrap_or(REFUND_TIMEOUT_HOURS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        symbol, testutils::Address as _, BytesN, Env, testutils::Ledger,
    };

    fn create_test_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    fn create_test_address(env: &Env, seed: u32) -> Address {
        let mut bytes = [0u8; 32];
        bytes[..4].copy_from_slice(&seed.to_be_bytes());
        Address::from_contract_id(&env, &BytesN::from_array(env, &bytes))
    }

    #[test]
    fn test_initialize() {
        let env = create_test_env();
        let contract = GameContract;
        let oracle_key = BytesN::from_array(&env, &[1u8; 32]);

        contract.initialize(env.clone(), oracle_key.clone(), None);

        let stored_key = contract.get_oracle(env.clone()).unwrap();
        assert_eq!(stored_key, oracle_key);
        assert_eq!(contract.get_refund_timeout(env), REFUND_TIMEOUT_HOURS);
    }

    #[test]
    #[should_panic(expected = "Contract already initialized")]
    fn test_double_initialize() {
        let env = create_test_env();
        let contract = GameContract;
        let oracle_key = BytesN::from_array(&env, &[1u8; 32]);

        contract.initialize(env.clone(), oracle_key.clone(), None);
        contract.initialize(env, oracle_key, None);
    }

    #[test]
    fn test_initialize_custom_timeout() {
        let env = create_test_env();
        let contract = GameContract;
        let oracle_key = BytesN::from_array(&env, &[1u8; 32]);
        let custom_timeout = 48u64;

        contract.initialize(env.clone(), oracle_key, Some(custom_timeout));
        assert_eq!(contract.get_refund_timeout(env), custom_timeout);
    }

    #[test]
    fn test_get_escrow_nonexistent() {
        let env = create_test_env();
        let contract = GameContract;
        let game_id = BytesN::from_array(&env, &[1u8; 32]);

        let escrow = contract.get_escrow(env, game_id);
        assert!(escrow.is_none());
    }

}
