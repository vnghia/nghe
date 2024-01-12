use super::m20240106_000001_create_user_table::User;
use super::m20240112_000002_create_music_folder_table::MusicFolder;

use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240112_000003_create_user_music_folder_table"
    }
}

const IDX_PK_USER_MUSIC_FOLDER: &str = "idx-pk-user-music-folder";
const FK_USER_MUSIC_FOLDER_USER_ID: &str = "fk-user-music-folder-user_id";
const FK_USER_MUSIC_FOLDER_MUSIC_FOLDER_ID: &str = "fk-user-music-folder-music_folder_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserMusicFolder::Table)
                    .col(ColumnDef::new(UserMusicFolder::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserMusicFolder::MusicFolderId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserMusicFolder::Allow)
                            .boolean()
                            .default(true)
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(IDX_PK_USER_MUSIC_FOLDER)
                            .col(UserMusicFolder::UserId)
                            .col(UserMusicFolder::MusicFolderId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_USER_MUSIC_FOLDER_USER_ID)
                            .from(UserMusicFolder::Table, UserMusicFolder::UserId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_USER_MUSIC_FOLDER_MUSIC_FOLDER_ID)
                            .from(UserMusicFolder::Table, UserMusicFolder::MusicFolderId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .to(MusicFolder::Table, MusicFolder::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserMusicFolder::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum UserMusicFolder {
    Table,
    UserId,
    MusicFolderId,
    Allow,
}
