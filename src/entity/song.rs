//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "song")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub title: String,
    pub path: String,
    pub music_folder_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::music_folder::Entity",
        from = "Column::MusicFolderId",
        to = "super::music_folder::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    MusicFolder,
}

impl Related<super::music_folder::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MusicFolder.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
