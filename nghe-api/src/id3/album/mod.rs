mod full;

use bon::Builder;
pub use full::Full;
use nghe_proc_macro::api_derive;
use uuid::Uuid;

use super::{artist, date, genre};

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")],
    Vec => #[serde(skip_serializing_if = "Vec::is_empty")],
    date::Date => #[serde(skip_serializing_if = "date::Date::is_none")],
    genre::Genres => #[serde(skip_serializing_if = "genre::Genres::is_empty")],
)]
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
