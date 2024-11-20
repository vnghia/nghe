use bon::Builder;
use nghe_proc_macro::api_derive;
use uuid::Uuid;

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")],
)]
#[api_derive(response = true)]
#[derive(Builder)]
#[builder(state_mod(vis = "pub"))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub track: Option<u16>,
    pub year: Option<u16>,
    pub size: u32,
    pub content_type: &'static str,
    pub suffix: &'static str,
    pub duration: u32,
    pub bit_rate: u32,
    pub bit_depth: Option<u8>,
    pub sampling_rate: u32,
    pub channel_count: u8,
    pub disc_number: Option<u16>,
    pub music_brainz_id: Option<Uuid>,
}
