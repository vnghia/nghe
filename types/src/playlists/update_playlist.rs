use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
#[derive(Debug)]
pub struct UpdatePlaylistParams {
    pub playlist_id: Uuid,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    #[serde(rename = "songIdToAdd")]
    pub song_ids_to_add: Vec<Uuid>,
    #[serde(rename = "songIndexToRemove")]
    pub song_indexes_to_remove: Vec<u32>,
}

#[add_subsonic_response]
pub struct UpdatePlaylistBody {}

add_request_types_test!(UpdatePlaylistParams);
