mod full;
mod required;

#[cfg(any(test, feature = "test"))]
use fake::{Fake, Faker};
pub use full::Full;
use nghe_proc_macro::api_derive;
pub use required::Required;
use strum::IntoStaticStr;
use time::OffsetDateTime;
use uuid::Uuid;

#[api_derive(serde(case = "lowercase"))]
#[derive(PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    Artist,
    AlbumArtist,
}

#[api_derive]
#[derive(bon::Builder)]
#[builder(on(_, required))]
#[builder(state_mod(vis = "pub"))]
pub struct Artist {
    #[serde(flatten)]
    pub required: Required,
    pub cover_art: Option<Uuid>,
    pub album_count: u16,
    #[dummy(expr = "Faker.fake::<Option<time::PrimitiveDateTime>>().\
                    map(time::PrimitiveDateTime::assume_utc)")]
    pub starred: Option<OffsetDateTime>,
    pub music_brainz_id: Option<Uuid>,
    pub roles: Vec<Role>,
}
