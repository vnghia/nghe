use nghe_proc_macro::api_derive;
use uuid::Uuid;

use crate::id3;

#[api_derive]
#[endpoint(path = "getArtist")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct Response {
    pub artist: id3::artist::Full,
}
