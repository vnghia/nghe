use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use axum_extra::headers::Range;
use axum_extra::TypedHeader;
use concat_string::concat_string;
use nghe_proc_macros::add_common_validate;
use tracing::instrument;
use uuid::Uuid;

use super::download::download;
use super::utils::get_song_download_info;
use crate::config::TranscodingConfig;
use crate::models::*;
use crate::open_subsonic::StreamResponse;
use crate::utils::fs::path::hash_size_to_path;
use crate::utils::fs::{FsTrait, LocalFs, LocalPathBuf, S3Fs};
use crate::utils::song::transcode;
use crate::{Database, DatabasePool, ServerError};

add_common_validate!(StreamParams, stream);

#[instrument(skip(output_path, buffer_size))]
fn spawn_transcoding(
    input: String,
    output_path: LocalPathBuf,
    output_ext: &str,
    done_path: Option<LocalPathBuf>,
    output_bit_rate: u32,
    output_time_offset: u32,
    buffer_size: usize,
) -> StreamResponse {
    let span = tracing::Span::current();

    let (tx, rx) = flume::bounded(0);
    rayon::spawn(move || {
        let _enter = span.enter();
        tracing::debug!("start transcoding");

        let write_to_file = done_path.is_some();
        if transcode(
            input,
            &output_path,
            write_to_file,
            output_bit_rate,
            output_time_offset,
            buffer_size,
            tx,
        )
        .is_err()
        {
            if write_to_file && let Err(e) = std::fs::remove_file(&output_path) {
                tracing::error!(removing = ?e);
            }
        } else if let Some(done_path) = done_path
            && let Err(e) = std::fs::rename(&output_path, done_path)
        {
            tracing::error!(moving = ?e);
        }

        tracing::debug!("finish transcoding");
    });

    StreamResponse::from_rx(output_ext, rx)
}

