mod common;
mod id3v2;
mod information;
mod lyric;
mod property;
mod tag;
mod vorbis_comments;

pub use information::SongInformation;
pub use lyric::{LyricLine, SongLyric};
pub use tag::{MediaDateMbz, SongDate};

#[cfg(test)]
pub mod test {
    pub use super::property::SongProperty;
    pub use super::tag::SongTag;
}
