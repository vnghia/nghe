use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "updatePlaylist")]
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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    crate::test_roundtrip!(Request, Response);
}
