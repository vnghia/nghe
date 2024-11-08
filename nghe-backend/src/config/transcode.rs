use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_path::utils::utf8_temp_dir;
use typed_path::Utf8NativePathBuf;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct Transcode {
    #[derivative(Default(value = "32 * 1024"))]
    pub buffer_size: usize,
    #[derivative(Default(value = "Some(10)"))]
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    pub channel_size: Option<usize>,
    #[serde(with = "crate::filesystem::path::serde::option")]
    #[derivative(Default(
        value = "Some(utf8_temp_dir().unwrap().join(\"nghe\").join(\"cache\").join(\"transcode\"))"
    ))]
    pub cache_dir: Option<Utf8NativePathBuf>,
}
