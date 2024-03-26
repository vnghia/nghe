use std::io::Cursor;
use std::path::Path;

use anyhow::Result;
use diesel::dsl::{exists, not};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{StreamExt, TryStreamExt};
use futures_buffered::FuturesUnorderedBounded;
use lofty::FileType;
use tracing::instrument;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use super::album::{upsert_album, upsert_song_album_artists};
use super::artist::upsert_artists;
use super::run_scan::ScanStatistic;
use super::song::{insert_song, update_song, upsert_song_artists};
use crate::config::parsing::ParsingConfig;
use crate::config::ScanConfig;
use crate::models::*;
use crate::utils::fs::files::scan_media_files;
use crate::utils::song::SongInformation;
use crate::DatabasePool;

#[instrument(
    skip(pool, song_absolute_path, song_file_size, parsing_config),
    ret(level = "trace"),
    err
)]
pub async fn process_path<P: AsRef<Path> + std::fmt::Debug>(
    pool: &DatabasePool,
    scan_started_at: time::OffsetDateTime,
    music_folder_id: Uuid,
    song_absolute_path: P,
    song_relative_path: &str,
    song_file_size: u64,
    parsing_config: &ParsingConfig,
) -> Result<bool> {
    let song_data = tokio::fs::read(&song_absolute_path).await?;
    let (song_file_hash, song_data) =
        tokio::task::spawn_blocking(move || (xxh3_64(&song_data), song_data)).await?;

    let song_file_hash = song_file_hash as i64;
    let song_file_size = song_file_size as i64;

    let song_id =
        if let Some((song_id_db, song_file_hash_db, song_file_size_db, song_relative_path_db)) =
            diesel::update(songs::table)
                .filter(songs::music_folder_id.eq(music_folder_id))
                .filter(songs::relative_path.eq(&song_relative_path).or(
                    songs::file_hash.eq(song_file_hash).and(songs::file_size.eq(song_file_size)),
                ))
                .set(songs::scanned_at.eq(time::OffsetDateTime::now_utc()))
                .returning((songs::id, songs::file_hash, songs::file_size, songs::relative_path))
                .get_result::<(Uuid, i64, i64, String)>(&mut pool.get().await?)
                .await
                .optional()?
        {
            // there is already an entry in the database with the same music folder and relative
            // path and it has the same size and hash with the file on local
            // disk, continue.
            if song_file_size_db == song_file_size && song_file_hash_db == song_file_hash {
                if song_relative_path != song_relative_path_db {
                    tracing::info!(new_path = ?song_relative_path, "duplicated song");
                    diesel::update(songs::table)
                        .filter(songs::id.eq(song_id_db))
                        .set(songs::relative_path.eq(&song_relative_path))
                        .execute(&mut pool.get().await?)
                        .await?;
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            Some(song_id_db)
        } else {
            None
        };

    let Ok(song_information) = SongInformation::read_from(
        &mut Cursor::new(&song_data),
        FileType::from_path(&song_absolute_path).expect("this should not happen"),
        parsing_config,
    ) else {
        return Ok(false);
    };

    let song_tag = &song_information.tag;

    let artist_ids = upsert_artists(pool, &song_tag.artists).await?;
    let album_id = upsert_album(pool, (&song_tag.album).into()).await?;

    let song_id = if let Some(song_id) = song_id {
        update_song(
            pool,
            song_id,
            song_information.to_update_information_db(album_id, song_file_hash, song_file_size),
        )
        .await?;
        song_id
    } else {
        insert_song(
            pool,
            song_information.to_full_information_db(
                album_id,
                song_file_hash,
                song_file_size,
                music_folder_id,
                &song_relative_path,
            ),
        )
        .await?
    };

    // if there are no album artists,
    // we assume that they are the same as artists.
    if !song_tag.album_artists.is_empty() {
        let album_artist_ids = upsert_artists(pool, &song_tag.album_artists).await?;
        upsert_song_album_artists(pool, &song_id, &album_artist_ids).await?;
    } else {
        upsert_song_album_artists(pool, &song_id, &artist_ids).await?;
    }
    // album artists for the same album
    // that are extracted from multiple songs
    // will be combined into a list.
    // for example:
    // song1 -> album -> album_artist1
    // song2 -> album -> album_artist2
    // album -> [album_artist1, album_artist2]
    diesel::delete(songs_album_artists::table)
        .filter(songs_album_artists::song_id.eq(song_id))
        .filter(songs_album_artists::upserted_at.lt(scan_started_at))
        .execute(&mut pool.get().await?)
        .await?;

    upsert_song_artists(pool, &song_id, &artist_ids).await?;
    diesel::delete(songs_artists::table)
        .filter(songs_artists::song_id.eq(song_id))
        .filter(songs_artists::upserted_at.lt(scan_started_at))
        .execute(&mut pool.get().await?)
        .await?;

    Ok(true)
}

#[instrument(skip(pool, music_folders, parsing_config, scan_config), ret, err)]
pub async fn scan_full(
    pool: &DatabasePool,
    scan_started_at: time::OffsetDateTime,
    music_folders: &[music_folders::MusicFolder],
    parsing_config: &ParsingConfig,
    scan_config: &ScanConfig,
) -> Result<ScanStatistic> {
    let span = tracing::Span::current();

    let mut scanned_song_count: usize = 0;
    let mut upserted_song_count: usize = 0;
    let mut scan_error_count: usize = 0;

    let mut scan_media_files_tasks = FuturesUnorderedBounded::new(scan_config.scan_media_task_size);
    let mut process_path_tasks = FuturesUnorderedBounded::new(scan_config.process_path_task_size);

    for music_folder in music_folders {
        let span = span.clone();
        let span_scan_one = span.clone();

        let music_folder_id = music_folder.id;
        let music_folder_path = std::path::PathBuf::from(&music_folder.path);

        while scan_media_files_tasks.len() == scan_media_files_tasks.capacity()
            && let Some(scan_media_files_task) = scan_media_files_tasks.next().await
        {
            scan_media_files_task?;
        }

        let (tx, rx) = crossfire::mpsc::bounded_tx_blocking_rx_future(scan_config.channel_size);
        scan_media_files_tasks.push(tokio::task::spawn_blocking(move || {
            let _enter = span.enter();
            scan_media_files(music_folder_path, tx)
        }));

        while let Ok((song_absolute_path, song_relative_path, song_file_size)) = rx.recv().await {
            let span = span_scan_one.clone();
            let pool = pool.clone();
            let parsing_config = parsing_config.clone();

            while process_path_tasks.len() >= process_path_tasks.capacity()
                && let Some(process_path_join_result) = process_path_tasks.next().await
            {
                if let Ok(process_path_result) = process_path_join_result
                    && let Ok(is_upserted) = process_path_result
                {
                    if is_upserted {
                        upserted_song_count += 1;
                    }
                } else {
                    scan_error_count += 1;
                }
            }

            scanned_song_count += 1;
            process_path_tasks.push(tokio::task::spawn(async move {
                let _enter = span.enter();
                process_path(
                    &pool,
                    scan_started_at,
                    music_folder_id,
                    &song_absolute_path,
                    &song_relative_path,
                    song_file_size,
                    &parsing_config,
                )
                .await
            }));
        }
    }

    scan_media_files_tasks.try_collect().await?;

    while let Some(process_path_join_result) = process_path_tasks.next().await {
        if let Ok(process_path_result) = process_path_join_result
            && let Ok(is_upserted) = process_path_result
        {
            if is_upserted {
                upserted_song_count += 1;
            }
        } else {
            scan_error_count += 1;
        }
    }

    let deleted_song_count = diesel::delete(songs::table)
        .filter(songs::scanned_at.lt(scan_started_at))
        .execute(&mut pool.get().await?)
        .await?;

    let albums_no_song = diesel::alias!(albums as albums_no_song);
    let deleted_album_count = diesel::delete(albums::table)
        .filter(
            albums::id.eq_any(
                albums_no_song
                    .filter(not(exists(
                        songs::table.filter(songs::album_id.eq(albums_no_song.field(albums::id))),
                    )))
                    .select(albums_no_song.field(albums::id)),
            ),
        )
        .execute(&mut pool.get().await?)
        .await?;

    let artists_no_song_no_album = diesel::alias!(artists as artists_no_song_no_album);
    let deleted_artist_count = diesel::delete(artists::table)
        .filter(
            artists::id.eq_any(
                artists_no_song_no_album
                    .filter(not(exists(
                        songs_album_artists::table.filter(
                            songs_album_artists::album_artist_id
                                .eq(artists_no_song_no_album.field(artists::id)),
                        ),
                    )))
                    .filter(not(exists(songs_artists::table.filter(
                        songs_artists::artist_id.eq(artists_no_song_no_album.field(artists::id)),
                    ))))
                    .select(artists_no_song_no_album.field(artists::id)),
            ),
        )
        .execute(&mut pool.get().await?)
        .await?;

    Ok(ScanStatistic {
        scanned_song_count,
        upserted_song_count,
        deleted_song_count,
        deleted_album_count,
        deleted_artist_count,
        scan_error_count,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fake::{Fake, Faker};
    use itertools::Itertools;

    use super::*;
    use crate::open_subsonic::scan::run_scan::{finish_scan, start_scan};
    use crate::utils::song::test::SongTag;
    use crate::utils::test::fs::SongFsInformation;
    use crate::utils::test::media::{
        assert_album_artist_names, assert_album_names, assert_albums_artists_info,
        assert_albums_info, assert_artists_info, assert_song_artist_names, assert_songs_info,
    };
    use crate::utils::test::setup::TestInfra;
    use crate::utils::test::{random, TemporaryFs};

    fn delete_and_update_songs<PM: AsRef<Path>>(
        temp_fs: &TemporaryFs,
        music_folder_path: PM,
        song_fs_infos: Vec<SongFsInformation>,
        n_delete: usize,
        n_update: usize,
    ) -> Vec<SongFsInformation> {
        let song_fs_infos = random::gen_bool_mask(song_fs_infos.len(), n_delete)
            .into_iter()
            .zip(song_fs_infos)
            .filter_map(|(d, s)| {
                if d {
                    std::fs::remove_file(s.absolute_path()).unwrap();
                    None
                } else {
                    Some(s)
                }
            })
            .collect_vec();

        let update_bool_mask = random::gen_bool_mask(song_fs_infos.len(), n_update);
        let new_song_fs_infos = temp_fs.create_media_files(
            &music_folder_path,
            update_bool_mask
                .iter()
                .copied()
                .zip(song_fs_infos.iter())
                .filter_map(|(u, s)| if u { Some(s.relative_path.clone()) } else { None })
                .collect(),
            fake::vec![SongTag; n_update],
        );

        [
            update_bool_mask
                .into_iter()
                .zip(song_fs_infos)
                .filter_map(|(u, s)| if u { None } else { Some(s) })
                .collect_vec(),
            new_song_fs_infos,
        ]
        .concat()
    }

    async fn wrap_scan_full(
        pool: &DatabasePool,
        music_folders: &[music_folders::MusicFolder],
        parsing_config: &ParsingConfig,
    ) -> ScanStatistic {
        let scan_started_at = start_scan(pool).await.unwrap();
        let scan_statistic =
            scan_full(pool, scan_started_at, music_folders, parsing_config, &ScanConfig::default())
                .await
                .unwrap();
        finish_scan(pool, scan_started_at, Ok(&scan_statistic)).await.unwrap();
        scan_statistic
    }

    #[tokio::test]
    async fn test_simple_scan() {
        let n_song = 50_usize;
        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[n_song], None).await;
        let ScanStatistic { upserted_song_count, deleted_song_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(test_infra.pool(), &song_fs_infos).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_update_same_path() {
        let n_song = 50_usize;
        let n_update_song = 20_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[n_song], None).await;
        let ScanStatistic { upserted_song_count, deleted_song_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        let song_fs_infos = delete_and_update_songs(
            &test_infra.fs,
            &test_infra.music_folders[0].path,
            song_fs_infos,
            0,
            n_update_song,
        );

        let ScanStatistic { upserted_song_count, deleted_song_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;

        assert_eq!(upserted_song_count, n_update_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(test_infra.pool(), &song_fs_infos).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_delete() {
        let n_song = 50_usize;
        let n_delete_song = 10_usize;
        let n_update_song = 20_usize;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[n_song], None).await;
        let ScanStatistic { upserted_song_count, deleted_song_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        let song_fs_infos = delete_and_update_songs(
            &test_infra.fs,
            &test_infra.music_folders[0].path,
            song_fs_infos,
            n_delete_song,
            n_update_song,
        );

        let ScanStatistic { upserted_song_count, deleted_song_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;

        assert_eq!(upserted_song_count, n_update_song);
        assert_eq!(deleted_song_count, n_delete_song);
        assert_songs_info(test_infra.pool(), &song_fs_infos).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_multiple_folders() {
        let n_song = 25_usize;

        let (test_infra, song_fs_infos) =
            TestInfra::setup_songs_no_scan(&[n_song, n_song], None).await;
        let ScanStatistic { upserted_song_count, deleted_song_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(upserted_song_count, n_song + n_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(test_infra.pool(), &song_fs_infos).await;
    }

    #[tokio::test]
    async fn test_scan_combine_album_artists() {
        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(
            &[2],
            vec![
                SongTag {
                    album: "album".to_owned(),
                    album_artists: vec!["artist1".to_owned(), "artist2".to_owned()],
                    ..Faker.fake()
                },
                SongTag {
                    album: "album".to_owned(),
                    album_artists: vec!["artist1".to_owned(), "artist3".to_owned()],
                    ..Faker.fake()
                },
            ],
        )
        .await;
        wrap_scan_full(test_infra.pool(), &test_infra.music_folders, &test_infra.fs.parsing_config)
            .await;

        assert_songs_info(test_infra.pool(), &song_fs_infos).await;
        assert_albums_artists_info(test_infra.pool(), &song_fs_infos).await;
    }

    #[tokio::test]
    async fn test_simple_scan_delete_old_albums() {
        let n_song = 10;
        let n_delete_song = 2;
        let n_update_song = 4;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[n_song], None).await;
        let ScanStatistic { deleted_album_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_album_count, 0);
        assert_albums_info(test_infra.pool(), &song_fs_infos).await;

        let song_fs_infos = delete_and_update_songs(
            &test_infra.fs,
            &test_infra.music_folders[0].path,
            song_fs_infos,
            n_delete_song,
            n_update_song,
        );

        let ScanStatistic { deleted_album_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;

        assert_eq!(deleted_album_count, n_delete_song + n_update_song);
        assert_albums_info(test_infra.pool(), &song_fs_infos).await;
    }

    #[tokio::test]
    async fn test_scan_delete_keep_album_with_songs() {
        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(
            &[2],
            vec![
                SongTag { album: "album".to_owned(), ..Faker.fake() },
                SongTag { album: "album".to_owned(), ..Faker.fake() },
            ],
        )
        .await;
        let ScanStatistic { deleted_album_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_album_count, 0);
        assert_album_names(test_infra.pool(), &["album"]).await;

        std::fs::remove_file(song_fs_infos[0].absolute_path()).unwrap();

        let ScanStatistic { deleted_album_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_album_count, 0);
        assert_album_names(test_infra.pool(), &["album"]).await;
    }

    #[tokio::test]
    async fn test_scan_all_artist() {
        let n_song = 10;
        let n_delete_song = 2;
        let n_update_song = 4;

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[n_song], None).await;
        wrap_scan_full(test_infra.pool(), &test_infra.music_folders, &test_infra.fs.parsing_config)
            .await;

        assert_artists_info(test_infra.pool(), &song_fs_infos).await;

        let song_fs_infos = delete_and_update_songs(
            &test_infra.fs,
            &test_infra.music_folders[0].path,
            song_fs_infos,
            n_delete_song,
            n_update_song,
        );

        wrap_scan_full(test_infra.pool(), &test_infra.music_folders, &test_infra.fs.parsing_config)
            .await;

        assert_artists_info(test_infra.pool(), &song_fs_infos).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_song_artists() {
        let song_tags = vec![
            // deleted
            SongTag {
                artists: vec!["artist1".to_owned()],
                album_artists: vec!["artist1".to_owned()],
                ..Faker.fake()
            },
            // not deleted but scanned (artist2)
            SongTag { artists: vec!["artist2".to_owned()], ..Faker.fake() },
            // not deleted nor scanned
            SongTag { artists: vec!["artist3".to_owned()], ..Faker.fake() },
        ];

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[3], song_tags).await;
        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        let music_folder_path = PathBuf::from(&test_infra.music_folders[0].path);
        assert_eq!(deleted_artist_count, 0);
        assert_song_artist_names(test_infra.pool(), &["artist1", "artist2", "artist3"]).await;

        test_infra.fs.create_media_file(
            &music_folder_path,
            &song_fs_infos[0].relative_path,
            SongTag { artists: vec!["artist2".to_owned()], ..Faker.fake() },
        );

        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_artist_count, 1);
        assert_song_artist_names(test_infra.pool(), &["artist2", "artist3"]).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_album_artists() {
        let song_tags = vec![
            SongTag {
                album: "album1".to_owned(),
                artists: vec!["artist2".to_owned()],
                album_artists: vec!["artist1".to_owned(), "artist2".to_owned()],
                ..Faker.fake()
            },
            SongTag {
                album: "album2".to_owned(),
                album_artists: vec!["artist2".to_owned(), "artist3".to_owned()],
                ..Faker.fake()
            },
        ];

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[2], song_tags).await;
        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(test_infra.pool(), &["artist1", "artist2", "artist3"]).await;

        std::fs::remove_file(song_fs_infos[0].absolute_path()).unwrap();

        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(test_infra.pool(), &["artist2", "artist3"]).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_combined_album_artists_with_delete() {
        let song_tags = vec![
            // deleted
            SongTag {
                album: "album".to_owned(),
                artists: vec!["artist1".to_owned(), "artist2".to_owned()],
                album_artists: vec!["artist1".to_owned()],
                ..Faker.fake()
            },
            // not deleted but scanned (artist2)
            SongTag {
                album: "album".to_owned(),
                album_artists: vec!["artist2".to_owned()],
                ..Faker.fake()
            },
            // not deleted nor scanned
            SongTag {
                album: "album".to_owned(),
                album_artists: vec!["artist3".to_owned()],
                ..Faker.fake()
            },
        ];

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[3], song_tags).await;
        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(test_infra.pool(), &["artist1", "artist2", "artist3"]).await;

        std::fs::remove_file(song_fs_infos[0].absolute_path()).unwrap();

        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(test_infra.pool(), &["artist2", "artist3"]).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_combined_album_artists_with_update() {
        let song_tags = vec![
            // deleted
            SongTag {
                album: "album".to_owned(),
                artists: vec!["artist1".to_owned(), "artist2".to_owned()],
                album_artists: vec!["artist1".to_owned()],
                ..Faker.fake()
            },
            // not deleted but scanned (artist2)
            SongTag {
                album: "album".to_owned(),
                album_artists: vec!["artist2".to_owned()],
                ..Faker.fake()
            },
            // not deleted nor scanned
            SongTag {
                album: "album".to_owned(),
                album_artists: vec!["artist3".to_owned()],
                ..Faker.fake()
            },
        ];
        let first_song_tag = song_tags[0].clone();

        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[3], song_tags).await;
        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        let music_folder_path = PathBuf::from(&test_infra.music_folders[0].path);
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(test_infra.pool(), &["artist1", "artist2", "artist3"]).await;

        test_infra.fs.create_media_file(
            &music_folder_path,
            &song_fs_infos[0].relative_path,
            SongTag {
                artists: vec!["artist2".to_owned()],
                album_artists: vec!["artist2".to_owned()],
                ..first_song_tag
            },
        );

        let ScanStatistic { deleted_artist_count, .. } = wrap_scan_full(
            test_infra.pool(),
            &test_infra.music_folders,
            &test_infra.fs.parsing_config,
        )
        .await;
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(test_infra.pool(), &["artist2", "artist3"]).await;
    }

    #[tokio::test]
    async fn test_duplicate_song() {
        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[1], None).await;
        wrap_scan_full(test_infra.pool(), &test_infra.music_folders, &test_infra.fs.parsing_config)
            .await;

        let music_folder_path = PathBuf::from(&test_infra.music_folders[0].path);
        let old_song_path = PathBuf::from(&song_fs_infos[0].relative_path);
        let new_song_path = old_song_path
            .with_file_name(Faker.fake::<String>())
            .with_extension(old_song_path.extension().unwrap());
        std::fs::copy(
            music_folder_path.join(&old_song_path),
            music_folder_path.join(&new_song_path),
        )
        .unwrap();

        let ScanStatistic { scanned_song_count, upserted_song_count, deleted_song_count, .. } =
            wrap_scan_full(
                test_infra.pool(),
                &test_infra.music_folders,
                &test_infra.fs.parsing_config,
            )
            .await;
        assert_eq!(scanned_song_count, 2);
        assert_eq!(deleted_song_count, 0);

        if upserted_song_count == 2 {
            // The new song is scanned before the old song.
            // The path is set to the new song and then back to the old song.
            assert_songs_info(test_infra.pool(), &song_fs_infos).await;
        } else if upserted_song_count == 1 {
            // The old song is scanned after the old song.
            // The path is set to the new song.
            assert_songs_info(
                test_infra.pool(),
                &[SongFsInformation {
                    relative_path: new_song_path.to_str().unwrap().to_owned(),
                    ..song_fs_infos[0].clone()
                }],
            )
            .await;
        } else {
            panic!("upserted song count value is invalid")
        }
    }

    #[tokio::test]
    async fn test_move_song() {
        let (test_infra, song_fs_infos) = TestInfra::setup_songs_no_scan(&[1], None).await;
        wrap_scan_full(test_infra.pool(), &test_infra.music_folders, &test_infra.fs.parsing_config)
            .await;

        let music_folder_path = PathBuf::from(&test_infra.music_folders[0].path);
        let old_song_path = PathBuf::from(&song_fs_infos[0].relative_path);
        let new_song_path = old_song_path
            .with_file_name(Faker.fake::<String>())
            .with_extension(old_song_path.extension().unwrap());
        std::fs::rename(
            music_folder_path.join(&old_song_path),
            music_folder_path.join(&new_song_path),
        )
        .unwrap();

        let ScanStatistic { scanned_song_count, upserted_song_count, deleted_song_count, .. } =
            wrap_scan_full(
                test_infra.pool(),
                &test_infra.music_folders,
                &test_infra.fs.parsing_config,
            )
            .await;
        assert_eq!(scanned_song_count, 1);
        assert_eq!(deleted_song_count, 0);
        assert_eq!(upserted_song_count, 1);
        assert_songs_info(
            test_infra.pool(),
            &[SongFsInformation {
                relative_path: new_song_path.to_str().unwrap().to_owned(),
                ..song_fs_infos[0].clone()
            }],
        )
        .await;
    }
}
