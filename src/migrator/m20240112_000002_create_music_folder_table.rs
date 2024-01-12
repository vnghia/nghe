use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240112_000002_create_music_folder_table"
    }
}

const IDX_PK_MUSIC_FOLDER: &str = "idx-pk-music-folder";
const IDX_MUSIC_FOLDER_PATH: &str = "idx-music-folder-path";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MusicFolder::Table)
                    .col(
                        ColumnDef::new(MusicFolder::Id)
                            .uuid()
                            .default(PgFunc::gen_random_uuid())
                            .not_null(),
                    )
                    .col(ColumnDef::new(MusicFolder::Path).string().not_null())
                    .col(
                        ColumnDef::new(MusicFolder::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name(IDX_PK_MUSIC_FOLDER)
                            .col(MusicFolder::Id),
                    )
                    .index(
                        Index::create()
                            .name(IDX_MUSIC_FOLDER_PATH)
                            .col(MusicFolder::Path)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MusicFolder::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum MusicFolder {
    Table,
    Id,
    Path,
    UpdatedAt,
}
