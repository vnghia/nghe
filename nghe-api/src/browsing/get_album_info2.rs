use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[api_derive]
#[endpoint(path = "getAlbumInfo2")]
pub struct Request {
    pub id: Uuid,
}

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")],
)]
#[api_derive]
pub struct AlbumInfo {
    // TODO: add notes field
    pub music_brainz_id: Option<Uuid>,
}

#[api_derive]
pub struct Response {
    pub album_info: AlbumInfo,
}
