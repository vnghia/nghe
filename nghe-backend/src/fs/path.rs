use ::serde::{Deserialize, Deserializer, Serializer};
use typed_path::Utf8TypedPathBuf;

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
    Ok(<String>::deserialize(deserializer)?.into())
}
