use crate::models::*;

mod download;
mod serve_music_folder;
mod stream;
mod utils;

use axum::{routing::get, Extension, Router};
pub use serve_music_folder::{
    ServeMusicFolder, ServeMusicFolderResponse, ServeMusicFolderResult, ServeMusicFolders,
};

pub fn router(music_folders: Vec<music_folders::MusicFolder>) -> Router<crate::Database> {
    let serve_music_folders = ServeMusicFolder::new(music_folders);

    Router::new()
        // download
        .route("/rest/download", get(download::download_handler))
        .route("/rest/download.view", get(download::download_handler))
        // stream
        .route("/rest/stream", get(stream::stream_handler))
        .route("/rest/stream.view", get(stream::stream_handler))
        .layer(Extension(serve_music_folders))
}
