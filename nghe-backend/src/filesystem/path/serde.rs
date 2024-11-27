use ::serde::{Deserialize, Deserializer, Serializer};
use typed_path::Utf8NativePathBuf;

pub fn serialize<S>(path: &Utf8NativePathBuf, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(path.as_str())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Utf8NativePathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    <String>::deserialize(deserializer).map(Utf8NativePathBuf::from)
}

pub mod option {
    #![allow(clippy::ref_option)]

    use super::*;

    pub fn serialize<S>(path: &Option<Utf8NativePathBuf>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(path) = path {
            serializer.serialize_str(path.as_str())
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Utf8NativePathBuf>, D::Error>
    where
        D: Deserializer<'de>,
    {
        <String>::deserialize(deserializer).map(|path| {
            let path = Utf8NativePathBuf::from(path);
            if path.is_absolute() { Some(path) } else { None }
        })
    }
}
