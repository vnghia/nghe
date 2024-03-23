mod download;
mod stream;
mod utils;

use axum::routing::get;
use axum::{Extension, Router};

use crate::config::TranscodingConfig;

pub fn router(transcoding_config: TranscodingConfig) -> Router<crate::Database> {
    Router::new()
        .route("/rest/download", get(download::download_handler))
        .route("/rest/download.view", get(download::download_handler))
        .route("/rest/stream", get(stream::stream_handler))
        .route("/rest/stream.view", get(stream::stream_handler))
        .layer(Extension(transcoding_config))
}
