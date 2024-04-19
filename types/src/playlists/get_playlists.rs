use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};

use super::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetPlaylistsParams {}

#[add_types_derive]
pub struct GetPlaylists {
    pub playlist: Vec<PlaylistId3>,
}

#[add_subsonic_response]
pub struct GetPlaylistsBody {
    pub playlists: GetPlaylists,
}

add_request_types_test!(GetPlaylistsParams);
