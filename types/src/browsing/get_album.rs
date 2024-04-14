use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use uuid::Uuid;

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetAlbumParams {
    pub id: Uuid,
}

#[add_types_derive]
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

add_request_types_test!(GetAlbumParams);
