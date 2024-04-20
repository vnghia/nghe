use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

use super::access_level::AccessLevel;

#[add_common_convert]
#[derive(Debug)]
pub struct AddPlaylistUserParams {
    pub playlist_id: Uuid,
    pub user_id: Uuid,
    pub access_level: AccessLevel,
}

#[add_subsonic_response]
pub struct AddPlaylistUserBody {}

add_request_types_test!(AddPlaylistUserParams);
