use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
#[derive(Debug)]
pub struct DeletePlaylistParams {
    pub id: Uuid,
}

#[add_subsonic_response]
pub struct DeletePlaylistBody {}

add_request_types_test!(DeletePlaylistParams);
