use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive(endpoint = true)]
#[endpoint(path = "getSong")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct Response {
    pub song: id3::song::WithAlbumGenres,
}
