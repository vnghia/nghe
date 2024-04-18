use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct GetMusicFolderIdsParams {}

#[add_subsonic_response]
#[derive(Debug)]
pub struct GetMusicFolderIdsBody {
    pub ids: Vec<Uuid>,
}

add_request_types_test!(GetMusicFolderIdsParams);
