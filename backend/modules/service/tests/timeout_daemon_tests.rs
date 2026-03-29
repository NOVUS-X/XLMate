use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::Utc;
use sea_orm::{
    Database, DatabaseConnection, EntityTrait, ActiveModelTrait, Set
};
use db_entity::{game, player, prelude::Game, prelude::Player};
use service::timeout_daemon::{GameTimeoutDaemon, TimeoutDaemonConfig};
use std::sync::Arc;

/// Integration tests for the game timeout daemon
#[cfg(test)]
mod timeout_daemon_tests {
    use super::*;

    /// Test database setup for timeout tests
    async fn setup_test_db() -> DatabaseConnection {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/xlmate_test".to_string());
        
        let db = Database::connect(&database_url).await
            .expect("Failed to connect to test database");

        // Clean up any existing test data
        cleanup_test_data(&db).await;
        
        db
    }

    async fn cleanup_test_data(db: &DatabaseConnection) {
        // Clean up games
        let _ = Game::delete_many().exec(db).await;
        // Clean up players  
        let _ = Player::delete_many().exec(db).await;
    }

    /// Create test players for timeout testing
    async fn create_test_players(db: &DatabaseConnection) -> (player::Model, player::Model) {
        let player1 = player::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set("timeout_test_white".to_string()),
            elo_rating: Set(1500),
            is_enabled: Set(true),
            ..Default::default()
        };
        let player1 = player1.insert(db).await.unwrap();

