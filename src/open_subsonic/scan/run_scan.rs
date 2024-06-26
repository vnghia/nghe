use std::io::Cursor;

use anyhow::Result;
use diesel::dsl::{exists, not};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::StreamExt;
use futures_buffered::FuturesUnorderedBounded;
use nghe_types::scan::start_scan::ScanMode;
use tracing::{instrument, Instrument};
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use super::album::{upsert_album, upsert_song_album_artists};
use super::artist::upsert_artists;
use super::genre::{upsert_genres, upsert_song_genres};
use super::song::{insert_song, update_song, upsert_song_artists, upsert_song_cover_art};
use super::ScanStat;
use crate::config::{ArtConfig, ParsingConfig, ScanConfig};
use crate::models::*;
use crate::utils::fs::FsTrait;
use crate::utils::path::PathInfo;
use crate::utils::song::file_type::SUPPORTED_EXTENSIONS;
use crate::utils::song::{SongInformation, SongLyric};
use crate::{DatabasePool, OSError};

#[instrument(
    skip(
        pool,
        fs,
        scan_mode,
        music_folder_id,
        music_folder_path,
        ignored_prefixes,
        parsing_config,
        art_config
    ),
    ret(level = "trace"),
    err
)]
pub async fn process_path<Fs: FsTrait>(
    pool: &DatabasePool,
    fs: &Fs,
    scan_started_at: time::OffsetDateTime,
    scan_mode: ScanMode,
    music_folder_id: Uuid,
    music_folder_path: &str,
    song_path_info: PathInfo<Fs>,
    ignored_prefixes: &[String],
    parsing_config: &ParsingConfig,
    art_config: &ArtConfig,
) -> Result<bool> {
    let song_path = &song_path_info.path;
    let song_relative_path = song_path.strip_prefix(music_folder_path)?;

    if scan_mode == ScanMode::Quick
        && let Some(last_modified) = song_path_info.metadata.last_modified
        && diesel::update(songs::table)
            .filter(songs::music_folder_id.eq(music_folder_id))
            .filter(songs::relative_path.eq(song_relative_path.as_str()))
            .filter(songs::updated_at.ge(last_modified))
            .set(songs::scanned_at.eq(time::OffsetDateTime::now_utc()))
            .execute(&mut pool.get().await?)
            .await?
            > 0
    {
        // There is one song at that path and updated at after the latest modification of this song,
        // return scanned but not upserted.
        // In case of duplication, the song that is not in the database before (because of hash size
        // constrait) will be scanned and will overwrite the song that is already inside the
        // database. This will always happen for each scan as long as there are duplications.
        // In case of moving, the song at the destination will be scanned and the path will be
        // updated.
        return Ok(false);
    }

    let song_data = fs.read(song_path).await?;
    let song_file_hash = xxh3_64(&song_data);

    let song_file_hash = song_file_hash as _;
    let song_file_size = song_path_info.metadata.size as _;

    let song_id =
        if let Some((song_id_db, song_file_hash_db, song_file_size_db, song_relative_path_db)) =
            diesel::update(songs::table)
                .filter(songs::music_folder_id.eq(music_folder_id))
                .filter(songs::relative_path.eq(song_relative_path.as_str()).or(
                    songs::file_hash.eq(song_file_hash).and(songs::file_size.eq(song_file_size)),
                ))
                .set(songs::scanned_at.eq(time::OffsetDateTime::now_utc()))
                .returning((songs::id, songs::file_hash, songs::file_size, songs::relative_path))
                .get_result::<(Uuid, i64, i32, String)>(&mut pool.get().await?)
                .await
                .optional()?
        {
            if song_file_size_db == song_file_size && song_file_hash_db == song_file_hash {
                if song_relative_path != song_relative_path_db {
                    // There is already an entry in the database with the same music folder, size
                    // and hash, but different relative path. Update its path to
                    // the newer one and continue if scan mode is not force
                    // and return the song id for further processing otherwise.
                    tracing::info!(old_path = ?song_relative_path_db, "duplicated song");
                    diesel::update(songs::table)
                        .filter(songs::id.eq(song_id_db))
                        .set(songs::relative_path.eq(song_relative_path.as_str()))
                        .execute(&mut pool.get().await?)
                        .await?;
                    if scan_mode > ScanMode::Full {
                        Some(song_id_db)
                    } else {
                        if let Ok(lrc_content) =
                            fs.read_to_string(song_path.with_extension("lrc")).await
                        {
                            SongLyric::from_str(&lrc_content, true)?
                                .upsert_lyric(pool, song_id_db)
                                .await?;
                        }
                        return Ok(true);
                    }
                } else if scan_mode > ScanMode::Full {
                    // There is an entry in the database with same music folder, path, size and
                    // hash. Continue if scan mode is not force and return the song id
                    // for further processing otherwise.
                    Some(song_id_db)
                } else {
                    return Ok(false);
                }
            } else {
                Some(song_id_db)
            }
        } else {
            None
        };

    let Ok(song_information) = SongInformation::read_from(
        &mut Cursor::new(&song_data),
        *SUPPORTED_EXTENSIONS
            .get(song_path.extension().ok_or_else(|| {
                OSError::InvalidParameter("song path does not have an extension".into())
            })?)
            .ok_or_else(|| {
                OSError::InvalidParameter("can not determine file type from extension".into())
            })?,
        fs.read_to_string(song_path.with_extension("lrc")).await.ok().as_deref(),
        parsing_config,
    ) else {
        return Ok(false);
    };

    let song_tag = &song_information.tag;

    let artist_ids = upsert_artists(pool, ignored_prefixes, &song_tag.artists).await?;
    let album_id = upsert_album(pool, (&song_tag.album).into()).await?;

    let cover_art_id = if let Some(ref picture) = song_tag.picture
        && let Some(ref song_art_dir) = art_config.song_dir
    {
        Some(upsert_song_cover_art(pool, picture, song_art_dir).await?)
    } else {
        None
    };

    let song_id = if let Some(song_id) = song_id {
        update_song(
            pool,
            song_id,
            song_information.to_update_information_db(
                album_id,
                song_file_hash,
                song_file_size,
                cover_art_id,
            ),
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
                cover_art_id,
                music_folder_id,
                &song_relative_path,
            ),
        )
        .await?
    };

    // if there are no album artists,
    // we assume that they are the same as artists.
    if !song_tag.album_artists.is_empty() {
        // if the song has compilation, all artists in the song artists will be added to song album
        // artists with compilation set to true. If it also contains any album artists, the
        // compilation field will be overwritten to false later. If the album artists field
        // is empty, the album artists will be the same with song artists which in turn set
        // any compilation field to false, so don't set to true in the first place.
        if song_tag.compilation {
            upsert_song_album_artists(pool, song_id, &artist_ids, true).await?;
        }
        let album_artist_ids =
            upsert_artists(pool, ignored_prefixes, &song_tag.album_artists).await?;
        upsert_song_album_artists(pool, song_id, &album_artist_ids, false).await?;
    } else {
        upsert_song_album_artists(pool, song_id, &artist_ids, false).await?;
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

    upsert_song_artists(pool, song_id, &artist_ids).await?;
    diesel::delete(songs_artists::table)
        .filter(songs_artists::song_id.eq(song_id))
        .filter(songs_artists::upserted_at.lt(scan_started_at))
        .execute(&mut pool.get().await?)
        .await?;

    upsert_song_genres(pool, song_id, &upsert_genres(pool, &song_tag.genres).await?).await?;
    diesel::delete(songs_genres::table)
        .filter(songs_genres::song_id.eq(song_id))
        .filter(songs_genres::upserted_at.lt(scan_started_at))
        .execute(&mut pool.get().await?)
        .await?;

    if let Some(ref lrc) = song_information.lrc {
        lrc.upsert_lyric(pool, song_id).await?;
    }

    diesel::delete(lyrics::table)
        .filter(lyrics::song_id.eq(song_id))
        .filter(lyrics::scanned_at.lt(scan_started_at))
        .execute(&mut pool.get().await?)
        .await?;

    Ok(true)
}

