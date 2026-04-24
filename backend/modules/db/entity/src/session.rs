use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, DeriveEntityModel, PartialEq, Eq)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub user_id: i32,

    #[sea_orm(column_type = "Text", nullable)]
    pub ip_address: Option<String>,

    #[sea_orm(column_type = "Text", nullable)]
    pub user_agent: Option<String>,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub expires_at: DateTime<Utc>,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub last_active_at: DateTime<Utc>,

    pub is_revoked: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
