use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(uuid(RefreshTokens::Id).primary_key())
                    .col(uuid(RefreshTokens::PlayerId).not_null())
                    .col(string_len(RefreshTokens::TokenHash, 256).not_null())
                    .col(uuid(RefreshTokens::FamilyId).not_null())
                    .col(timestamp_with_time_zone(RefreshTokens::ExpiresAt).not_null())
                    .col(boolean(RefreshTokens::IsRevoked).not_null().default(false))
                    .col(timestamp_with_time_zone(RefreshTokens::CreatedAt).not_null())
                    .col(timestamp_with_time_zone_null(RefreshTokens::UsedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-refresh_tokens-player_id")
                            .from(RefreshTokens::Table, RefreshTokens::PlayerId)
                            .to(Player::Table, Player::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on token_hash for fast lookups
        manager
            .create_index(
                Index::create()
                    .name("idx-refresh_tokens-token_hash")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::TokenHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on family_id for batch invalidation
        manager
            .create_index(
                Index::create()
                    .name("idx-refresh_tokens-family_id")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::FamilyId)
                    .to_owned(),
            )
            .await?;

        // Index on player_id for user token lookups
        manager
            .create_index(
                Index::create()
                    .name("idx-refresh_tokens-player_id")
                    .table(RefreshTokens::Table)
                    .col(RefreshTokens::PlayerId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RefreshTokens {
    Table,
    Id,
    PlayerId,
    TokenHash,
    FamilyId,
    ExpiresAt,
    IsRevoked,
    CreatedAt,
    UsedAt,
}

#[derive(DeriveIden)]
enum Player {
    Table,
    Id,
}
