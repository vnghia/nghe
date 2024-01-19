use super::m20240112_000002_create_music_folder_table::MusicFolder;

use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240119_000004_create_song_table"
    }
}

const IDX_PK_SONG: &str = "idx-pk-song";
const IDX_SONG_PATH_MUSIC_FOLDER_ID: &str = "idx-song-path-music_folder_id";
const FK_SONG_MUSIC_FOLDER_ID: &str = "fk-song-music_folder_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Song::Table)
                    .col(ColumnDef::new(Song::Id).uuid().not_null())
                    .col(ColumnDef::new(Song::Title).string().not_null())
                    .col(ColumnDef::new(Song::Path).string().not_null())
                    .col(ColumnDef::new(Song::MusicFolderId).uuid().not_null())
                    .primary_key(Index::create().name(IDX_PK_SONG).col(Song::Id))
                    .index(
                        Index::create()
                            .name(IDX_SONG_PATH_MUSIC_FOLDER_ID)
                            .col(Song::Path)
                            .col(Song::MusicFolderId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_SONG_MUSIC_FOLDER_ID)
                            .from(Song::Table, Song::MusicFolderId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .to(MusicFolder::Table, MusicFolder::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Song::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Song {
    Table,
    Id,
    Title,
    Path,
    MusicFolderId,
}
