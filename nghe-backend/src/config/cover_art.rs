use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::{StringWithSeparator, serde_as};
use typed_path::Utf8PlatformPathBuf;
use typed_path::utils::utf8_temp_dir;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct CoverArt {
    #[serde(with = "crate::filesystem::path::serde::option")]
    #[educe(Default(
        expression = Some(
            utf8_temp_dir()
                .unwrap()
                .join("nghe")
                .join("cover_art")
                .with_platform_encoding()
        )
    ))]
    pub dir: Option<Utf8PlatformPathBuf>,
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, String>")]
    #[educe(Default(expression = vec![
        "cover.png".to_owned(),
        "cover.jpg".to_owned(),
        "cover.jpeg".to_owned(),
        "cover.webp".to_owned(),
    ]))]
    pub names: Vec<String>,
    #[serde(with = "crate::filesystem::path::serde::option")]
    #[educe(Default(
        expression = Some(
            utf8_temp_dir()
                .unwrap()
                .join("nghe")
                .join("cache")
                .join("cover_art")
                .with_platform_encoding()
        )
    ))]
    pub cache_dir: Option<Utf8PlatformPathBuf>,
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use strum::IntoEnumIterator;
    use typed_path::Utf8PlatformPath;

    use super::*;
    use crate::file::picture;

    impl CoverArt {
        pub fn with_prefix(self, prefix: impl AsRef<Utf8PlatformPath>) -> Self {
            Self {
                dir: self.dir.map(|_| prefix.as_ref().join("cover_art")),
                names: picture::Format::iter().map(picture::Format::name).collect(),
                cache_dir: self.cache_dir.map(|_| prefix.as_ref().join("cache").join("cover_art")),
            }
        }
    }
}
