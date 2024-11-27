mod full;
mod required;

use bon::Builder;
pub use full::Full;
use nghe_proc_macro::api_derive;
pub use required::Required;
use strum::IntoStaticStr;
use uuid::Uuid;

#[api_derive(response = false)]
#[derive(IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    Artist,
    AlbumArtist,
}

#[api_derive]
#[derive(Builder)]
#[builder(on(_, required))]
#[builder(state_mod(vis = "pub"))]
pub struct Artist {
    #[serde(flatten)]
    pub required: Required,
    pub album_count: u16,
    pub music_brainz_id: Option<Uuid>,
    #[builder(default)]
    pub roles: Vec<Role>,
}

mod serde {
    use ::serde::{Serialize, Serializer};

    use super::*;

    impl Serialize for Role {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(self.into())
        }
    }
}
