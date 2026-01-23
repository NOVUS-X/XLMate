use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Player::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Player::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Player::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(Player::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Player::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Player::WalletAddress).string().null())
                    .col(ColumnDef::new(Player::Biography).string().not_null().default("".to_string()))
                    .col(ColumnDef::new(Player::Country).string().not_null().default("Unknown".to_string()))
                    .col(ColumnDef::new(Player::Flair).string().not_null().default("".to_string()))
                    .col(ColumnDef::new(Player::RealName).string().not_null().default("".to_string()))
                    .col(ColumnDef::new(Player::Location).string().null())
                    .col(ColumnDef::new(Player::FideRating).integer().null())
                    .col(ColumnDef::new(Player::SocialLinks).json().null())
                    .col(ColumnDef::new(Player::IsEnabled).boolean().not_null().default(true))
                    .col(
                        ColumnDef::new(Player::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Player::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Player::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Player {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    WalletAddress,
    Biography,
    Country,
    Flair,
    RealName,
    Location,
    FideRating,
    SocialLinks,
    IsEnabled,
    CreatedAt,
    UpdatedAt,
}
