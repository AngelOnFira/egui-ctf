//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "submission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub flag: String,
    pub time: DateTime,
    pub correct: bool,
    pub fk_hacker_id: Option<i64>,
    pub fk_team_id: Option<i32>,
    pub fk_challenge_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::challenge::Entity",
        from = "Column::FkChallengeId",
        to = "super::challenge::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Challenge,
    #[sea_orm(
        belongs_to = "super::hacker::Entity",
        from = "Column::FkHackerId",
        to = "super::hacker::Column::DiscordId",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Hacker,
    #[sea_orm(
        belongs_to = "super::team::Entity",
        from = "Column::FkTeamId",
        to = "super::team::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Team,
}

impl Related<super::challenge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Challenge.def()
    }
}

impl Related<super::hacker::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Hacker.def()
    }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
