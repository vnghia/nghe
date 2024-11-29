use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive]
#[endpoint(path = "getStarred2")]
pub struct Request {
    #[serde(rename = "musicFolderId")]
    pub music_folder_ids: Option<Vec<Uuid>>,
}

#[api_derive]
pub struct Starred2 {
    pub artist: Vec<id3::artist::Artist>,
    pub album: Vec<id3::album::Album>,
    pub song: Vec<id3::song::Short>,
}

#[api_derive]
pub struct Response {
    pub starred2: Starred2,
}
