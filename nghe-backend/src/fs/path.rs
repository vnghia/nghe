#![allow(clippy::module_name_repetitions)]

use typed_path::{Utf8NativeEncoding, Utf8Path, Utf8PathBuf};

pub type LocalPath = Utf8Path<Utf8NativeEncoding>;
pub type LocalPathBuf = Utf8PathBuf<Utf8NativeEncoding>;

pub mod serde {
    use ::serde::{Deserialize, Deserializer, Serializer};

    use super::*;

    pub fn serialize<S>(path: &LocalPathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(path.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<LocalPathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(<String>::deserialize(deserializer)?.into())
    }
}
