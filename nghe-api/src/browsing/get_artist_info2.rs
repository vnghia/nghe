use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "getArtistInfo2")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct ArtistInfo2 {
    // TODO: add biography and lastfm url field
    pub music_brainz_id: Option<Uuid>,
}

#[api_derive]
pub struct Response {
    pub artist_info2: ArtistInfo2,
}
