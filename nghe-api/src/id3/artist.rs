use bon::Builder;
use nghe_proc_macro::api_derive;
use serde_with::skip_serializing_none;
use strum::IntoStaticStr;
use uuid::Uuid;

#[api_derive(response = true, serde = false)]
#[derive(IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Role {
    Artist,
    AlbumArtist,
}

#[skip_serializing_none]
#[api_derive(response = true)]
#[derive(Builder)]
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
