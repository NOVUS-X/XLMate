use sea_orm_migration::{prelude::*, MigrationTrait};
use crate::m20250428_121011_create_players_table::Player;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_game_white_player")
                    .table(Game::Table)
                    .col(Game::WhitePlayer)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_game_black_player")
                    .table(Game::Table)
                    .col(Game::BlackPlayer)
                    .to_owned(),
            )
            .await?;
        // NOTE: GIN index creation removed because IndexType::Gin is not available in this scope/version
        // and causing compilation errors. It was also attempted via raw SQL in previous migration.
        /*
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_game_pgn")
                    .table(Game::Table)
                    .col((Game::Pgn, IndexType::Gin))
                    .to_owned(),
            )
            .await?;
        */
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_player_username")
                    .table(Player::Table)
                    .col(Player::Username)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_game_started_at")
                    .table(Game::Table)
                    .col(Game::StartedAt)
                    .to_owned(),
            )
            .await?;
        println!("Common indexes created successfully.");
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_game_white_player").table(Game::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_game_black_player").table(Game::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_game_pgn").table(Game::Table).to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_player_username")
                    .table(Player::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_game_started_at")
                    .table(Game::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
#[iden = "smdb"] // Specify the schema here
enum Game {
    #[iden = "game"] // Specify the table name here
    Table,
    Id,
    WhitePlayer,
    BlackPlayer,
    StartedAt,
    // Add other columns if needed for future migrations involving this table
}
