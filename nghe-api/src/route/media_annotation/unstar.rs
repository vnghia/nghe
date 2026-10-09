use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "unstar")]
#[cfg_attr(feature = "test", derive(Default))]
pub struct Request {
    #[serde(rename = "id")]
    #[conf(repeat, long)]
    pub song_ids: Vec<Uuid>,
    #[serde(rename = "albumId")]
    #[conf(repeat, long)]
    pub album_ids: Vec<Uuid>,
    #[serde(rename = "artistId")]
    #[conf(repeat, long)]
    pub artist_ids: Vec<Uuid>,
}

#[api_derive]
pub struct Response;
