use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::stream::{Format, Request};
use nghe_proc_macro::handler;
use uuid::Uuid;

use super::download;
use crate::database::Database;
use crate::filesystem::{Filesystem, Trait};
use crate::http::binary;
use crate::http::header::ToOffset;
#[cfg(test)]
use crate::test::transcode::Status as TranscodeStatus;
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

    let transcode_config = if let Some(ref cache_dir) = config.cache_dir {
        let output = property.path_create_dir(cache_dir, bitrate.to_string().as_str()).await?;

        let span = tracing::Span::current();
        let (can_acquire_lock, output) = tokio::task::spawn_blocking(move || {
            let _entered = span.enter();
            (transcode::Lock::read(&output).is_ok(), output)
        })
        .await?;

        // If local cache is turned on and we can acquire the read lock, it means that:
        //  - The file exists.
        //  - No process is writing to it. The transcoding process is finish.
        //
        // In that case, we have two cases:
        //  - If time offset is greater than 0, we can use the transcoded file as transcoder input
        //    so it only needs to activate `atrim` filter.
        //  - Otherwise, we only need to stream the transcoded file from local cache.
        // If the lock can not be acquired, we have two cases:
        //  - If time offset is greater than 0, we spawn a transcoding process without writing it
        //    back to the local cache.
        //  - Otherwise, we spawn a transcoding process and let the sink tries acquiring the write
        //    lock for further processing.
        if can_acquire_lock {
            if time_offset > 0 {
                (
                    output.as_str().to_owned(),
                    None,
                    #[cfg(test)]
                    TranscodeStatus::UseCachedOutput,
                )
            } else {
                return binary::Response::from_path(
                    output,
                    format,
                    size_offset,
                    #[cfg(test)]
                    TranscodeStatus::ServeCachedOutput,
                )
                .await;
            }
        } else {
            (
                filesystem.transcode_input(source.path.to_path()).await?,
                if time_offset > 0 { None } else { Some(output) },
                #[cfg(test)]
                if time_offset > 0 { TranscodeStatus::NoCache } else { TranscodeStatus::WithCache },
            )
        }
    } else {
        (
            filesystem.transcode_input(source.path.to_path()).await?,
            None,
            #[cfg(test)]
            TranscodeStatus::NoCache,
        )
    };

    let input = transcode_config.0;
    let output = transcode_config.1;

    let (sink, rx) = transcode::Sink::new(&config, format, output).await?;
    #[cfg(test)]
    let transcode_status = sink.status(transcode_config.2);
    transcode::Transcoder::spawn(input, sink, bitrate, time_offset)?;

    binary::Response::from_rx(
        rx,
        format,
        #[cfg(test)]
        transcode_status,
    )
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_extra::headers::HeaderMapExt;
    use itertools::Itertools;
    use nghe_api::common::{filesystem, format};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_stream(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
    ) {
        mock.add_music_folder().ty(ty).call().await;
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().format(audio::Format::Flac).call().await;

        let user_id = mock.user(0).await.user.id;
        let song_id = music_folder.query_id(0).await;
        let mut stream_set = tokio::task::JoinSet::new();

        let database = mock.database();
        let filesystem = mock.filesystem();
        let config = mock.config.transcode.clone();
        let format = format::Transcode::Opus;
        let bitrate = 32;
        let time_offset = 0;

        let transcoded = {
            let path = music_folder.absolute_path(0);
            let input = music_folder.to_impl().transcode_input(path.to_path()).await.unwrap();
            transcode::Transcoder::spawn_collect(&input, &config, format, bitrate, time_offset)
                .await
        };

        let request = Request {
            id: song_id,
            max_bit_rate: Some(bitrate),
            format: Some(format.into()),
            time_offset: Some(time_offset),
        };

        for _ in 0..2 {
            let database = database.clone();
            let filesystem = filesystem.clone();
            let config = config.clone();
            stream_set.spawn(async move {
                handler(&database, &filesystem, None, config, user_id, request)
                    .await
                    .unwrap()
                    .extract()
                    .await
            });
        }

        let responses = stream_set.join_all().await;
        let mut transcode_status = vec![];
        assert_eq!(responses.len(), 2);
        for (status, headers, body) in responses.into_iter().take(2) {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(transcoded, body);
            transcode_status.push(headers.typed_get::<crate::test::transcode::Header>().unwrap().0);
        }
        assert_eq!(
            transcode_status.into_iter().sorted().collect_vec(),
            &[TranscodeStatus::NoCache, TranscodeStatus::WithCache]
        );

        let (status, headers, body) = handler(database, filesystem, None, config, user_id, request)
            .await
            .unwrap()
            .extract()
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            headers.typed_get::<crate::test::transcode::Header>().unwrap().0,
            TranscodeStatus::ServeCachedOutput
        );
        assert_eq!(transcoded, body);
    }
}
