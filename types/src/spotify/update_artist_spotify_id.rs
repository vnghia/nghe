use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct UpdateArtistSpotifyIdParams {
    pub artist_id: Uuid,
    pub spotify_url: String,
}

#[add_subsonic_response]
pub struct UpdateArtistSpotifyIdBody {}

add_request_types_test!(UpdateArtistSpotifyIdParams);
