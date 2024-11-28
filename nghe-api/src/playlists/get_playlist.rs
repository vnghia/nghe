use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::playlist;

#[api_derive]
#[endpoint(path = "getPlaylist")]
pub struct Request {
    pub id: Uuid,
}

#[api_derive]
pub struct Response {
    pub playlist: playlist::Full,
}
