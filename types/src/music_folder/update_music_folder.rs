use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

use super::FsType;

#[add_common_convert]
pub struct UpdateMusicFolderParams {
    pub id: Uuid,
    pub name: Option<String>,
    pub path: Option<String>,
    pub fs_type: FsType,
}

#[add_subsonic_response]
pub struct UpdateMusicFolderBody {}

add_request_types_test!(UpdateMusicFolderParams);