async fn stream(
    pool: &DatabasePool,
    local_fs: &LocalFs,
    s3_fs: Option<&S3Fs>,
    range: Option<Range>,
    user_id: Uuid,
    params: StreamParams,
    transcoding_config: TranscodingConfig,
) -> Result<StreamResponse> {
    let format = params.format.unwrap_or(Format::Raw);
    if format == Format::Raw {
        download(pool, local_fs, s3_fs, range, user_id, params.id).await
    } else {
        // Lowest bitrate possible. Only works well with opus.
        let bit_rate = params.max_bit_rate.unwrap_or(32);
        let time_offset = params.time_offset.unwrap_or(0);
        let buffer_size = transcoding_config.buffer_size;
        let format = format.as_ref();

        let (music_folder_path, fs_type, song_relative_path, song_file_hash, song_file_size) =
            get_song_download_info(pool, user_id, params.id).await?;

        if let Some(cache_dir) = transcoding_config.cache_dir {
            // Transcoding cache is enabled
            let cache_dir = hash_size_to_path(cache_dir, song_file_hash as _, song_file_size as _)
                .join(format)
                .join(bit_rate.to_string());
            tokio::fs::create_dir_all(&cache_dir).await?;

            let done_path = cache_dir.join(concat_string!("done.", format));
            let transcoding_path = cache_dir.join(concat_string!("transcoding.", format));

            if tokio::fs::metadata(&done_path).await.is_ok() {
                if time_offset == 0 {
                    // If the song is already transcoded and time offset is 0,
                    // we will just stream the transcoded file.
                    StreamResponse::try_from_path(&done_path, None, None, true).await
                } else {
                    // If the song is already transcoded but time offset is not 0,
                    // we will use the transcoded file as input, which will active only `atrim`
                    // filter.
                    Ok(spawn_transcoding(
                        done_path.into_string(),
                        transcoding_path,
                        format,
                        None,
                        bit_rate,
                        time_offset,
                        buffer_size,
                    ))
                }
            } else {
                // If the song is not transcoded yet, spawn a new transcoding process,
                // only write to file if the file is not being transcoding,
                // i.e transcoding.format does not exist and time offset is 0.
                let is_transcoding = tokio::fs::metadata(&transcoding_path).await.is_err();
                Ok(spawn_transcoding(
                    match fs_type {
                        music_folders::FsType::Local => {
                            local_fs
                                .read_to_transcoding_input(LocalFs::join(
                                    music_folder_path,
                                    song_relative_path,
                                ))
                                .await?
                        }
                        music_folders::FsType::S3 => {
                            S3Fs::unwrap(s3_fs)?
                                .read_to_transcoding_input(S3Fs::join(
                                    music_folder_path,
                                    song_relative_path,
                                ))
                                .await?
                        }
                    },
                    transcoding_path,
                    format,
                    if is_transcoding && time_offset == 0 { Some(done_path) } else { None },
                    bit_rate,
                    time_offset,
                    buffer_size,
                ))
            }
        } else {
            Ok(spawn_transcoding(
                match fs_type {
                    music_folders::FsType::Local => {
                        local_fs
                            .read_to_transcoding_input(LocalFs::join(
                                music_folder_path,
                                song_relative_path,
                            ))
                            .await?
                    }
                    music_folders::FsType::S3 => {
                        S3Fs::unwrap(s3_fs)?
                            .read_to_transcoding_input(S3Fs::join(
                                music_folder_path,
                                song_relative_path,
                            ))
                            .await?
                    }
                },
                concat_string!("format.", format).into(),
                format,
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
    Extension(local_fs): Extension<LocalFs>,
    Extension(s3_fs): Extension<Option<S3Fs>>,
    range: Option<TypedHeader<Range>>,
    Extension(transcoding_config): Extension<TranscodingConfig>,
    req: StreamRequest,
) -> Result<StreamResponse, ServerError> {
    let range = range.map(|h| h.0);
    stream(
        &database.pool,
        &local_fs,
        s3_fs.as_ref(),
        range,
        req.user_id,
        req.params,
        transcoding_config,
    )
    .await
    .map_err(ServerError)
}

#[cfg(test)]
mod tests {
    use axum::http::header;
    use axum::response::IntoResponse;
    use strum::IntoEnumIterator;

    use super::*;
    use crate::utils::song::transcode_to_memory;
    use crate::utils::test::http::to_bytes;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_stream_raw() {
        for fs_type in music_folders::FsType::iter() {
            let mut infra = Infra::new().await.add_folder(fs_type, true).await.add_user(None).await;
            infra.add_n_song(0, 1).await.scan(.., None).await;

            let stream_bytes = to_bytes(
                stream(
                    infra.pool(),
                    infra.fs.local(),
                    infra.fs.s3_option(),
                    None,
                    infra.user_id(0),
                    StreamParams {
                        id: infra.song_ids(..).await[0],
                        max_bit_rate: None,
                        format: None,
                        time_offset: None,
                    },
                    infra.fs.transcoding_config.clone(),
                )
                .await
                .unwrap()
                .into_response(),
            )
            .await
            .to_vec();
            let raw_bytes = infra.fs.read_song(&infra.song_fs_infos(..)[0]).await;
            assert_eq!(stream_bytes, raw_bytes, "{:?} failed for streaming raw", fs_type);
        }
    }

    #[tokio::test]
    async fn test_stream_simple() {
        for fs_type in music_folders::FsType::iter() {
            let mut infra = Infra::new().await.add_folder(fs_type, true).await.add_user(None).await;
            infra.add_n_song(0, 1).await.scan(.., None).await;
            let transcode_bytes = transcode_to_memory(
                infra.fs.read_to_transcoding_input(&infra.song_fs_infos(..)[0]).await,
                Format::Opus,
                32,
                0,
                infra.fs.transcoding_config.buffer_size,
            )
            .await;

            let response = stream(
                infra.pool(),
                infra.fs.local(),
                infra.fs.s3_option(),
                None,
                infra.user_id(0),
                StreamParams {
                    id: infra.song_ids(..).await[0],
                    max_bit_rate: Some(32),
                    format: Some(Format::Opus),
                    time_offset: None,
                },
                infra.fs.transcoding_config.clone(),
            )
            .await
            .unwrap()
            .into_response();

            let headers = response.headers().clone();
            let size = headers.get(header::CONTENT_LENGTH);
            let encoding = headers.get(header::TRANSFER_ENCODING).unwrap().to_str().unwrap();
            let accept = headers.get(header::ACCEPT_RANGES).unwrap().to_str().unwrap();
            let stream_bytes = to_bytes(response).await.to_vec();

            assert!(size.is_none(), "{:?} failed for streaming simple", fs_type);
            assert_eq!(encoding, "chunked", "{:?} failed for streaming simple", fs_type);
            assert_eq!(accept, "bytes", "{:?} failed for streaming simple", fs_type);
            assert_eq!(stream_bytes, transcode_bytes, "{:?} failed for streaming simple", fs_type);
        }
    }

    #[tokio::test]
    async fn test_stream_no_cache() {
        for fs_type in music_folders::FsType::iter() {
            let mut infra = Infra::new().await.add_folder(fs_type, true).await.add_user(None).await;
            infra.add_n_song(0, 1).await.scan(.., None).await;
            let transcode_bytes = transcode_to_memory(
                infra.fs.read_to_transcoding_input(&infra.song_fs_infos(..)[0]).await,
                Format::Opus,
                32,
                0,
                infra.fs.transcoding_config.buffer_size,
            )
            .await;

            let response = stream(
                infra.pool(),
                infra.fs.local(),
                infra.fs.s3_option(),
                None,
                infra.user_id(0),
                StreamParams {
                    id: infra.song_ids(..).await[0],
                    max_bit_rate: Some(32),
                    format: Some(Format::Opus),
                    time_offset: None,
                },
                TranscodingConfig { cache_dir: None, ..Default::default() },
            )
            .await
            .unwrap()
            .into_response();

            let headers = response.headers().clone();
            let size = headers.get(header::CONTENT_LENGTH);
            let encoding = headers.get(header::TRANSFER_ENCODING).unwrap().to_str().unwrap();
            let accept = headers.get(header::ACCEPT_RANGES).unwrap().to_str().unwrap();
            let stream_bytes = to_bytes(response).await.to_vec();

            assert!(size.is_none(), "{:?} failed for streaming no cache", fs_type);
            assert_eq!(encoding, "chunked", "{:?} failed for streaming no cache", fs_type);
            assert_eq!(accept, "bytes", "{:?} failed for streaming no cache", fs_type);
            assert_eq!(
                stream_bytes, transcode_bytes,
                "{:?} failed for streaming no cache",
                fs_type
            );
        }
    }
}
