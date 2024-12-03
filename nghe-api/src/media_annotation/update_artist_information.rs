use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "updateArtistInformation", internal = true)]
pub struct Request {
    pub artist_id: Uuid,
    pub spotify_id: Option<String>,
}

#[api_derive]
pub struct Response;
