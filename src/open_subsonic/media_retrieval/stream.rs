use std::path::PathBuf;

use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use concat_string::concat_string;
use nghe_proc_macros::add_validate;
use tracing::instrument;
use uuid::Uuid;

use super::download::download;
use super::utils::get_song_download_info;
use crate::config::TranscodingConfig;
use crate::open_subsonic::StreamResponse;
use crate::utils::fs::path::hash_to_path;
use crate::utils::song::transcode;
use crate::{Database, DatabasePool, ServerError};

#[add_validate]
#[derive(Debug)]
pub struct StreamParams {
    id: Uuid,
    max_bit_rate: Option<u32>,
    format: Option<String>,
    time_offset: Option<u32>,
}

#[instrument(skip(output_path, buffer_size))]
fn spawn_transcoding(
    input_path: PathBuf,
    output_path: PathBuf,
    output_ext: &str,
    done_path: Option<PathBuf>,
    output_bit_rate: u32,
    output_time_offset: u32,
    buffer_size: usize,
) -> StreamResponse {
    let span = tracing::Span::current();

    let (tx, rx) = crossfire::mpsc::bounded_tx_blocking_rx_future(1);
    tokio::task::spawn_blocking(move || {
        let _enter = span.enter();
        tracing::debug!("start transcoding");

        let write_to_file = done_path.is_some();
        if transcode(
            &input_path,
            &output_path,
            write_to_file,
            output_bit_rate,
            output_time_offset,
            buffer_size,
            tx,
        )
        .is_err()
        {
            if write_to_file && std::fs::remove_file(&output_path).is_err() {
                tracing::error!("could not remove transcoding temporary cache")
            }
        } else if let Some(done_path) = done_path
            && std::fs::rename(&output_path, done_path).is_err()
        {
            tracing::error!("could not move transcoding temporary cache to final cache")
        }

        tracing::debug!("finish transcoding");
    });

    StreamResponse::from_rx(output_ext, rx)
}

async fn stream(
    pool: &DatabasePool,
    user_id: Uuid,
    params: StreamParams,
    transcoding_config: TranscodingConfig,
) -> Result<StreamResponse> {
    let format = params.format.unwrap_or("raw".to_owned());
    if format == "raw" {
        download(pool, user_id, params.id).await
    } else {
        // Lowest bitrate possible. Only works well with opus.
        let bit_rate = params.max_bit_rate.unwrap_or(32);
        let time_offset = params.time_offset.unwrap_or(0);
        let buffer_size = transcoding_config.buffer_size;

        let (absolute_path, song_file_hash) =
            get_song_download_info(pool, user_id, params.id).await?;

        if let Some(cache_path) = transcoding_config.cache_path {
            // Transcoding cache is enabled
            let cache_dir =
                hash_to_path(cache_path, song_file_hash).join(&format).join(bit_rate.to_string());
            tokio::fs::create_dir_all(&cache_dir).await?;

            let done_path = cache_dir.join(concat_string!("done.", &format));
            let transcoding_path = cache_dir.join(concat_string!("transcoding.", &format));

            if tokio::fs::metadata(&done_path).await.is_ok() {
                if time_offset == 0 {
                    // If the song is already transcoded and time offset is 0,
                    // we will just stream the transcoded file.
                    StreamResponse::try_from_path(&done_path).await
                } else {
                    // If the song is already transcoded but time offset is not 0,
                    // we will use the transcoded file as input, which will active only `atrim`
                    // filter.
                    Ok(spawn_transcoding(
                        done_path,
                        transcoding_path,
                        &format,
                        None,
                        bit_rate,
                        time_offset,
                        buffer_size,
                    ))
                }
            } else if tokio::fs::metadata(&transcoding_path).await.is_ok() && time_offset == 0 {
                // if the song is being transcoding and time offset is 0,
                // we will stream the file, which is being written to by another process.
                StreamResponse::try_from_path(&transcoding_path).await
            } else {
                // If the song is being transcoding but time offset is not 0,
                // or it is not being transcoding, spawn a new transcoding processs,
                // and only write to output if time offset is 0.
                Ok(spawn_transcoding(
                    absolute_path,
                    transcoding_path,
                    &format,
                    if time_offset == 0 { Some(done_path) } else { None },
                    bit_rate,
                    time_offset,
                    buffer_size,
                ))
            }
        } else {
            Ok(spawn_transcoding(
                absolute_path,
                concat_string!("format.", &format).into(),
                &format,
                None,
                bit_rate,
                time_offset,
                buffer_size,
            ))
        }
    }
}

pub async fn stream_handler(
    State(database): State<Database>,
    Extension(transcoding_config): Extension<TranscodingConfig>,
    req: StreamRequest,
) -> Result<StreamResponse, ServerError> {
    stream(&database.pool, req.user_id, req.params, transcoding_config).await.map_err(ServerError)
}
