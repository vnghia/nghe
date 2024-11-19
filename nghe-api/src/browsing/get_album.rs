use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive(endpoint = true)]
#[endpoint(path = "getAlbum")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct Response {
    pub album: id3::album::WithArtistsSongs,
}
