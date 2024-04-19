mod create_playlist;
mod id3;
mod utils;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/createPlaylist", get(create_playlist::create_playlist_handler))
        .route("/rest/createPlaylist.view", get(create_playlist::create_playlist_handler))
}
