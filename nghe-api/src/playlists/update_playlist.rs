use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "updatePlaylist")]
#[cfg_attr(feature = "test", derive(Default))]
pub struct Request {
    pub playlist_id: Uuid,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    #[serde(rename = "songIdToAdd")]
    pub add_ids: Option<Vec<Uuid>>,
    #[serde(rename = "songIndexToRemove")]
    pub remove_indexes: Option<Vec<u16>>,
}

#[api_derive]
pub struct Response;
