use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

use super::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct CreatePlaylistParams {
    pub name: Option<String>,
    pub playlist_id: Option<Uuid>,
    #[serde(rename = "songId")]
    pub song_ids: Option<Vec<Uuid>>,
}

#[add_subsonic_response]
pub struct CreatePlaylistBody {
    pub playlist: PlaylistId3WithSongs,
}

add_request_types_test!(CreatePlaylistParams);
