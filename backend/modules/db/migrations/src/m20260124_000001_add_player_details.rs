use sea_orm_migration::prelude::*;

use crate::m20250428_121011_create_players_table::Player;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("biography")).string().not_null().default("".to_string()))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("country")).string().not_null().default("Unknown".to_string()))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("flair")).string().not_null().default("".to_string()))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("real_name")).string().not_null().default("".to_string()))
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("location")).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("fide_rating")).integer().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("social_links")).json().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .add_column(ColumnDef::new(Alias::new("is_enabled")).boolean().not_null().default(true))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Player::Table)
                    .drop_column(Alias::new("biography"))
                    .drop_column(Alias::new("country"))
                    .drop_column(Alias::new("flair"))
                    .drop_column(Alias::new("real_name"))
                    .drop_column(Alias::new("location"))
                    .drop_column(Alias::new("fide_rating"))
                    .drop_column(Alias::new("social_links"))
                    .drop_column(Alias::new("is_enabled"))
                    .to_owned(),
            )
            .await
    }
}
