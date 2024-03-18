use std::ffi::CString;

use super::{download::download, utils::get_song_download_info};
use crate::{
    open_subsonic::common::binary_response::BinaryResponse, utils::song::transcode, Database,
    DatabasePool, ServerError,
};

use anyhow::Result;
use axum::extract::State;
use concat_string::concat_string;
use nghe_proc_macros::add_validate;
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
) -> Result<BinaryResponse> {
    let format = params.format.unwrap_or("opus".to_owned());
    if format == "raw" {
        return download(pool, user_id, params.id).await;
    }

    // Lowest bitrate possible. Only works well with opus.
    let max_bit_rate = params.max_bit_rate.unwrap_or(32000);

    let (absolute_path, _) = get_song_download_info(pool, user_id, params.id).await?;
    // ffmpeg requires a filename with extension
    let output_path = concat_string!("output.", format);

    let data = tokio::task::spawn_blocking(move || {
        transcode(
            &CString::new(absolute_path.to_str().expect("non utf-8 path encountered")).unwrap(),
            &CString::new(output_path).unwrap(),
            max_bit_rate,
            params.time_offset,
        )
    })
    .await??;

    Ok(BinaryResponse { format, data })
}

pub async fn stream_handler(
    State(database): State<Database>,
    req: StreamRequest,
) -> Result<BinaryResponse, ServerError> {
    stream(&database.pool, req.user_id, req.params)
        .await
        .map_err(ServerError)
}
