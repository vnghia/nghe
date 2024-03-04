mod common;
mod id3v2;
mod information;
mod vorbis_comments;

pub use information::SongInformation;

#[cfg(test)]
pub mod test {
    pub use super::common::{SongProperty, SongTag};

    pub mod id3v2 {
        pub use super::super::id3v2::test::*;
    }

    pub mod vorbis_comments {
        pub use super::super::vorbis_comments::test::*;
    }
}
