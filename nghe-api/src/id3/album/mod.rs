mod full;

use bon::Builder;
pub use full::Full;
use nghe_proc_macro::api_derive;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{artist, date, genre};

#[api_derive]
#[derive(Builder)]
#[builder(on(_, required))]
#[builder(state_mod(vis = "pub"))]
pub struct Album {
    pub id: Uuid,
    pub name: String,
    pub cover_art: Option<Uuid>,
    pub song_count: u16,
    pub duration: u32,
    pub created: OffsetDateTime,
    pub year: Option<u16>,
    pub music_brainz_id: Option<Uuid>,
    #[builder(default)]
    pub genres: genre::Genres,
    #[builder(default)]
    pub artists: Vec<artist::Artist>,
    #[builder(default)]
    pub original_release_date: date::Date,
    #[builder(default)]
    pub release_date: date::Date,
}
