#[cfg(test)]
pub mod test_helpers {
    use sea_orm::{Database, DatabaseConnection};
    use std::env;

    pub async fn setup_test_db() -> DatabaseConnection {
        // Set up test environment
        dotenv::from_filename(".env.test").ok();
        
        // Use SQLite in-memory database for tests
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite::memory:".to_string());
        
        let db = Database::connect(&database_url).await
            .expect("Failed to connect to test database");
        
        // Run migrations if needed
        setup_schema(&db).await;
        
        db
    }

    async fn setup_schema(db: &DatabaseConnection) {
        // Create basic tables needed for tests
        // This is a simplified version - in a real app you'd run proper migrations
        let create_player_table = sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE IF NOT EXISTS player (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash BLOB NOT NULL,
                biography TEXT DEFAULT '',
                country TEXT DEFAULT '',
                flair TEXT DEFAULT '',
                real_name TEXT DEFAULT '',
                location TEXT,
                fide_rating INTEGER,
                elo_rating INTEGER NOT NULL DEFAULT 1200,
                social_links TEXT,
                is_enabled BOOLEAN NOT NULL DEFAULT TRUE
            )
            "#.to_string(),
        );
        
        let _ = db.execute(create_player_table).await;
    }

    pub async fn cleanup_test_db(db: &DatabaseConnection) {
        // Clean up test data
        let cleanup = sea_orm::Statement::from_string(
            db.get_database_backend(),
            "DELETE FROM player".to_string(),
        );
        let _ = db.execute(cleanup).await;
    }
}
