use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};
use uuid::Uuid;

use crate::id3::*;

#[add_common_convert]
#[derive(Debug)]
pub struct GetAlbumInfo2Params {
    pub id: Uuid,
}

#[add_types_derive]
#[derive(Debug)]
pub struct AlbumInfo {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub notes: Option<String>,
    #[serde(flatten)]
    pub info: InfoId3,
}

#[add_subsonic_response]
pub struct GetAlbumInfo2Body {
    pub album_info: AlbumInfo,
}
