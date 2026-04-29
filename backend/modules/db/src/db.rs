pub mod db {
    use sea_orm::{ConnectOptions, Database, DatabaseConnection};

    pub async fn get_db() -> DatabaseConnection {
        dotenv::dotenv().ok();
        let db_url = match std::env::var("DATABASE_URL") {
            Ok(v) => v,
            Err(_) => {
                eprintln!("WARNING: DATABASE_URL is not defined, defaulting to in-memory sqlite for tests");
                "sqlite::memory:".to_string()
            }
        };

        let connect_options = ConnectOptions::new(db_url).to_owned();

        let db: DatabaseConnection =
            Database::connect(connect_options)
                .await
                .unwrap();

        db
    }
}
