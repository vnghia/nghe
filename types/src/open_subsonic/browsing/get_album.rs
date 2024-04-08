use nghe_proc_macros::{add_common_convert, add_response_derive, add_subsonic_response};
use uuid::Uuid;

use crate::open_subsonic::common::id3::response::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetAlbumParams {
    pub id: Uuid,
}

#[add_response_derive]
#[derive(Debug)]
pub struct AlbumId3WithSongs {
    #[serde(flatten)]
    pub album: AlbumId3,
    #[serde(rename = "song")]
    pub songs: Vec<SongId3>,
}

#[add_subsonic_response]
pub struct GetAlbumBody {
    pub album: AlbumId3WithSongs,
}