#[instrument(skip(pool, fs, ignored_prefixes, parsing_config, scan_config, art_config), ret, err)]
pub async fn run_scan(
    pool: &DatabasePool,
    fs: &(impl FsTrait + 'static),
    scan_started_at: time::OffsetDateTime,
    scan_mode: ScanMode,
    music_folder: music_folders::MusicFolder,
    ignored_prefixes: &[String],
    parsing_config: &ParsingConfig,
    scan_config: &ScanConfig,
    art_config: &ArtConfig,
) -> Result<ScanStat> {
    tracing::info!("start scanning and parsing");

    let music_folder_id = music_folder.id;

    let mut scanned_song_count: usize = 0;
    let mut upserted_song_count: usize = 0;
    let mut scan_error_count: usize = 0;

    let (tx, rx) = flume::bounded(scan_config.channel_size);
    let scan_songs_task = fs.scan_songs(music_folder.path.clone(), tx);

    let mut process_path_tasks = FuturesUnorderedBounded::new(scan_config.pool_size);
    while let Ok(song_path) = rx.recv_async().await {
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

        let pool = pool.clone();
        let fs = fs.clone();
        let music_folder_path = music_folder.path.clone();
        let ignored_prefixes = ignored_prefixes.to_vec();
        let parsing_config = parsing_config.clone();
        let art_config = art_config.clone();

        let span = tracing::Span::current();
        process_path_tasks.push(tokio::task::spawn(
            async move {
                process_path(
                    &pool,
                    &fs,
                    scan_started_at,
                    scan_mode,
                    music_folder_id,
                    &music_folder_path,
                    song_path,
                    &ignored_prefixes,
                    &parsing_config,
                    &art_config,
                )
                .await
            }
            .instrument(span),
        ));
    }

    scan_songs_task.await?;

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
        .filter(songs::music_folder_id.eq(music_folder_id))
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

    let genres_no_song = diesel::alias!(genres as genres_no_song);
    let deleted_genre_count = diesel::delete(genres::table)
        .filter(
            genres::id.eq_any(
                genres_no_song
                    .filter(not(exists(
                        songs_genres::table
                            .filter(songs_genres::genre_id.eq(genres_no_song.field(genres::id))),
                    )))
                    .select(genres_no_song.field(genres::id)),
            ),
        )
        .execute(&mut pool.get().await?)
        .await?;

    tracing::info!("finish scanning and parsing");
    Ok(ScanStat {
        scanned_song_count,
        upserted_song_count,
        deleted_song_count,
        deleted_album_count,
        deleted_artist_count,
        deleted_genre_count,
        scan_error_count,
    })
}

#[cfg(test)]
mod tests {

    use fake::{Fake, Faker};

    use super::*;
    use crate::utils::song::test::SongTag;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_simple_scan() {
        let n_song = 50;
        let mut infra = Infra::new().await.n_folder(1).await;
        infra.add_n_song(0, n_song).await;
        let ScanStat { upserted_song_count, deleted_song_count, .. } = infra.scan(.., None).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_update_same_path() {
        let n_song = 50_usize;
        let n_update_song = 20_usize;
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.add_n_song(0, n_song).await.scan(.., None).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.update_n_song(0, n_update_song).await.scan(.., None).await;
        assert_eq!(upserted_song_count, n_update_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_delete() {
        let n_song = 50_usize;
        let n_delete_song = 10_usize;
        let n_update_song = 20_usize;
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.add_n_song(0, n_song).await.scan(.., None).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        let ScanStat { upserted_song_count, deleted_song_count, .. } = infra
            .delete_n_song(0, n_delete_song)
            .await
            .update_n_song(0, n_update_song)
            .await
            .scan(.., None)
            .await;
        assert_eq!(upserted_song_count, n_update_song);
        assert_eq!(deleted_song_count, n_delete_song);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_multiple_folders() {
        let n_song = 25_usize;
        let mut infra = Infra::new().await.n_folder(2).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.add_n_song(0, n_song).await.add_n_song(1, n_song).await.scan(.., None).await;
        assert_eq!(upserted_song_count, n_song + n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_scan_combine_album_artists() {
        let mut infra = Infra::new().await.n_folder(1).await;
        infra
            .add_songs(
                0,
                vec![
                    SongTag {
                        album: "album".into(),
                        album_artists: vec!["artist1".into(), "artist2".into()],
                        ..Faker.fake()
                    },
                    SongTag {
                        album: "album".into(),
                        album_artists: vec!["artist1".into(), "artist3".into()],
                        ..Faker.fake()
                    },
                ],
            )
            .await
            .scan(.., None)
            .await;

        infra.assert_song_infos().await;
        infra.assert_no_compilation_album_artist_infos(..).await;
    }

    #[tokio::test]
    async fn test_simple_scan_delete_old_albums() {
        let n_song = 10;
        let n_delete_song = 2;
        let n_update_song = 4;

        let mut infra = Infra::new().await.n_folder(1).await;
        let ScanStat { deleted_album_count, .. } =
            infra.add_n_song(0, n_song).await.scan(.., None).await;
        assert_eq!(deleted_album_count, 0);
        infra.assert_album_infos(&infra.album_no_ids(..)).await;

        let ScanStat { deleted_album_count, .. } = infra
            .delete_n_song(0, n_delete_song)
            .await
            .update_n_song(0, n_update_song)
            .await
            .scan(.., None)
            .await;
        assert_eq!(deleted_album_count, n_delete_song + n_update_song);
        infra.assert_album_infos(&infra.album_no_ids(..)).await;
    }

    #[tokio::test]
    async fn test_scan_delete_keep_album_with_songs() {
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { deleted_album_count, .. } = infra
            .add_songs(
                0,
                vec![
                    SongTag { album: "album".into(), ..Faker.fake() },
                    SongTag { album: "album".into(), ..Faker.fake() },
                ],
            )
            .await
            .scan(.., None)
            .await;
        assert_eq!(deleted_album_count, 0);
        infra.assert_album_infos(&["album".into()]).await;

        let ScanStat { deleted_album_count, .. } =
            infra.delete_song(0, 0).await.scan(.., None).await;
        assert_eq!(deleted_album_count, 0);
        infra.assert_album_infos(&["album".into()]).await;
    }

    #[tokio::test]
    async fn test_scan_all_artist() {
        let n_song = 10;
        let n_delete_song = 2;
        let n_update_song = 4;
        let mut infra = Infra::new().await.n_folder(1).await;

        infra.add_n_song(0, n_song).await.scan(.., None).await;
        infra.assert_artist_infos(..).await;

        infra
            .delete_n_song(0, n_delete_song)
            .await
            .update_n_song(0, n_update_song)
            .await
            .scan(.., None)
            .await;
        infra.assert_artist_infos(..).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_song_artists() {
        let mut infra = Infra::new().await.n_folder(1).await;
        let ScanStat { deleted_artist_count, .. } = infra
            .add_songs(
                0,
                vec![
                    // deleted
                    SongTag {
                        artists: vec!["artist1".into()],
                        album_artists: vec!["artist1".into()],
                        ..Faker.fake()
                    },
                    // not deleted but scanned (artist2)
                    SongTag { artists: vec!["artist2".into()], ..Faker.fake() },
                    // not deleted nor scanned
                    SongTag { artists: vec!["artist3".into()], ..Faker.fake() },
                ],
            )
            .await
            .scan(.., None)
            .await;
        assert_eq!(deleted_artist_count, 0);
        infra
            .assert_song_artist_no_ids(&["artist1".into(), "artist2".into(), "artist3".into()])
            .await;

        let ScanStat { deleted_artist_count, .. } = infra
            .update_song(0, 0, SongTag { artists: vec!["artist2".into()], ..Faker.fake() })
            .await
            .scan(.., None)
            .await;
        assert_eq!(deleted_artist_count, 1);
        infra.assert_song_artist_no_ids(&["artist2".into(), "artist3".into()]).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_album_artists() {
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { deleted_artist_count, .. } = infra
            .add_songs(
                0,
                vec![
                    SongTag {
                        album: "album1".into(),
                        artists: vec!["artist2".into()],
                        album_artists: vec!["artist1".into(), "artist2".into()],
                        ..Faker.fake()
                    },
                    SongTag {
                        album: "album2".into(),
                        album_artists: vec!["artist2".into(), "artist3".into()],
                        ..Faker.fake()
                    },
                ],
            )
            .await
            .scan(.., None)
            .await;
        assert_eq!(deleted_artist_count, 0);
        infra
            .assert_no_compilation_album_artist_no_ids(&[
                "artist1".into(),
                "artist2".into(),
                "artist3".into(),
            ])
            .await;

        let ScanStat { deleted_artist_count, .. } =
            infra.delete_song(0, 0).await.scan(.., None).await;
        assert_eq!(deleted_artist_count, 1);
        infra
            .assert_no_compilation_album_artist_no_ids(&["artist2".into(), "artist3".into()])
            .await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_combined_album_artists_with_delete() {
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { deleted_artist_count, .. } = infra
            .add_songs(
                0,
                vec![
                    // deleted
                    SongTag {
                        album: "album".into(),
                        artists: vec!["artist1".into(), "artist2".into()],
                        album_artists: vec!["artist1".into()],
                        ..Faker.fake()
                    },
                    // not deleted but scanned (artist2)
                    SongTag {
                        album: "album".into(),
                        album_artists: vec!["artist2".into()],
                        ..Faker.fake()
                    },
                    // not deleted nor scanned
                    SongTag {
                        album: "album".into(),
                        album_artists: vec!["artist3".into()],
                        ..Faker.fake()
                    },
                ],
            )
            .await
            .scan(.., None)
            .await;
        assert_eq!(deleted_artist_count, 0);
        infra
            .assert_no_compilation_album_artist_no_ids(&[
                "artist1".into(),
                "artist2".into(),
                "artist3".into(),
            ])
            .await;

        let ScanStat { deleted_artist_count, .. } =
            infra.delete_song(0, 0).await.scan(.., None).await;
        assert_eq!(deleted_artist_count, 1);
        infra
            .assert_no_compilation_album_artist_no_ids(&["artist2".into(), "artist3".into()])
            .await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_combined_album_artists_with_update() {
        let mut infra = Infra::new().await.n_folder(1).await;
        let song_tags = vec![
            // deleted
            SongTag {
                album: "album".into(),
                artists: vec!["artist1".into(), "artist2".into()],
                album_artists: vec!["artist1".into()],
                ..Faker.fake()
            },
            // not deleted but scanned (artist2)
            SongTag {
                album: "album".into(),
                album_artists: vec!["artist2".into()],
                ..Faker.fake()
            },
            // not deleted nor scanned
            SongTag {
                album: "album".into(),
                album_artists: vec!["artist3".into()],
                ..Faker.fake()
            },
        ];
        let first_song_tag = song_tags[0].clone();

        let ScanStat { deleted_artist_count, .. } =
            infra.add_songs(0, song_tags).await.scan(.., None).await;
        assert_eq!(deleted_artist_count, 0);
        infra
            .assert_no_compilation_album_artist_no_ids(&[
                "artist1".into(),
                "artist2".into(),
                "artist3".into(),
            ])
            .await;

        let ScanStat { deleted_artist_count, .. } = infra
            .update_song(
                0,
                0,
                SongTag {
                    artists: vec!["artist2".into()],
                    album_artists: vec!["artist2".into()],
                    ..first_song_tag
                },
            )
            .await
            .scan(.., None)
            .await;
        assert_eq!(deleted_artist_count, 1);
        infra
            .assert_no_compilation_album_artist_no_ids(&["artist2".into(), "artist3".into()])
            .await;
    }

    #[tokio::test]
    async fn test_duplicate_song() {
        let mut infra = Infra::new().await.n_folder(1).await;
        infra.add_n_song(0, 1).await.scan(.., None).await;

        let ScanStat { scanned_song_count, deleted_song_count, .. } =
            infra.copy_song(0, 0, &Infra::fake_fs_name()).await.scan(.., None).await;
        assert_eq!(scanned_song_count, 2);
        assert_eq!(deleted_song_count, 0);

        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_move_song() {
        let mut infra = Infra::new().await.n_folder(1).await;
        infra.add_n_song(0, 1).await.scan(.., None).await;

        let ScanStat { scanned_song_count, upserted_song_count, deleted_song_count, .. } = infra
            .copy_song(0, 0, &Infra::fake_fs_name())
            .await
            .delete_song(0, 0)
            .await
            .scan(.., None)
            .await;
        assert_eq!(scanned_song_count, 1);
        assert_eq!(upserted_song_count, 1);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_force_scan() {
        let n_song = 50_usize;
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.add_n_song(0, n_song).await.scan(.., Some(ScanMode::Force)).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_full_and_force_scan() {
        let n_song = 50_usize;
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.add_n_song(0, n_song).await.scan(.., Some(ScanMode::Full)).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.scan(.., Some(ScanMode::Force)).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_force_scan_move_song() {
        let n_song = 50_usize;
        let mut infra = Infra::new().await.n_folder(1).await;
        infra.add_n_song(0, n_song).await.scan(.., Some(ScanMode::Full)).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } = infra
            .copy_song(0, 0, &Infra::fake_fs_name())
            .await
            .delete_song(0, 0)
            .await
            .scan(.., Some(ScanMode::Force))
            .await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_keep_genre() {
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag { genres: vec!["genre".into()], ..Faker.fake() })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let ScanStat { deleted_genre_count, .. } =
            infra.delete_n_song(0, 5).await.scan(.., None).await;
        assert_eq!(deleted_genre_count, 0);
    }

    #[tokio::test]
    async fn test_delete_genre() {
        let n_song = 10_usize;
        let mut infra = Infra::new().await.n_folder(1).await;
        infra
            .add_songs(
                0,
                (0..n_song)
                    .map(|_| SongTag { genres: vec!["genre".into()], ..Faker.fake() })
                    .collect(),
            )
            .await
            .scan(.., None)
            .await;

        let ScanStat { deleted_genre_count, .. } =
            infra.delete_n_song(0, n_song).await.scan(.., None).await;
        assert_eq!(deleted_genre_count, 1);
    }

    #[tokio::test]
    async fn test_quick_scan() {
        let n_song = 50_usize;
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.add_n_song(0, n_song).await.scan(.., Some(ScanMode::Quick)).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_quick_and_quick_scan() {
        let n_song = 50_usize;
        let mut infra = Infra::new().await.n_folder(1).await;

        let ScanStat { upserted_song_count, deleted_song_count, .. } =
            infra.add_n_song(0, n_song).await.scan(.., Some(ScanMode::Quick)).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;

        let ScanStat { scanned_song_count, upserted_song_count, deleted_song_count, .. } =
            infra.scan(.., Some(ScanMode::Quick)).await;
        assert_eq!(scanned_song_count, n_song);
        assert_eq!(upserted_song_count, 0);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_quick_scan_duplicate_song() {
        let mut infra = Infra::new().await.n_folder(1).await;
        infra.add_n_song(0, 1).await.scan(.., None).await;

        let ScanStat { scanned_song_count, upserted_song_count, deleted_song_count, .. } = infra
            .copy_song(0, 0, &Infra::fake_fs_name())
            .await
            .scan(.., Some(ScanMode::Quick))
            .await;
        assert_eq!(scanned_song_count, 2);
        assert_eq!(upserted_song_count, 1);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;

        let ScanStat { scanned_song_count, upserted_song_count, deleted_song_count, .. } =
            infra.scan(.., Some(ScanMode::Quick)).await;
        assert_eq!(scanned_song_count, 2);
        assert_eq!(upserted_song_count, 1);
        assert_eq!(deleted_song_count, 0);
    }

    #[tokio::test]
    async fn test_quick_scan_move_song() {
        let n_song = 50_usize;
        let mut infra = Infra::new().await.n_folder(1).await;
        infra.add_n_song(0, n_song).await.scan(.., Some(ScanMode::Full)).await;

        let ScanStat { scanned_song_count, upserted_song_count, deleted_song_count, .. } = infra
            .copy_song(0, 0, &Infra::fake_fs_name())
            .await
            .delete_song(0, 0)
            .await
            .scan(.., Some(ScanMode::Quick))
            .await;
        assert_eq!(scanned_song_count, n_song);
        assert_eq!(upserted_song_count, 1);
        assert_eq!(deleted_song_count, 0);
        infra.assert_song_infos().await;
    }

    #[tokio::test]
    async fn test_upsert_both_album_artist_and_artist() {
        let mut infra = Infra::new().await.n_folder(1).await;
        infra
            .add_songs(
                0,
                vec![SongTag {
                    album: "album".into(),
                    artists: vec!["artist1".into()],
                    album_artists: vec!["artist1".into()],
                    ..Faker.fake()
                }],
            )
            .await
            .scan(.., None)
            .await;
        infra.assert_no_compilation_album_artist_no_ids(&["artist1".into()]).await;
    }
}
