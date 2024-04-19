use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

use super::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetPlaylistParams {
    pub id: Uuid,
}

#[add_subsonic_response]
pub struct GetPlaylistBody {
    pub playlist: PlaylistId3WithSongs,
}

add_request_types_test!(GetPlaylistParams);
