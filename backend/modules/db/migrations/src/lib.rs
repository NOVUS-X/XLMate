pub use sea_orm_migration::prelude::*;

mod m20250428_121011_create_players_table;
mod m20250429_163843_create_games_table;
mod m20250429_192832_add_common_indexes;
mod m20250604_160341_create_games_and_moves;
mod m20260121_000001_create_refresh_tokens_table;
mod m20260124_000001_add_player_details;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250428_121011_create_players_table::Migration),
            Box::new(m20250429_163843_create_games_table::Migration),
            Box::new(m20250429_192832_add_common_indexes::Migration),
            Box::new(m20250604_160341_create_games_and_moves::Migration),
            Box::new(m20260121_000001_create_refresh_tokens_table::Migration),
            Box::new(m20260124_000001_add_player_details::Migration),
        ]
    }
}
