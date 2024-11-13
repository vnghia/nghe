use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive(endpoint = true)]
#[endpoint(path = "getArtists")]
pub struct Request {
    pub music_folder_id: Option<Uuid>,
}

#[api_derive(response = true)]
pub struct Index {
    pub name: String,
    pub artist: Vec<id3::Artist>,
}

#[api_derive(response = true)]
pub struct Artists {
    pub ignored_articles: String,
    pub index: Vec<Index>,
}

#[api_derive]
pub struct Response {
    pub artists: Artists,
}
