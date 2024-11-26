use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::formats::SpaceSeparator;
use serde_with::{serde_as, StringWithSeparator};
use typed_path::utils::utf8_temp_dir;
use typed_path::Utf8NativePathBuf;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct CoverArt {
    #[serde(with = "crate::filesystem::path::serde::option")]
    #[derivative(Default(
        value = "Some(utf8_temp_dir().unwrap().join(\"nghe\").join(\"cache\").join(\"cover_art\"))"
    ))]
    pub dir: Option<Utf8NativePathBuf>,
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, String>")]
    #[derivative(Default(value = "vec![\"cover.jpg\".to_owned(), \"cover.png\".to_owned()]"))]
    pub names: Vec<String>,
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;
    use typed_path::Utf8NativePath;

    use super::*;
    use crate::file::picture;

    impl CoverArt {
        pub fn with_prefix(self, prefix: impl AsRef<Utf8NativePath>) -> Self {
            Self {
                dir: self.dir.map(|_| prefix.as_ref().join("cache").join("cover_art")),
                names: picture::Format::iter().map(picture::Format::name).collect(),
            }
        }
    }
}
