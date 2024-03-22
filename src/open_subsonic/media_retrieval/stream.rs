use super::{download::download, utils::get_song_download_info};
use crate::{
    config::TranscodingConfig, open_subsonic::StreamResponse, utils::song::transcode, Database,
    DatabasePool, ServerError,
};

use anyhow::Result;
use axum::{extract::State, Extension};
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
    user_id: Uuid,
    params: StreamParams,
    transcoding_config: TranscodingConfig,
) -> Result<StreamResponse> {
    let format = params.format.unwrap_or("raw".to_owned());
    if format == "raw" {
        return download(pool, user_id, params.id).await;
    }

    // Lowest bitrate possible. Only works well with opus.
    let max_bit_rate = params.max_bit_rate.unwrap_or(32) * 1000;

    let absolute_path = get_song_download_info(pool, user_id, params.id).await?;
    // ffmpeg requires a filename with extension
    let output_path = concat_string!("output.", &format);

    let (tx, rx) = crossfire::mpsc::bounded_tx_blocking_rx_future(1);

    tokio::task::spawn_blocking(move || {
        let input_path = absolute_path.to_str().expect("non utf-8 path encountered");
        let output_path = output_path.as_str();

        tracing::debug!(
            "start transcoding {} to {} with bitrate {}",
            input_path,
            output_path,
            max_bit_rate
        );
        if let Err(e) = transcode(
            &CString::new(input_path).unwrap(),
            &CString::new(output_path).unwrap(),
            max_bit_rate,
            params.time_offset,
            transcoding_config.buffer_size,
            tx,
        ) {
            tracing::debug!(
                "can not transcode {} to {} with bitrate {} because of {:?}",
                input_path,
                output_path,
                max_bit_rate,
                e
            );
        }
    });

    Ok(StreamResponse::from_rx(&format, rx))
}

pub async fn stream_handler(
    State(database): State<Database>,
    Extension(transcoding_config): Extension<TranscodingConfig>,
    req: StreamRequest,
) -> Result<StreamResponse, ServerError> {
    stream(&database.pool, req.user_id, req.params, transcoding_config)
        .await
        .map_err(ServerError)
}
