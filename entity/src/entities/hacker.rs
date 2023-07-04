//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "hacker")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub discord_id: String,
    pub username: String,
    pub fk_team_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::submission::Entity")]
    Submission,
    #[sea_orm(
        belongs_to = "super::team::Entity",
        from = "Column::FkTeamId",
        to = "super::team::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Team,
    #[sea_orm(has_many = "super::token::Entity")]
    Token,
}

impl Related<super::submission::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Submission.def()
    }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl Related<super::token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Token.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
