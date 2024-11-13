use bon::Builder;
use nghe_proc_macro::api_derive;
use serde_with::skip_serializing_none;
use uuid::Uuid;

#[skip_serializing_none]
#[api_derive(response = true)]
#[derive(Builder)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub album_count: Option<u16>,
    pub music_brainz_id: Option<Uuid>,
}
