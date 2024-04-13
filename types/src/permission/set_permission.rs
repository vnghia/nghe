use nghe_proc_macros::{add_common_convert, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct SetPermissionParams {
    pub user_ids: Vec<Uuid>,
    pub music_folder_ids: Vec<Uuid>,
    pub allow: bool,
}

#[add_subsonic_response]
pub struct SetPermissionBody {}
