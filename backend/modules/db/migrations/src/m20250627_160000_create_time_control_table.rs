use sea_orm_migration::{prelude::*, schema::*, prelude::extension::postgres::Type};
use super::m20250429_163843_create_games_table::Game;
use sea_orm_migration::prelude::ForeignKeyAction;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Ensure the schema exists
        manager
            .get_connection()
            .execute_unprepared("CREATE SCHEMA IF NOT EXISTS \"smdb\"")
            .await?;

        // Create the time_control table within the smdb schema
        manager
            .create_table(
                Table::create()
                    .table((Smdb, TimeControl::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TimeControl::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TimeControl::GameId).uuid().not_null().unique())
                    .col(ColumnDef::new(TimeControl::InitialTime).big_integer().not_null()) // milliseconds
                    .col(ColumnDef::new(TimeControl::Increment).big_integer().not_null()) // milliseconds
                    .col(ColumnDef::new(TimeControl::Delay).big_integer().not_null()) // milliseconds
                    .col(ColumnDef::new(TimeControl::WhiteRemainingTime).big_integer().not_null()) // milliseconds
                    .col(ColumnDef::new(TimeControl::BlackRemainingTime).big_integer().not_null()) // milliseconds
                    .col(ColumnDef::new(TimeControl::WhiteClockRunning).boolean().not_null().default(false))
                    .col(ColumnDef::new(TimeControl::BlackClockRunning).boolean().not_null().default(false))
                    .col(ColumnDef::new(TimeControl::LastMoveTime).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(TimeControl::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TimeControl::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_time_control_game")
                            .from(TimeControl::Table, TimeControl::GameId)
                            .to(Game::Table, Game::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for efficient querying
        manager
            .create_index(
                Index::create()
                    .name("idx_time_control_game_id")
                    .table((Smdb, TimeControl::Table))
                    .col(TimeControl::GameId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_time_control_last_move_time")
                    .table((Smdb, TimeControl::Table))
                    .col(TimeControl::LastMoveTime)
                    .to_owned(),
            )
            .await?;

        // Create index for finding active games with running clocks
        manager
            .create_index(
                Index::create()
                    .name("idx_time_control_running_clocks")
                    .table((Smdb, TimeControl::Table))
                    .col(TimeControl::WhiteClockRunning)
                    .col(TimeControl::BlackClockRunning)
                    .to_owned(),
            )
            .await?;

        println!("Time control table created successfully.");
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes
        manager
            .drop_index(Index::drop().name("idx_time_control_game_id").table((Smdb, TimeControl::Table)).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_time_control_last_move_time").table((Smdb, TimeControl::Table)).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_time_control_running_clocks").table((Smdb, TimeControl::Table)).to_owned())
            .await?;

        // Drop Foreign Key
        manager
            .drop_foreign_key(ForeignKey::drop().name("fk_time_control_game").table((Smdb, TimeControl::Table)).to_owned())
            .await?;

        // Drop the table
        manager
            .drop_table(Table::drop().table((Smdb, TimeControl::Table)).to_owned())
            .await?;

        println!("Time control table dropped successfully.");
        Ok(())
    }
}

// Define the TimeControl table structure for use within this migration
#[derive(DeriveIden)]
enum TimeControl {
    Table,
    Id,
    GameId,
    InitialTime,
    Increment,
    Delay,
    WhiteRemainingTime,
    BlackRemainingTime,
    WhiteClockRunning,
    BlackClockRunning,
    LastMoveTime,
    CreatedAt,
    UpdatedAt,
}

// Define the schema identifier
#[derive(DeriveIden)]
struct Smdb;
