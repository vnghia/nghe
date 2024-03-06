pub use crate::schema::user_music_folder_permissions;
pub use user_music_folder_permissions::*;

use diesel::prelude::*;
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = user_music_folder_permissions)]
#[cfg_attr(
    test,
    derive(
        Debug, Clone, Copy, Queryable, Selectable, PartialEq, Eq, PartialOrd, Ord
    )
)]
pub struct NewUserMusicFolderPermission {
    pub user_id: Uuid,
    pub music_folder_id: Uuid,
    pub allow: bool,
}

#[cfg(test)]
pub type UserMusicFolderPermission = NewUserMusicFolderPermission;
