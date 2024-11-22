use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
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
}

#[cfg(test)]
mod test {
    use typed_path::Utf8NativePath;

    use super::*;

    impl CoverArt {
        pub fn with_prefix(self, prefix: impl AsRef<Utf8NativePath>) -> Self {
            Self { dir: self.dir.map(|_| prefix.as_ref().join("cache").join("cover_art")) }
        }
    }
}
