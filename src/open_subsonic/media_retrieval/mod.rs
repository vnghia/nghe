mod download;
mod get_cover_art;
mod get_lyrics_by_song_id;
mod stream;
mod utils;

use axum::Extension;

use crate::config::{ArtConfig, TranscodingConfig};
use crate::utils::fs::{LocalFs, S3Fs};

pub fn router(
    local_fs: LocalFs,
    s3_fs: Option<S3Fs>,
    transcoding_config: TranscodingConfig,
    art_config: ArtConfig,
) -> axum::Router<crate::Database> {
    nghe_proc_macros::build_router!(download, stream, get_cover_art, get_lyrics_by_song_id)
        .layer(Extension(local_fs))
        .layer(Extension(s3_fs))
        .layer(Extension(transcoding_config))
        .layer(Extension(art_config))
}
