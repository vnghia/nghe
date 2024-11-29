use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "unstar")]
#[cfg_attr(feature = "test", derive(Default))]
pub struct Request {
    #[serde(rename = "id")]
    pub song_ids: Option<Vec<Uuid>>,
    #[serde(rename = "albumId")]
    pub album_ids: Option<Vec<Uuid>>,
    #[serde(rename = "artistId")]
    pub artist_ids: Option<Vec<Uuid>>,
}

#[api_derive]
pub struct Response;
