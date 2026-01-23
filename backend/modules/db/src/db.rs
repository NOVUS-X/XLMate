//! Database connections live here, with built-in mock mode for testing.
//! 
//! Missing DATABASE_URL? No problem - an in-memory SQLite spins up automatically.
//! For production, just set DATABASE_URL=postgres://user:pass@host:5432/dbname.

pub mod db {
    use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbBackend, Schema};
    use migration::{Migrator, MigratorTrait};

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
            #[cfg(debug_assertions)]
            eprintln!("⚠️  WARNING: Using MOCK DATABASE (in-memory SQLite)");
            #[cfg(debug_assertions)]
            eprintln!("⚠️  Data will NOT be persisted. Set DATABASE_URL for production.");
            
            let mock_url = "sqlite::memory:";
            let connect_options = ConnectOptions::new(mock_url);
            
            let db = Database::connect(connect_options)
                .await
                .expect("Failed to create mock database");
            
            // AUTOMATIC MIGRATION: Initialize schema for tests
            Migrator::up(&db, None).await.expect("Failed to run migrations for mock DB");
            
            db
        } else {
            // Real database connection for production
            let connect_options = ConnectOptions::new(&database_url);
            
            Database::connect(connect_options)
                .await
                .expect("Failed to connect to database. Check DATABASE_URL.")
        }
    }
}


