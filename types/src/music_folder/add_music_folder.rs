use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};

#[add_common_convert]
pub struct AddMusicFolderParams {
    pub name: String,
    pub path: String,
    pub permission: bool,
}

#[add_subsonic_response]
pub struct AddMusicFolderBody {}

add_request_types_test!(AddMusicFolderParams);
