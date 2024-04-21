mod download;
mod get_cover_art;
mod get_lyrics_by_song_id;
mod stream;
mod utils;

use axum::Extension;

use crate::config::{ArtConfig, TranscodingConfig};

pub fn router(
    transcoding_config: TranscodingConfig,
    art_config: ArtConfig,
) -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(download, stream, get_cover_art, get_lyrics_by_song_id)
        .layer(Extension(transcoding_config))
        .layer(Extension(art_config))
}
