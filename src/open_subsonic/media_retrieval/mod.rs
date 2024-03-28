mod download;
mod get_cover_art;
mod stream;
mod utils;

use axum::routing::get;
use axum::{Extension, Router};

use crate::config::{ArtConfig, TranscodingConfig};

pub fn router(
    transcoding_config: TranscodingConfig,
    art_config: ArtConfig,
) -> Router<crate::Database> {
    Router::new()
        .route("/rest/download", get(download::download_handler))
        .route("/rest/download.view", get(download::download_handler))
        .route("/rest/stream", get(stream::stream_handler))
        .route("/rest/stream.view", get(stream::stream_handler))
        .route("/rest/getCoverArt", get(get_cover_art::get_cover_art_handler))
        .route("/rest/getCoverArt.view", get(get_cover_art::get_cover_art_handler))
        .layer(Extension(transcoding_config))
        .layer(Extension(art_config))
}
