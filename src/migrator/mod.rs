pub use sea_orm_migration::prelude::*;

mod m20240106_000001_create_user_table;
mod m20240112_000002_create_music_folder_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240106_000001_create_user_table::Migration),
            Box::new(m20240112_000002_create_music_folder_table::Migration),
        ]
    }
}
