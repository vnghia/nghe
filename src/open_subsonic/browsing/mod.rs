mod get_album;
mod get_artist;
mod get_artists;
mod get_genres;
mod get_indexes;
mod get_music_directory;
mod get_music_folders;
mod get_song;
mod refresh_music_folders;
mod refresh_permissions;

use axum::routing::get;
use axum::Router;
pub use refresh_music_folders::refresh_music_folders;
pub use refresh_permissions::refresh_permissions;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/getMusicFolders", get(get_music_folders::get_music_folders_handler))
        .route("/rest/getMusicFolders.view", get(get_music_folders::get_music_folders_handler))
        .route("/rest/getArtists", get(get_artists::get_artists_handler))
        .route("/rest/getArtists.view", get(get_artists::get_artists_handler))
        .route("/rest/getArtist", get(get_artist::get_artist_handler))
        .route("/rest/getArtist.view", get(get_artist::get_artist_handler))
        .route("/rest/getAlbum", get(get_album::get_album_handler))
        .route("/rest/getAlbum.view", get(get_album::get_album_handler))
        .route("/rest/getSong", get(get_song::get_song_handler))
        .route("/rest/getSong.view", get(get_song::get_song_handler))
        .route("/rest/getIndexes", get(get_indexes::get_indexed_handler))
        .route("/rest/getIndexes.view", get(get_indexes::get_indexed_handler))
        .route("/rest/getMusicDirectory", get(get_music_directory::get_music_directory_handler))
        .route(
            "/rest/getMusicDirectory.view",
            get(get_music_directory::get_music_directory_handler),
        )
        .route("/rest/getGenres", get(get_genres::get_genres_handler))
        .route("/rest/getGenres.view", get(get_genres::get_genres_handler))
}
