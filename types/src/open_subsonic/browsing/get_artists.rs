use nghe_proc_macros::{add_common_convert, add_response_derive, add_subsonic_response};
use uuid::Uuid;

use crate::open_subsonic::common::id3::response::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetArtistsParams {
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[add_response_derive]
#[cfg_attr(feature = "test", derive(Debug))]
pub struct Index {
    pub name: String,
    #[serde(rename = "artist")]
    pub artists: Vec<ArtistId3>,
}

#[add_response_derive]
#[cfg_attr(feature = "test", derive(Debug))]
pub struct Indexes {
    pub ignored_articles: String,
    pub index: Vec<Index>,
}

#[add_subsonic_response]
pub struct GetArtistsBody {
    pub artists: Indexes,
}
