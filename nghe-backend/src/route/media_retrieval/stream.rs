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
use crate::{Error, config, transcode};

#[handler]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    #[handler(header)] range: Option<Range>,
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
    let source_path = source.path.to_path();

    let transcode_args = if let Some(ref cache_dir) = config.cache_dir {
        let output = property.path_create_dir(cache_dir, bitrate.to_string().as_str()).await?;
        let cache_exists = tokio::fs::try_exists(&output).await?;

        // If the cache exists, it means that the transcoding process is finish. Since we write the
        // transcoding cache atomically, we are guaranteed that that file is in a complete state and
        // is usable immediately. In that case, we have two cases:
        //  - If time offset is greater than 0, we can use the transcoded file as transcoder input
        //    so it only needs to activate `atrim` filter.
        //  - Otherwise, we only need to stream the transcoded file from local cache.
        if cache_exists {
            if time_offset > 0 {
                (
                    transcode::Path { input: output.as_str().to_owned(), output: None },
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
            // If the file does not exist, we have two cases:
            //  - If time offset is greater than 0, we spawn a transcoding process without writing
            //    it back to the local cache.
            //  - Otherwise, we spawn a transcoding process and let the sink writes the transcoded
            //    chunk to the cache file.
            (
                transcode::Path {
                    input: filesystem.transcode_input(source_path).await?,
                    output: if time_offset > 0 { None } else { Some(output) },
                },
                #[cfg(test)]
                if time_offset > 0 { TranscodeStatus::NoCache } else { TranscodeStatus::WithCache },
            )
        }
    } else {
        (
            transcode::Path { input: filesystem.transcode_input(source_path).await?, output: None },
            #[cfg(test)]
            TranscodeStatus::NoCache,
        )
    };

    let (rx, _) =
        transcode::Transcoder::spawn(&config, transcode_args.0, format, bitrate, time_offset);

    binary::Response::from_rx(
        rx,
        format,
        #[cfg(test)]
        transcode_args.1,
    )
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use axum::http::StatusCode;
    use axum_extra::headers::HeaderMapExt;
    use itertools::Itertools;
    use nghe_api::common::{filesystem, format};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::transcode::{Header as TranscodeHeader, Status as TranscodeStatus};
    use crate::test::{Mock, mock};

    async fn spawn_stream(
        mock: &Mock,
        n_task: usize,
        user_id: Uuid,
        request: Request,
    ) -> (Vec<(StatusCode, Vec<u8>)>, Vec<TranscodeStatus>) {
        let mut stream_set = tokio::task::JoinSet::new();
        for _ in 0..n_task {
            let database = mock.database().clone();
            let filesystem = mock.filesystem().clone();
            let config = mock.config.transcode.clone();
            stream_set.spawn(async move {
                handler(&database, &filesystem, None, config, user_id, request)
                    .await
                    .unwrap()
                    .extract()
                    .await
            });
        }
        let (responses, transcode_status): (Vec<_>, Vec<_>) = stream_set
            .join_all()
            .await
            .into_iter()
            .map(|(status, headers, body)| {
                ((status, body), headers.typed_get::<TranscodeHeader>().unwrap().0)
            })
            .unzip();
        (responses, transcode_status.into_iter().sorted().collect())
    }

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

        let config = &mock.config.transcode;
        let user_id = mock.user_id(0).await;
        let song_id = music_folder.song_id_filesystem(0).await;
        let format = format::Transcode::Opus;
        let bitrate = 32;

        let transcoded = {
            let path = music_folder.absolute_path(0);
            let input = music_folder.to_impl().transcode_input(path.to_path()).await.unwrap();
            transcode::Transcoder::spawn_collect(config, &input, format, bitrate, 0).await
        };

        let request = Request {
            id: song_id,
            max_bit_rate: Some(bitrate),
            format: Some(format.into()),
            time_offset: None,
        };

        let (responses, transcode_status) = spawn_stream(&mock, 2, user_id, request).await;
        for (status, body) in responses {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(transcoded, body);
        }
        assert_eq!(transcode_status, &[TranscodeStatus::WithCache, TranscodeStatus::WithCache]);

        let (responses, transcode_status) = spawn_stream(&mock, 2, user_id, request).await;
        for (status, body) in responses {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(transcoded, body);
        }
        assert_eq!(
            transcode_status,
            &[TranscodeStatus::ServeCachedOutput, TranscodeStatus::ServeCachedOutput]
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_stream_time_offset(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
    ) {
        mock.add_music_folder().ty(ty).call().await;
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().format(audio::Format::Flac).call().await;

        let user_id = mock.user_id(0).await;
        let song_id = music_folder.song_id_filesystem(0).await;
        let config = &mock.config.transcode;
        let format = format::Transcode::Opus;
        let bitrate = 32;
        let time_offset = 10;

        let transcoded = {
            let path = music_folder.absolute_path(0);
            let input = music_folder.to_impl().transcode_input(path.to_path()).await.unwrap();
            transcode::Transcoder::spawn_collect(config, &input, format, bitrate, time_offset).await
        };

        let request = Request {
            id: song_id,
            max_bit_rate: Some(bitrate),
            format: Some(format.into()),
            time_offset: Some(time_offset),
        };

        let (responses, transcode_status) = spawn_stream(&mock, 2, user_id, request).await;
        for (status, body) in responses {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(transcoded, body);
        }
        assert_eq!(transcode_status, &[TranscodeStatus::NoCache, TranscodeStatus::NoCache]);

        let transcode_status =
            spawn_stream(&mock, 1, user_id, Request { time_offset: None, ..request }).await.1;
        assert_eq!(transcode_status, &[TranscodeStatus::WithCache]);

        let (responses, transcode_status) = spawn_stream(&mock, 2, user_id, request).await;
        for (status, body) in responses {
            assert_eq!(status, StatusCode::OK);
            assert!(!body.is_empty());
        }
        assert_eq!(
            transcode_status,
            &[TranscodeStatus::UseCachedOutput, TranscodeStatus::UseCachedOutput]
        );
    }
}
