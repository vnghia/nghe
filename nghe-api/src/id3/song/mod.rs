mod full;
mod short;

use std::borrow::Cow;

use bon::Builder;
pub use full::Full;
use nghe_proc_macro::api_derive;
pub use short::Short;
use time::OffsetDateTime;
use uuid::Uuid;

use super::artist;

#[api_derive]
#[derive(Builder)]
#[builder(on(_, required))]
#[builder(state_mod(vis = "pub"))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub track: Option<u16>,
    pub year: Option<u16>,
    pub cover_art: Option<Uuid>,
    pub size: u32,
    pub content_type: Cow<'static, str>,
    pub suffix: Cow<'static, str>,
    pub duration: u32,
    pub bit_rate: u32,
    pub bit_depth: Option<u8>,
    pub sampling_rate: u32,
    pub channel_count: u8,
    pub disc_number: Option<u16>,
    pub created: OffsetDateTime,
    pub artists: Vec<artist::Required>,
    pub music_brainz_id: Option<Uuid>,
}
