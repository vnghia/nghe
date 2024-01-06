use sea_orm_migration::prelude::*;
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240106_000001_create_user_table"
    }
}

const IDX_PK_USER: &'static str = "idx-pk-user";
const IDX_USER_USERNAME: &'static str = "idx-user-username";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .default(PgFunc::gen_random_uuid())
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::Username).string().not_null())
                    .col(ColumnDef::new(User::Password).binary().not_null())
                    .col(ColumnDef::new(User::Email).string().not_null())
                    .col(
                        ColumnDef::new(User::AdminRole)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::DownloadRole)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::ShareRole)
                            .boolean()
                            .default(false)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .primary_key(Index::create().name(IDX_PK_USER).col(User::Id))
                    .index(
                        Index::create()
                            .name(IDX_USER_USERNAME)
                            .col(User::Username)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Username,
    Email,
    Password,
    AdminRole,
    DownloadRole,
    ShareRole,
    CreatedAt,
    UpdatedAt,
}
