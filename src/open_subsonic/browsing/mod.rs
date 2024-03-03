mod get_album;
mod get_artist;
mod get_artists;
mod get_music_folders;
mod get_song;
mod refresh_music_folders;
mod refresh_permissions;

pub use refresh_music_folders::refresh_music_folders;
pub use refresh_permissions::refresh_permissions;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route(
            "/rest/getMusicFolders",
            get(get_music_folders::get_music_folders_handler),
        )
        .route("/rest/getArtists", get(get_artists::get_artists_handler))
        .route("/rest/getArtist", get(get_artist::get_artist_handler))
        .route("/rest/getAlbum", get(get_album::get_album_handler))
        .route("/rest/getSong", get(get_song::get_song_handler))
}