        let player2 = player::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set("timeout_test_black".to_string()),
            elo_rating: Set(1400),
            is_enabled: Set(true),
            ..Default::default()
        };
        let player2 = player2.insert(db).await.unwrap();

        (player1, player2)
    }

    /// Create a test game that should timeout
    async fn create_timeout_test_game(
        db: &DatabaseConnection,
        white_player: &player::Model,
        black_player: &player::Model,
        started_minutes_ago: i64,
    ) -> game::Model {
        let start_time = Utc::now() - chrono::Duration::minutes(started_minutes_ago);
        
        let game = game::ActiveModel {
            id: Set(Uuid::new_v4()),
            white_player: Set(white_player.id),
            black_player: Set(black_player.id),
            fen: Set("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()),
            pgn: Set(serde_json::json!({})),
            result: Set(None), // Game is still active
            variant: Set(db_entity::game::GameVariant::Standard),
            started_at: Set(start_time.into()),
            duration_sec: Set(600),
            created_at: Set(start_time.into()),
            updated_at: Set(start_time.into()),
            is_imported: Set(false),
            original_pgn: Set(None),
            ..Default::default()
        };
        
        game.insert(db).await.unwrap()
    }

    /// Create a time control record for a test game
    async fn create_time_control_record(
        db: &DatabaseConnection,
        game_id: Uuid,
        white_remaining_ms: i64,
        black_remaining_ms: i64,
        white_clock_running: bool,
        black_clock_running: bool,
        last_move_minutes_ago: i64,
    ) -> Result<(), sea_orm::DbErr> {
        let last_move_time = Utc::now() - chrono::Duration::minutes(last_move_minutes_ago);
        
        // Use sea_orm statement instead of sqlx macro
        use sea_orm::ConnectionTrait;
        let db_backend = db.get_database_backend();
        let sql = format!(
            "INSERT INTO smdb.time_control (id, game_id, initial_time, increment, delay, white_remaining_time, black_remaining_time, white_clock_running, black_clock_running, last_move_time, created_at, updated_at) VALUES ('{}', '{}', 600000, 0, 0, {}, {}, {}, {}, '{}', '{}', '{}')",
            Uuid::new_v4(), game_id, white_remaining_ms, black_remaining_ms, white_clock_running, black_clock_running, last_move_time, Utc::now(), Utc::now()
        );
        
        db.execute(sea_orm::Statement::from_string(db_backend, sql)).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_daemon_basic_functionality() {
        let db = setup_test_db().await;
        let db_arc = Arc::new(db);
        let (white_player, black_player) = create_test_players(&*db_arc).await;
        
        // Create a game that should timeout (started 35 minutes ago)
        let test_game = create_timeout_test_game(&*db_arc, &white_player, &black_player, 35).await;
        
        // Create time control record with white clock running and no time left
        create_time_control_record(
            &*db_arc,
            test_game.id,
            0,   // White has no time left
            300000, // Black has 5 minutes left
            true, // White clock is running
            false, // Black clock is not running
            5     // Last move was 5 minutes ago
        ).await.unwrap();

        // Configure daemon to check frequently
        let config = TimeoutDaemonConfig {
            check_interval_secs: 1,
            batch_size: 10,
            idle_threshold_secs: 1800, // 30 minutes
        };

        let daemon = GameTimeoutDaemon::new(db_arc.clone(), config);
        
        // Start the daemon
        daemon.start().await.unwrap();
        
        // Wait a few seconds for daemon to process
        sleep(Duration::from_secs(3)).await;
        
        // Stop the daemon
        daemon.stop();
        
        // Verify the game was completed
        let updated_game = Game::find_by_id(test_game.id)
            .one(&*db_arc)
            .await
            .unwrap()
            .expect("Game should still exist");
            
        // Game should have a result (timeout resolved)
        assert!(updated_game.result.is_some(), "Game should be completed after timeout");
        
        // Verify winner is black (white timed out)
        match updated_game.result.unwrap() {
            db_entity::game::ResultSide::BlackWins => {
                // Expected - white timed out, black wins
            }
            _ => panic!("Expected black to win due to white timeout"),
        }
        
        // Verify player ratings were updated
        let updated_white = Player::find_by_id(white_player.id)
            .one(&*db_arc)
            .await
            .unwrap()
            .expect("White player should still exist");
            
        let updated_black = Player::find_by_id(black_player.id)
            .one(&*db_arc)
            .await
            .unwrap()
            .expect("Black player should still exist");
            
        // White should lose rating points, black should gain
        assert!(updated_white.elo_rating < 1500, "White should lose rating points");
        assert!(updated_black.elo_rating > 1400, "Black should gain rating points");
        
        cleanup_test_data(&*db_arc).await;
    }

    #[tokio::test]
    async fn test_timeout_daemon_both_players_timeout() {
        let db = setup_test_db().await;
        let db_arc = Arc::new(db);
        let (white_player, black_player) = create_test_players(&*db_arc).await;
        
        // Create a game that should timeout
        let test_game = create_timeout_test_game(&*db_arc, &white_player, &black_player, 25).await;
        
        // Create time control record with both players having no time left
        create_time_control_record(
            &*db_arc,
            test_game.id,
            0,   // White has no time left
            0,   // Black has no time left
            true, // White clock is running
            true, // Black clock is running
            2     // Last move was 2 minutes ago
        ).await.unwrap();

        let config = TimeoutDaemonConfig {
            check_interval_secs: 1,
            batch_size: 10,
            idle_threshold_secs: 1800,
        };

        let daemon = GameTimeoutDaemon::new(db_arc.clone(), config);
        daemon.start().await.unwrap();
        
        sleep(Duration::from_secs(3)).await;
        daemon.stop();
        
        // Verify the game was completed as a draw
        let updated_game = Game::find_by_id(test_game.id)
            .one(&*db_arc)
            .await
            .unwrap()
            .expect("Game should still exist");
            
        assert!(updated_game.result.is_some(), "Game should be completed after timeout");
        
        // Should be a draw when both timeout
        match updated_game.result.unwrap() {
            db_entity::game::ResultSide::Draw => {
                // Expected - both timed out
            }
            _ => panic!("Expected draw when both players timeout"),
        }
        
        cleanup_test_data(&*db_arc).await;
    }

    #[tokio::test]
    async fn test_timeout_daemon_no_timeout_for_active_game() {
        let db = setup_test_db().await;
        let db_arc = Arc::new(db);
        let (white_player, black_player) = create_test_players(&*db_arc).await;
        
        // Create a recent game that should NOT timeout
        let test_game = create_timeout_test_game(&*db_arc, &white_player, &black_player, 5).await;
        
        // Create time control record with plenty of time left
        create_time_control_record(
            &*db_arc,
            test_game.id,
            300000,   // White has 5 minutes left
            300000,   // Black has 5 minutes left
            true,     // White clock is running
            false,    // Black clock is not running
            1         // Last move was 1 minute ago
        ).await.unwrap();

        let config = TimeoutDaemonConfig {
            check_interval_secs: 1,
            batch_size: 10,
            idle_threshold_secs: 1800,
        };

        let daemon = GameTimeoutDaemon::new(db_arc.clone(), config);
        daemon.start().await.unwrap();
        
        sleep(Duration::from_secs(3)).await;
        daemon.stop();
        
        // Verify the game is still active
        let updated_game = Game::find_by_id(test_game.id)
            .one(&*db_arc)
            .await
            .unwrap()
            .expect("Game should still exist");
            
        assert!(updated_game.result.is_none(), "Active game should not be completed");
        
        cleanup_test_data(&*db_arc).await;
    }

    #[tokio::test]
    async fn test_timeout_daemon_config_validation() {
        let db = setup_test_db().await;
        let db_arc = Arc::new(db);
        
        // Test default config
        let default_config = TimeoutDaemonConfig::default();
        assert_eq!(default_config.check_interval_secs, 30);
        assert_eq!(default_config.batch_size, 100);
        assert_eq!(default_config.idle_threshold_secs, 300);
        
        // Test custom config
        let custom_config = TimeoutDaemonConfig {
            check_interval_secs: 10,
            batch_size: 50,
            idle_threshold_secs: 600,
        };
        assert_eq!(custom_config.check_interval_secs, 10);
        assert_eq!(custom_config.batch_size, 50);
        assert_eq!(custom_config.idle_threshold_secs, 600);
        
        // Test daemon creation
        let daemon = GameTimeoutDaemon::new(db_arc.clone(), custom_config);
        assert!(!daemon.is_running());
        
        cleanup_test_data(&*db_arc).await;
    }

    #[tokio::test]
    async fn test_timeout_daemon_prevent_multiple_starts() {
        let db = setup_test_db().await;
        let db_arc = Arc::new(db);
        let config = TimeoutDaemonConfig::default();
        
        let daemon = GameTimeoutDaemon::new(db_arc.clone(), config);
        
        // First start should succeed
        assert!(daemon.start().await.is_ok());
        assert!(daemon.is_running());
        
        // Second start should fail
        assert!(daemon.start().await.is_err());
        assert!(daemon.is_running());
        
        // Stop should work
        daemon.stop();
        assert!(!daemon.is_running());
        
        cleanup_test_data(&*db_arc).await;
    }
}
