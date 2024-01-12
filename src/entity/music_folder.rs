//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize)]
#[sea_orm(table_name = "music_folder")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub path: String,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_music_folder::Entity")]
    UserMusicFolder,
}

impl Related<super::user_music_folder::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserMusicFolder.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_music_folder::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::user_music_folder::Relation::MusicFolder.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
