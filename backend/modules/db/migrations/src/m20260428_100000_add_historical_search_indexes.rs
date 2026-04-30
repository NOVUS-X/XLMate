use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Index on `result` column for fast filtering of game outcomes
        manager
            .create_index(
                Index::create()
                    .name("idx_games_result")
                    .table((Smdb, Game::Table))
                    .col(Game::Result)
                    .to_owned(),
            )
            .await?;

        // 2. Composite index for head-to-head history: (white_player, black_player, created_at DESC)
        // This is extremely useful for searching games between two specific players.
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE INDEX "idx_games_head_to_head" ON "smdb"."game" ("white_player", "black_player", "created_at" DESC)"#
            )
            .await?;

        // 2b. Mirrored composite index for head-to-head history: (black_player, white_player, created_at DESC)
        // This ensures queries in both color directions are fully optimized.
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE INDEX "idx_games_head_to_head_mirrored" ON "smdb"."game" ("black_player", "white_player", "created_at" DESC)"#
            )
            .await?;

        // 3. Index on `variant` for variant-specific historical search
        manager
            .create_index(
                Index::create()
                    .name("idx_games_variant_search")
                    .table((Smdb, Game::Table))
                    .col(Game::Variant)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_games_result").table((Smdb, Game::Table)).to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "smdb"."idx_games_head_to_head""#)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(r#"DROP INDEX IF EXISTS "smdb"."idx_games_head_to_head_mirrored""#)
            .await?;

        manager
            .drop_index(Index::drop().name("idx_games_variant_search").table((Smdb, Game::Table)).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Game {
    Table,
    Result,
    WhitePlayer,
    BlackPlayer,
    Variant,
    CreatedAt,
}

#[derive(DeriveIden)]
struct Smdb;
