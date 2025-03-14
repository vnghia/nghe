use ::serde::{Deserialize, Deserializer, Serializer};
use typed_path::Utf8PlatformPathBuf;

pub mod option {
    #![allow(clippy::ref_option)]

    use super::*;

    pub fn serialize<S>(
        path: &Option<Utf8PlatformPathBuf>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(path) = path {
            serializer.serialize_str(path.as_str())
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Utf8PlatformPathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        <String>::deserialize(deserializer).map(|path| {
            let path = Utf8PlatformPathBuf::from(path);
            if path.is_absolute() { Some(path) } else { None }
        })
    }
}
