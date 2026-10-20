mod full;
mod short;

use std::borrow::Cow;

#[cfg(any(test, feature = "test"))]
use fake::{Fake, Faker};
pub use full::Full;
use nghe_proc_macro::api_derive;
pub use short::Short;
use time::OffsetDateTime;
use uuid::Uuid;

use super::artist;

#[api_derive]
#[derive(bon::Builder)]
#[builder(on(_, required))]
#[builder(state_mod(vis = "pub"))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub track: Option<u16>,
    pub year: Option<u16>,
    pub cover_art: Option<Uuid>,
    pub size: u32,
    #[dummy(expr = "Faker.fake::<String>().into()")]
    pub content_type: Cow<'static, str>,
    #[dummy(expr = "Faker.fake::<String>().into()")]
    pub suffix: Cow<'static, str>,
    #[dummy(expr = "time::SignedDuration::seconds(Faker.fake())")]
    pub duration: time::SignedDuration,
    pub bit_rate: u32,
    pub bit_depth: Option<u8>,
    pub sampling_rate: u32,
    pub channel_count: u8,
    pub disc_number: Option<u16>,
    #[dummy(expr = "Faker.fake::<time::PrimitiveDateTime>().assume_utc()")]
    pub created: OffsetDateTime,
    pub artist: String,
    pub artist_id: Uuid,
    pub artists: Vec<artist::Required>,
    pub music_brainz_id: Option<Uuid>,
    #[dummy(expr = "Faker.fake::<Option<time::PrimitiveDateTime>>().\
                    map(time::PrimitiveDateTime::assume_utc)")]
    pub starred: Option<OffsetDateTime>,
}
