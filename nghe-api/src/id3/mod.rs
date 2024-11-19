mod album;
mod artist;
mod artist_with_albums;
mod date;
mod genre;

pub use album::Album;
pub use artist::{Artist, Role};
pub use artist_with_albums::ArtistWithAlbum;
pub use date::Date;
pub use genre::{Genre, Genres};

pub mod builder {
    pub mod artist {
        pub use super::super::artist::artist_builder::*;
        pub use super::super::artist::ArtistBuilder as Builder;
    }

    pub mod album {
        pub use super::super::album::album_builder::*;
        pub use super::super::album::AlbumBuilder as Builder;
    }
}
