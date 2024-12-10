use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::{serde_as, StringWithSeparator};
use typed_path::utils::utf8_temp_dir;
use typed_path::Utf8PlatformPathBuf;

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
                .join("cache")
                .join("cover_art")
                .with_platform_encoding_checked()
                .unwrap()
        )
    ))]
    pub dir: Option<Utf8PlatformPathBuf>,
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, String>")]
    #[educe(Default(expression = vec!["cover.jpg".to_owned(), "cover.png".to_owned()]))]
    pub names: Vec<String>,
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
                dir: self.dir.map(|_| prefix.as_ref().join("cache").join("cover_art")),
                names: picture::Format::iter().map(picture::Format::name).collect(),
            }
        }
    }
}
