use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_path::Utf8PlatformPathBuf;
use typed_path::utils::utf8_temp_dir;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Transcode {
    #[educe(Default(expression = 32 * 1024))]
    pub buffer_size: usize,
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    #[educe(Default(expression = Some(10)))]
    pub channel_size: Option<usize>,
    #[serde(with = "crate::filesystem::path::serde::option")]
    #[educe(Default(
        expression = Some(
            utf8_temp_dir()
                .unwrap()
                .join("nghe")
                .join("cache")
                .join("transcode")
                .with_platform_encoding()
        )
    ))]
    pub cache_dir: Option<Utf8PlatformPathBuf>,
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use typed_path::Utf8PlatformPath;

    use super::*;

    impl Transcode {
        pub fn with_prefix(self, prefix: impl AsRef<Utf8PlatformPath>) -> Self {
            Self {
                cache_dir: self.cache_dir.map(|_| prefix.as_ref().join("cache").join("transcode")),
                ..self
            }
        }
    }
}
