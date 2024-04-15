use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct RemoveMusicFolderParams {
    pub id: Uuid,
}

#[add_subsonic_response]
pub struct RemoveMusicFolderBody {}

add_request_types_test!(RemoveMusicFolderParams);
