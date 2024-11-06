use ::serde::{Deserialize, Deserializer, Serializer};
use typed_path::Utf8TypedPathBuf;

use crate::filesystem::path;

pub fn serialize<S>(path: &Utf8TypedPathBuf, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(path.as_str())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Utf8TypedPathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    <String>::deserialize(deserializer).map(path::Local::from_string)
}

pub mod option {
    #![allow(clippy::ref_option)]

    use super::*;

    pub fn serialize<S>(path: &Option<Utf8TypedPathBuf>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(path) = path {
            serializer.serialize_str(path.as_str())
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Utf8TypedPathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        <String>::deserialize(deserializer).map(|path| {
            let path = path::Local::from_string(path);
            if path.is_absolute() { Some(path) } else { None }
        })
    }
}
