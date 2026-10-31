use axum_extra::headers::Range;
pub use nghe_api::route::media_retrieval::stream::{Format, Request};
use nghe_proc_macro::handler;
use uuid::Uuid;

use super::download;
use crate::database::Database;
use crate::file::audio::transcode;
use crate::filesystem::{Filesystem, Trait as _, local};
use crate::http::binary;
use crate::http::header::ToOffset;
#[cfg(test)]
use crate::test::binary::Status as BinaryStatus;
use crate::{Error, config};

#[handler]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    #[handler(header)] range: Option<Range>,
    config: config::Transcode,
    user_id: Uuid,
    request: Request,
) -> Result<binary::Response, Error> {
    let audio = binary::Source::audio(database, filesystem, user_id, request.id).await?;
    let filesystem = audio.filesystem;
    let source = audio.source;
    let bit_rate = audio.bit_rate;

    let size_offset =
        range.map(|range| range.to_offset(source.property.size.into())).transpose()?;

    let max_bit_rate = request.max_bit_rate;
    let time_offset = request.time_offset.unwrap_or(0);

    let source_format = source.property.format;
    let sink_format = match request.format.unwrap_or_default() {
        Format::Raw => {
            if time_offset == 0 {
                return download::handler_impl(filesystem, source, size_offset).await;
            }
            source_format.into()
        }
        Format::Transcode(format) => {
            if time_offset == 0
                && source_format == format
                && max_bit_rate.is_none_or(|max_bit_rate| max_bit_rate >= bit_rate)
            {
                return download::handler_impl(filesystem, source, size_offset).await;
            }
            format
        }
    };
    let property = source.property.replace(sink_format);
    let source_path = source.path.to_path();

    let max_bit_rate = max_bit_rate.unwrap_or(32);

    let transcode_args = if let Some(ref cache_dir) = config.cache_dir {
        let output = property.path_create_dir(cache_dir, max_bit_rate.to_string()).await?;
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
                    local::Filesystem::read_to_read_platform(output, None).await?.into(),
                    None,
                    #[cfg(test)]
                    BinaryStatus::UseCachedOutput,
                )
            } else {
                return binary::Response::from_path(
                    output,
                    sink_format,
                    size_offset,
                    #[cfg(test)]
                    BinaryStatus::ServeCachedOutput,
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
                filesystem.read_to_read(source_path, None).await?,
                if time_offset > 0 { None } else { Some(output) },
                #[cfg(test)]
                if time_offset > 0 { BinaryStatus::NoCache } else { BinaryStatus::WithCache },
            )
        }
    } else {
        (
            filesystem.read_to_read(source_path, None).await?,
            None,
            #[cfg(test)]
            BinaryStatus::NoCache,
        )
    };

    let (rx, _) = transcode::Transcoder::spawn(
        &config,
        transcode::source::Args { format: source_format, source: transcode_args.0 },
        transcode::sink::Args { format: sink_format, cache: transcode_args.1 },
        max_bit_rate,
        time_offset,
    );

    binary::Response::from_rx(
        rx,
        sink_format,
        #[cfg(test)]
        transcode_args.2,
    )
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use axum::http::StatusCode;
    use axum_extra::headers::HeaderMapExt;
    use nghe_api::{filesystem, format};
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::binary::Header as BinaryHeader;
    use crate::test::{Mock, assets, mock};

    async fn spawn_stream(
        mock: &Mock,
        n_task: usize,
        user_id: Uuid,
        request: Request,
    ) -> Vec<(StatusCode, Vec<u8>, Option<BinaryStatus>)> {
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
        stream_set
            .join_all()
            .await
            .into_iter()
            .map(|(status, headers, body)| {
                (status, body, headers.typed_get::<BinaryHeader>().map(|header| header.0))
            })
            .collect()
    }

    #[rstest]
    #[tokio::test]
    async fn test_download(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
        #[values(false, true)] raw: bool,
        #[values(None, Some(10_000))] max_bit_rate: Option<u32>,
        #[values(None, Some(10))] time_offset: Option<u32>,
    ) {
        mock.add_music_folder().ty(ty).call().await;
        let mut music_folder = mock.music_folder(0).await;
        let source_format = audio::Format::Flac;
        music_folder.add_audio_filesystem::<&str>().format(source_format).call().await;

        let user_id = mock.user_id(0).await;
        let song_id = music_folder.song_id_filesystem(0).await;
        let sink_format: format::Transcode = source_format.into();

        let request = Request {
            id: song_id,
            max_bit_rate,
            format: if raw { Some(Format::Raw) } else { Some(sink_format.into()) },
            time_offset,
        };

        let time_offset = time_offset.unwrap_or(0);
        let original =
            music_folder.to_impl().read(music_folder.absolute_path(0).to_path()).await.unwrap();

        for (status, body, binary_status) in spawn_stream(&mock, 2, user_id, request).await {
            assert_eq!(status, StatusCode::OK);
            if time_offset == 0 {
                assert_eq!(body, original);
            } else {
                assert!(!body.is_empty());
            }
            assert_eq!(
                binary_status,
                if time_offset == 0 { None } else { Some(BinaryStatus::NoCache) }
            );
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_stream(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
        #[values(None, Some(10))] time_offset: Option<u32>,
    ) {
        mock.add_music_folder().ty(ty).call().await;
        let mut music_folder = mock.music_folder(0).await;
        let source_format = audio::Format::Flac;
        music_folder.add_audio_filesystem::<&str>().format(source_format).call().await;

        let config = &mock.config.transcode;
        let user_id = mock.user_id(0).await;
        let song_id = music_folder.song_id_filesystem(0).await;
        let sink_format = format::Transcode::Opus;
        let bit_rate = 32;

        let request = Request {
            id: song_id,
            max_bit_rate: Some(bit_rate),
            format: Some(sink_format.into()),
            time_offset,
        };

        let time_offset = time_offset.unwrap_or(0);
        let transcoded = {
            let path = music_folder.absolute_path(0);
            transcode::Transcoder::spawn_collect(
                config,
                transcode::source::Args {
                    format: source_format,
                    source: music_folder
                        .to_impl()
                        .read_to_read(path.to_path(), None)
                        .await
                        .unwrap(),
                },
                transcode::sink::Args { format: sink_format, cache: None },
                bit_rate,
                time_offset,
            )
            .await
        };

        for (status, body, binary_status) in spawn_stream(&mock, 2, user_id, request).await {
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body, transcoded);
            if ty != filesystem::Type::Local {
                let local_transcoded = transcode::Transcoder::spawn_collect(
                    config,
                    transcode::source::Args {
                        format: source_format,
                        source: assets::source(assets::path(source_format)).await,
                    },
                    transcode::sink::Args { format: sink_format, cache: None },
                    bit_rate,
                    time_offset,
                )
                .await;
                assert_eq!(body, local_transcoded);
            }
            assert_eq!(
                binary_status,
                Some(if time_offset == 0 {
                    BinaryStatus::WithCache
                } else {
                    BinaryStatus::NoCache
                })
            );
        }

        if time_offset > 0 {
            for (_, _, binary_status) in
                spawn_stream(&mock, 1, user_id, Request { time_offset: None, ..request }).await
            {
                assert_eq!(binary_status, Some(BinaryStatus::WithCache));
            }
        }

        for (status, body, binary_status) in spawn_stream(&mock, 2, user_id, request).await {
            assert_eq!(status, StatusCode::OK);
            if time_offset == 0 {
                assert_eq!(body, transcoded);
            } else {
                assert!(!body.is_empty());
            }
            assert_eq!(
                binary_status,
                Some(if time_offset == 0 {
                    BinaryStatus::ServeCachedOutput
                } else {
                    BinaryStatus::UseCachedOutput
                })
            );
        }
    }
}
