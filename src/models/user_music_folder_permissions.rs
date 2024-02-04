use super::music_folders::MusicFolder;
use super::users::User;
pub use crate::schema::user_music_folder_permissions;
pub use user_music_folder_permissions::*;

use diesel::prelude::*;
use uuid::Uuid;

#[derive(
    Debug,
    Identifiable,
    Associations,
    Queryable,
    Selectable,
    Insertable,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[diesel(table_name = user_music_folder_permissions)]
#[diesel(primary_key(user_id, music_folder_id))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(MusicFolder))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserMusicFolderPermission {
    pub user_id: Uuid,
    pub music_folder_id: Uuid,
    pub allow: bool,
}

pub type NewUserMusicFolderPermission = UserMusicFolderPermission;
