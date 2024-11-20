pub mod album;
pub mod artist;
pub mod date;
pub mod genre;
pub mod song;

pub mod builder {
    pub mod artist {
        pub use super::super::artist::artist_builder::*;
        pub use super::super::artist::ArtistBuilder as Builder;
    }

    pub mod album {
        pub use super::super::album::album_builder::*;
        pub use super::super::album::AlbumBuilder as Builder;
    }

    pub mod song {
        pub use super::super::song::song_builder::*;
        pub use super::super::song::SongBuilder as Builder;
    }
}
