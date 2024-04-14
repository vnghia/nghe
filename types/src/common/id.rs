use std::str::FromStr;

use concat_string::concat_string;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use strum::{AsRefStr, EnumString};
use uuid::Uuid;

pub const TYPED_ID_SEPARATOR: char = ':';
pub const TYPED_ID_STR: &str = TYPED_ID_SEPARATOR.as_ascii().unwrap().as_str();

#[derive(Debug, Default, Clone, Copy, AsRefStr, EnumString)]
#[cfg_attr(feature = "test", derive(PartialEq, Eq))]
#[cfg_attr(test, derive(fake::Dummy))]
pub enum MediaType {
    #[strum(serialize = "ar")]
    #[default]
    Aritst,
    #[strum(serialize = "al")]
    Album,
    #[strum(serialize = "so")]
    Song,
}

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "test", derive(PartialEq, Eq))]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct TypedId<T> {
    pub t: Option<T>,
    pub id: Uuid,
}

pub type MediaTypedId = TypedId<MediaType>;

impl<T: AsRef<str>> Serialize for TypedId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut uuid_buffer = Uuid::encode_buffer();
        let uuid_str = self.id.hyphenated().encode_lower(&mut uuid_buffer);

        if let Some(ref t) = self.t {
            serializer.serialize_str(&concat_string!(t.as_ref(), TYPED_ID_STR, uuid_str))
        } else {
            serializer.serialize_str(uuid_str)
        }
    }
}

impl<'de, T: FromStr> Deserialize<'de> for TypedId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some((t, id)) = s.split_once(TYPED_ID_SEPARATOR) {
            Ok(Self {
                t: Some(
                    T::from_str(t)
                        .map_err(|_| de::Error::custom("could not construct type from string"))?,
                ),
                id: Uuid::from_str(id).map_err(de::Error::custom)?,
            })
        } else {
            Ok(Self { t: None, id: Uuid::from_str(&s).map_err(de::Error::custom)? })
        }
    }
}
