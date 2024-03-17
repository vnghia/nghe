use std::ffi::CString;

use super::{download::download, utils::get_song_absolute_path};
use crate::{utils::song::transcode, Database, DatabasePool, ServerError};

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

async fn stream(pool: &DatabasePool, user_id: Uuid, params: StreamParams) -> Result<Vec<u8>> {
    let format = params.format.unwrap_or("opus".to_owned());
    if format == "raw" {
        return download(pool, user_id, params.id).await;
    }

    // Lowest bitrate possible. Only works well with opus.
    let max_bit_rate = params.max_bit_rate.unwrap_or(32000);

    let song_absolute_path = get_song_absolute_path(pool, user_id, params.id).await?;
    // ffmpeg requires a filename with extension
    let output_path = concat_string!("output.", format);

    tokio::task::spawn_blocking(move || {
        transcode(
            &CString::new(
                song_absolute_path
                    .to_str()
                    .expect("non utf-8 path encountered"),
            )
            .unwrap(),
            &CString::new(output_path).unwrap(),
            max_bit_rate,
            params.time_offset,
        )
    })
    .await?
}

pub async fn stream_handler(
    State(database): State<Database>,
    req: StreamRequest,
) -> Result<Vec<u8>, ServerError> {
    stream(&database.pool, req.user.id, req.params)
        .await
        .map_err(ServerError)
}
