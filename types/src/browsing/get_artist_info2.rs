use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};
use uuid::Uuid;

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetArtistInfo2Params {
    pub id: Uuid,
}

#[add_types_derive]
#[derive(Debug)]
pub struct ArtistInfo {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub biography: Option<String>,
    #[serde(flatten)]
    pub info: InfoId3,
}

#[add_subsonic_response]
pub struct GetArtistInfo2Body {
    pub artist_info2: ArtistInfo,
}
