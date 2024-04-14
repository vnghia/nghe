use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use uuid::Uuid;

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetArtistsParams {
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[add_types_derive]
#[cfg_attr(feature = "test", derive(Debug))]
pub struct Index {
    pub name: String,
    #[serde(rename = "artist")]
    pub artists: Vec<ArtistId3>,
}

#[add_types_derive]
#[cfg_attr(feature = "test", derive(Debug))]
pub struct Indexes {
    pub ignored_articles: String,
    pub index: Vec<Index>,
}

#[add_subsonic_response]
pub struct GetArtistsBody {
    pub artists: Indexes,
}

add_request_types_test!(GetArtistsParams);
