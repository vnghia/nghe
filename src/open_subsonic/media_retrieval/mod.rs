mod download;
mod stream;
mod utils;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        // download
        .route("/rest/download", get(download::download_handler))
        .route("/rest/download.view", get(download::download_handler))
        // stream
        .route("/rest/stream", get(stream::stream_handler))
        .route("/rest/stream.view", get(stream::stream_handler))
}
