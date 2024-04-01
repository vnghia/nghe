use diesel::prelude::*;
pub use user_music_folder_permissions::*;
use uuid::Uuid;

pub use crate::schema::user_music_folder_permissions;

#[derive(Insertable)]
#[diesel(table_name = user_music_folder_permissions)]
pub struct NewUserMusicFolderPermission {
    pub user_id: Uuid,
    pub music_folder_id: Uuid,
    pub allow: bool,
}

#[cfg(test)]
pub type UserMusicFolderPermission = NewUserMusicFolderPermission;
