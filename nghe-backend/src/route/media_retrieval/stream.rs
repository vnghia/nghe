use std::ffi::CString;

use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::stream::{Format, Request};
use nghe_proc_macro::handler;
use uuid::Uuid;

use super::download;
use crate::database::Database;
use crate::filesystem::{Filesystem, Trait};
use crate::http::binary;
use crate::http::header::ToOffset;
use crate::{config, transcode, Error};

#[handler(role = stream, headers = [range])]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    range: Option<Range>,
    config: config::Transcode,
    user_id: Uuid,
    request: Request,
) -> Result<binary::Response, Error> {
    let (filesystem, source) =
        binary::Source::audio(database, filesystem, user_id, request.id).await?;
    let size_offset =
        range.map(|range| range.to_offset(source.property.size.into())).transpose()?;

    let bitrate = request.max_bit_rate.unwrap_or(32);
    let time_offset = request.time_offset.unwrap_or(0);

    let format = match request.format.unwrap_or_default() {
        Format::Raw => return download::handler_impl(filesystem, source, size_offset).await,
        Format::Transcode(format) => format,
    };
    let property = source.property.replace(format);

    let (input, output) = if let Some(ref cache_dir) = config.cache_dir {
        let output = property.path(cache_dir, bitrate.to_string().as_str());
        let (can_acquire_lock, output) =
            tokio::task::spawn_blocking(move || (transcode::Lock::read(&output).is_ok(), output))
                .await?;

        // If local cache is turned on and we can acquire the read lock, it means that:
        //  - The file exists.
        //  - No process is writing to it. The transcoding process is finish.
        //
        // In that case, we have two cases:
        //  - if time offset is greater than 0, we can use the transcoded file as transcoder input
        //    so it only needs to activate `atrim` filter.
        //  - otherwise, we only need to stream the transcoded file from local cache.
        // If the lock can not be acquired, we have two cases:
        //  - if time offset is greater than 0, we spawn a transcoding process without writing it
        //    back to the local cache.
        //  - otherwise, we spawn a transcoding process and let the sink tries acquiring the write
        //    lock for further processing.
        if can_acquire_lock {
            if time_offset > 0 {
                (CString::new(output.as_str())?, None)
            } else {
                return binary::Response::from_path(output, format, size_offset).await;
            }
        } else {
            (
                filesystem.transcode_input(source.path.to_path()).await?,
                if time_offset > 0 { None } else { Some(output) },
            )
        }
    } else {
        (filesystem.transcode_input(source.path.to_path()).await?, None)
    };

    let (sink, rx) = transcode::Sink::new(&config, format, output);
    transcode::Transcoder::spawn(&input, sink, bitrate, time_offset)?;

    binary::Response::from_rx(rx, format)
}
