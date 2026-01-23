//! Database connections live here, with built-in mock mode for testing.
//! 
//! Missing DATABASE_URL? No problem - an in-memory SQLite spins up automatically.
//! For production, just set DATABASE_URL=postgres://user:pass@host:5432/dbname.

pub mod db {
    use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbBackend, Schema, SqlxSqliteConnector};
    use migration::{Migrator, MigratorTrait};
    use std::str::FromStr;
    use tokio::sync::OnceCell;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    // Singleton instance for Mock DB to allow shared state in memory
    static MOCK_DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

    /// Quick check: am running in mock mode?
    pub fn is_mock_mode() -> bool {
        dotenv::dotenv().ok();
        match std::env::var("DATABASE_URL") {
            Ok(url) => url.starts_with("mock://") || url.is_empty(),
            Err(_) => true, // No DATABASE_URL means mock mode
        }
    }

    /// Returns a database connection - real PostgreSQL or mock SQLite.
    /// 
    /// The mode is auto-detected from DATABASE_URL.
    /// Warning: Mock database vanishes on restart!
    pub async fn get_db() -> DatabaseConnection {
        dotenv::dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL").unwrap_or_default();
        
        // In-memory SQLite when no real database is configured
        if database_url.is_empty() || database_url.starts_with("mock://") {
            let db = MOCK_DB.get_or_init(|| async {
                #[cfg(debug_assertions)]
                eprintln!("⚠️  WARNING: Using MOCK DATABASE (in-memory SQLite)");
                #[cfg(debug_assertions)]
                eprintln!("⚠️  Data will NOT be persisted. Set DATABASE_URL for production.");
                
                // Use file-based SQLite for test persistence across runtimes
                // Use process ID in filename to ensure fresh DB per test run
                let db_filename = format!("test_{}.db", std::process::id());
                let opts = SqliteConnectOptions::new()
                    .filename(&db_filename)
                    .create_if_missing(true);

                let pool = SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_with(opts)
                    .await
                    .expect("Failed to create mock database pool");
                
                let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
                
                // AUTOMATIC MIGRATION: Initialize schema for tests (run once)
                Migrator::up(&db, None).await.expect("Failed to run migrations for mock DB");
                
                db
            }).await;
            
            db.clone()
        } else {
            // Real database connection for production
            let connect_options = ConnectOptions::new(&database_url);
            
            Database::connect(connect_options)
                .await
                .expect("Failed to connect to database. Check DATABASE_URL.")
        }
    }
}


