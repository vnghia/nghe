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
    #[conf(repeat, long)]
    pub add_ids: Vec<Uuid>,
    #[serde(rename = "songIndexToRemove")]
    #[conf(repeat, long)]
    pub remove_indexes: Vec<u16>,
}

#[api_derive]
pub struct Response;
