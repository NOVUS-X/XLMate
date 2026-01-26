use sea_orm::entity::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "game_result")]
pub enum GameResult {
    #[sea_orm(string_value = "ongoing")]
    Ongoing,
    #[sea_orm(string_value = "white_wins")]
    WhiteWins,
    #[sea_orm(string_value = "black_wins")]
    BlackWins,
    #[sea_orm(string_value = "draw")]
    Draw,
    #[sea_orm(string_value = "abandoned")]
    Abandoned,
}

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub result: GameResult,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::game_move::Entity")]
    GameMoves,
}

impl Related<super::game_move::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GameMoves.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

