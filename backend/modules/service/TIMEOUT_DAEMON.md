# Game Timeout Daemon

A Tokio-based asynchronous daemon that periodically monitors active chess games and automatically resolves games where players have run out of time.

## Overview

The timeout daemon ensures that games don't remain active indefinitely when players abandon them or their clocks expire. It automatically:

- Scans for active games on a configurable interval
- Checks remaining time for each player using the `time_control` table
- Resolves games that have timed out with appropriate results
- Updates player Elo ratings based on timeout outcomes
- Logs all timeout resolutions for audit purposes

## Architecture

### Components

1. **Time Control Table** (`smdb.time_control`)
   - Stores clock state for each active game
   - Tracks remaining time, clock running state, and last move time
   - Enables efficient timeout detection via database queries

2. **Timeout Daemon Service** (`service::timeout_daemon`)
   - Tokio async task that runs periodically
   - Configurable check intervals and batch sizes
   - Atomic game resolution with rating updates

3. **Match Resolution** (integrates with existing `GameService`)
   - Uses existing `GameService::complete_game` for consistency
   - Leverages `RatingService` for Elo calculations
   - Ensures database transaction atomicity

### Database Schema

```sql
CREATE TABLE smdb.time_control (
    id UUID PRIMARY KEY,
    game_id UUID UNIQUE NOT NULL,
    initial_time BIGINT NOT NULL,      -- milliseconds
    increment BIGINT NOT NULL,         -- milliseconds
    delay BIGINT NOT NULL,            -- milliseconds
    white_remaining_time BIGINT NOT NULL, -- milliseconds
    black_remaining_time BIGINT NOT NULL, -- milliseconds
    white_clock_running BOOLEAN NOT NULL DEFAULT false,
    black_clock_running BOOLEAN NOT NULL DEFAULT false,
    last_move_time TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    FOREIGN KEY (game_id) REFERENCES smdb.game(id) ON DELETE CASCADE
);

-- Indexes for efficient timeout detection
CREATE INDEX idx_time_control_game_id ON smdb.time_control(game_id);
CREATE INDEX idx_time_control_last_move_time ON smdb.time_control(last_move_time);
CREATE INDEX idx_time_control_running_clocks ON smdb.time_control(white_clock_running, black_clock_running);
```

## Configuration

The daemon accepts configuration through the `TimeoutDaemonConfig` struct:

```rust
pub struct TimeoutDaemonConfig {
    pub check_interval_secs: u64,     // How often to check (default: 30s)
    pub batch_size: u64,               // Max games per batch (default: 100)
    pub idle_threshold_secs: u64,      // Idle threshold (default: 300s)
}
```

Environment variables can be used to override defaults:

```bash
TIMEOUT_CHECK_INTERVAL_SECS=30
TIMEOUT_BATCH_SIZE=100
TIMEOUT_IDLE_THRESHOLD_SECS=300
```

## Timeout Logic

### Timeout Detection

The daemon uses a two-pronged approach:

1. **Primary**: Query `time_control` table for actual clock states
   - Calculates real remaining time based on `last_move_time`
   - Accounts for running vs stopped clocks
   - Handles increment and delay logic

2. **Fallback**: Simple age-based timeout for games without time control records
   - Games older than 30 minutes are flagged for review
   - Uses simplified winner determination

### Resolution Rules

- **Single Player Timeout**: Opponent wins
- **Both Players Timeout**: Game is drawn
- **No Time Control Data**: Uses fallback logic (alternating winner based on game age)

### Rating Updates

Timeout games are treated like normal games for rating purposes:

- Winner gains Elo points based on opponent's rating
- Loser loses corresponding Elo points
- Draws result in minimal rating changes
- Uses standard K-factor (32) and rating bounds (100-3000)

## Integration

### Server Startup

The daemon starts automatically with the API server:

```rust
// In server.rs
let timeout_config = TimeoutDaemonConfig::default();
let timeout_daemon = Arc::new(GameTimeoutDaemon::new((**db).clone(), timeout_config));

if let Err(e) = timeout_daemon.start().await {
    eprintln!("Warning: Failed to start timeout daemon: {}", e);
} else {
    eprintln!("Game timeout daemon started successfully");
}
```

### Game Creation

When games are created, corresponding `time_control` records should be created:

```rust
// Example time control creation
sqlx::query!(
    r#"
    INSERT INTO smdb.time_control (
        id, game_id, initial_time, increment, delay,
        white_remaining_time, black_remaining_time,
        white_clock_running, black_clock_running,
        last_move_time, created_at, updated_at
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
    "#,
    // ... parameters
)
.execute(&db)
.await?;
```

## Testing

Comprehensive tests are provided in `service/tests/timeout_daemon_tests.rs`:

- **Basic timeout functionality**: Single player timeout
- **Double timeout**: Both players timeout simultaneously  
- **Active game preservation**: Games with time remaining are not affected
- **Configuration validation**: Default and custom configs
- **Daemon lifecycle**: Start/stop behavior and duplicate prevention

### Running Tests

```bash
cd backend/modules/service
cargo test timeout_daemon_tests -- --nocapture
```

## Monitoring & Logging

The daemon provides detailed logging:

```
INFO  Game timeout daemon started - checking every 30 seconds
INFO  Processed 3 timed out games
INFO  Game timeout resolved: Game abc-123. White timed out (remaining: 0ms). New ratings - White: 1484, Black: 1516
ERROR Error checking game timeouts: Database connection failed
```

### Health Check

The daemon status can be checked programmatically:

```rust
let is_running = daemon.is_running();
```

## Performance Considerations

- **Batch Processing**: Processes games in configurable batches to prevent memory issues
- **Database Indexing**: Optimized indexes on time_control table for fast queries
- **Async Operations**: Non-blocking database operations using Tokio
- **Configurable Frequency**: Check interval can be adjusted based on server load

## Troubleshooting

### Common Issues

1. **Daemon fails to start**
   - Check database connection
   - Verify time_control table exists (run migration)
   - Check database permissions

2. **Games not timing out**
   - Verify time_control records exist for games
   - Check that clocks are marked as running
   - Verify last_move_time is recent

3. **Ratings not updating**
   - Check RatingService logs
   - Verify database transactions are committing
   - Check player records exist

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug cargo run
```

## Future Enhancements

Potential improvements for production:

1. **Graceful Shutdown**: Proper cleanup on server shutdown
2. **Metrics Integration**: Prometheus metrics for timeout rates
3. **Dynamic Configuration**: Runtime config updates
4. **Time Control Variants**: Support for more complex time controls
5. **Notification System**: WebSocket notifications for timeout events

## Security Considerations

- **Database Access**: Daemon uses read/write access only to game-related tables
- **Rate Limiting**: Built-in batch processing prevents database overload
- **Audit Trail**: All timeout resolutions are logged
- **Transaction Safety**: Game completion is atomic with rating updates
