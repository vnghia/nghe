use nghe_proc_macro::api_derive;

use super::playlist;

#[api_derive]
#[endpoint(path = "getPlaylists")]
pub struct Request;

#[api_derive]
pub struct Playlists {
    pub playlist: Vec<playlist::Playlist>,
}

#[api_derive]
pub struct Response {
    pub playlists: Playlists,
}
