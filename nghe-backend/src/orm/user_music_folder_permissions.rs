use diesel::prelude::*;
use uuid::Uuid;

pub use crate::schema::user_music_folder_permissions;

pub mod schema {
    pub use super::user_music_folder_permissions::*;
}

pub use schema::table;

#[derive(Insertable)]
#[diesel(table_name = user_music_folder_permissions, check_for_backend(super::Type))]
pub struct New {
    pub user_id: Uuid,
    pub music_folder_id: Uuid,
}
