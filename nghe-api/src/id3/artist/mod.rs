mod with_albums;

use bon::Builder;
use nghe_proc_macro::api_derive;
use strum::IntoStaticStr;
use uuid::Uuid;
pub use with_albums::WithAlbums;

#[api_derive(response = true, json = false)]
#[derive(IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    Artist,
    AlbumArtist,
}

#[serde_with::apply(
    Option => #[serde(skip_serializing_if = "Option::is_none")],
    Vec => #[serde(skip_serializing_if = "Vec::is_empty")],
)]
#[api_derive(response = true)]
#[derive(Builder)]
#[builder(state_mod(vis = "pub"))]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub album_count: Option<u16>,
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
