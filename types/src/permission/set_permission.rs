use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct SetPermissionParams {
    pub user_id: Option<Uuid>,
    pub music_folder_id: Option<Uuid>,
    pub allow: bool,
}

#[add_subsonic_response]
pub struct SetPermissionBody {}

add_request_types_test!(SetPermissionParams);
