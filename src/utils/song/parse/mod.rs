mod common;
mod id3v2;
mod information;
mod vorbis_comments;

pub use common::SongDate;
pub use information::SongInformation;

#[cfg(test)]
pub mod test {
    pub use super::common::{test::*, SongProperty, SongTag};
}
