mod create_playlist;
mod get_playlist;
mod get_playlists;
mod id3;
mod utils;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/createPlaylist", get(create_playlist::create_playlist_handler))
        .route("/rest/createPlaylist.view", get(create_playlist::create_playlist_handler))
        .route("/rest/getPlaylists", get(get_playlists::get_playlists_handler))
        .route("/rest/getPlaylists.view", get(get_playlists::get_playlists_handler))
        .route("/rest/getPlaylist", get(get_playlist::get_playlist_handler))
        .route("/rest/getPlaylist.view", get(get_playlist::get_playlist_handler))
}
