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
        // get_music_folders
        .route(
            "/rest/getMusicFolders",
            get(get_music_folders::get_music_folders_handler),
        )
        .route(
            "/rest/getMusicFolders.view",
            get(get_music_folders::get_music_folders_handler),
        )
        // get_artists
        .route("/rest/getArtists", get(get_artists::get_artists_handler))
        .route(
            "/rest/getArtists.view",
            get(get_artists::get_artists_handler),
        )
        // get_artist
        .route("/rest/getArtist", get(get_artist::get_artist_handler))
        .route("/rest/getArtist.view", get(get_artist::get_artist_handler))
        // get_album
        .route("/rest/getAlbum", get(get_album::get_album_handler))
        .route("/rest/getAlbum.view", get(get_album::get_album_handler))
        // get_song
        .route("/rest/getSong", get(get_song::get_song_handler))
        .route("/rest/getSong.view", get(get_song::get_song_handler))
}
