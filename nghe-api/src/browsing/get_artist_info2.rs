use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "getArtistInfo2")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
#[derive(Default)]
pub struct ArtistInfo2 {
    pub music_brainz_id: Option<Uuid>,
    #[serde(rename = "lastFmUrl")]
    pub lastfm_url: Option<String>,
    pub biography: Option<String>,
}

#[api_derive]
pub struct Response {
    pub artist_info2: ArtistInfo2,
}
