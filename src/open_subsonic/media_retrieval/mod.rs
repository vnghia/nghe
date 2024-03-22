mod download;
mod stream;
mod utils;

use crate::config::TranscodingConfig;
use axum::{routing::get, Extension, Router};

pub fn router(transcoding_config: TranscodingConfig) -> Router<crate::Database> {
    Router::new()
        // download
        .route("/rest/download", get(download::download_handler))
        .route("/rest/download.view", get(download::download_handler))
        // stream
        .route("/rest/stream", get(stream::stream_handler))
        .route("/rest/stream.view", get(stream::stream_handler))
        // transcoding config
        .layer(Extension(transcoding_config))
}
