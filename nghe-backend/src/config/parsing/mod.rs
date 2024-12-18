pub mod id3v2;
pub mod vorbis_comments;

use serde::{Deserialize, Serialize};
use vorbis_comments::VorbisComments;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Parsing {
    pub vorbis_comments: VorbisComments,
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    impl Parsing {
        pub fn test() -> Self {
            Self { vorbis_comments: VorbisComments::test() }
        }
    }
}
