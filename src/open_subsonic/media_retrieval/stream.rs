use super::{download::download, utils::get_song_stream_info, ServeMusicFolders};
use crate::{
    open_subsonic::StreamResponse, utils::song::transcode, Database, DatabasePool, ServerError,
};

use anyhow::Result;
use axum::{body::Body, extract::State, http::header, response::IntoResponse, Extension};
use concat_string::concat_string;
use nghe_proc_macros::add_validate;
use std::ffi::CString;
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct StreamParams {
    id: Uuid,
    max_bit_rate: Option<u32>,
    format: Option<String>,
    time_offset: Option<u32>,
}

async fn stream(
    pool: &DatabasePool,
    serve_music_folders: &mut ServeMusicFolders,
    user_id: Uuid,
    params: StreamParams,
) -> Result<impl IntoResponse> {
    let format = params.format.unwrap_or("raw".to_owned());
    if format == "raw" {
        return Ok(download(pool, serve_music_folders, user_id, params.id)
            .await?
            .into_response());
    }

    // Lowest bitrate possible. Only works well with opus.
    let max_bit_rate = params.max_bit_rate.unwrap_or(32) * 1000;

    let (absolute_path, _) = get_song_stream_info(pool, user_id, params.id).await?;
    // ffmpeg requires a filename with extension
    let output_path = concat_string!("output.", format);

    let (tx, rx) = crossfire::mpsc::bounded_tx_blocking_rx_future(1);

    tokio::task::spawn_blocking(move || {
        transcode(
            &CString::new(absolute_path.to_str().expect("non utf-8 path encountered")).unwrap(),
            &CString::new(output_path).unwrap(),
            max_bit_rate,
            params.time_offset,
            tx,
        )
    });
    tracing::debug!("spawned a new task for transcoding");

    Ok((
        [(
            header::CONTENT_TYPE,
            mime_guess::from_ext(&format)
                .first_or_octet_stream()
                .essence_str()
                .to_owned(),
        )],
        Body::from_stream(StreamResponse::new(rx)),
    )
        .into_response())
}

pub async fn stream_handler(
    State(database): State<Database>,
    Extension(mut serve_music_folders): Extension<ServeMusicFolders>,
    req: StreamRequest,
) -> Result<impl IntoResponse, ServerError> {
    stream(
        &database.pool,
        &mut serve_music_folders,
        req.user_id,
        req.params,
    )
    .await
    .map_err(ServerError)
}
