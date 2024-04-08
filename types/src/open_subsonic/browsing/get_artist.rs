use nghe_proc_macros::{add_common_convert, add_response_derive, add_subsonic_response};
use uuid::Uuid;

use crate::open_subsonic::common::id3::response::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetArtistParams {
    pub id: Uuid,
}

#[add_response_derive]
#[derive(Debug)]
pub struct ArtistId3WithAlbums {
    #[serde(flatten)]
    pub artist: ArtistId3,
    #[serde(rename = "album")]
    pub albums: Vec<AlbumId3>,
}

#[add_subsonic_response]
pub struct GetArtistBody {
    pub artist: ArtistId3WithAlbums,
}