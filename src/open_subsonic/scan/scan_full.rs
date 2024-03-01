use super::{
    album::upsert_album, album::upsert_song_album_artists, artist::upsert_artists,
    song::upsert_song, song::upsert_song_artists,
};
use crate::{
    models::*,
    utils::{fs::files::scan_media_files, song::tag::SongTag},
    DatabasePool, OSResult,
};

use diesel::{
    dsl::{exists, not},
    ExpressionMethods, OptionalExtension, QueryDsl,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

pub async fn scan_full(
    pool: &DatabasePool,
    scan_started_at: &time::OffsetDateTime,
    music_folders: &[music_folders::MusicFolder],
) -> OSResult<(usize, usize, usize, usize, usize)> {
    let mut scanned_song_count: usize = 0;
    let mut upserted_song_count: usize = 0;

    for music_folder in music_folders {
        let music_folder_path = std::path::PathBuf::from(&music_folder.path);
        for (song_absolute_path, song_relative_path, song_file_size) in
            tokio::task::spawn_blocking(move || scan_media_files(music_folder_path)).await??
        {
            let song_file_metadata_db = diesel::update(songs::table)
                .filter(songs::music_folder_id.eq(music_folder.id))
                .filter(songs::path.eq(&song_relative_path))
                .set(songs::scanned_at.eq(time::OffsetDateTime::now_utc()))
                .returning((songs::id, songs::file_hash, songs::file_size))
                .get_result::<(Uuid, i64, i64)>(&mut pool.get().await?)
                .await
                .optional()?;

            let song_data = tokio::fs::read(&song_absolute_path).await?;
            let (song_file_hash, song_data) =
                tokio::task::spawn_blocking(move || (xxh3_64(&song_data), song_data)).await?;

            scanned_song_count += 1;

            let song_id = if let Some((song_id_db, song_file_hash_db, song_file_size_db)) =
                song_file_metadata_db
            {
                // there is already an entry in the database with the same music folder and relative path
                // and it has the same size and hash with the file on local disk, continue.
                if song_file_size_db as u64 == song_file_size
                    && song_file_hash_db as u64 == song_file_hash
                {
                    continue;
                }
                Some(song_id_db)
            } else {
                None
            };

            let song_tag = SongTag::parse(&song_data, &song_absolute_path)?;

            let artist_ids = upsert_artists(pool, &song_tag.artists).await?;
            let album_id = upsert_album(pool, (&song_tag.album).into()).await?;

            let song_id = upsert_song(
                pool,
                song_id,
                song_tag.to_new_or_update_song(
                    music_folder.id,
                    album_id,
                    song_file_hash,
                    song_file_size,
                    // only supply path if song id is none
                    // i.e: we are inserting a new song.
                    if song_id.is_none() {
                        Some(&song_relative_path)
                    } else {
                        None
                    },
                ),
            )
            .await?;

            upsert_song_album_artists(
                pool,
                &song_id,
                &upsert_artists(pool, &song_tag.album_artists).await?,
            )
            .await?;
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

            upserted_song_count += 1;
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

    tracing::info!("done scanning songs");
    Ok((
        scanned_song_count,
        upserted_song_count,
        deleted_song_count,
        deleted_album_count,
        deleted_artist_count,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::scan::run_scan::{finish_scan, start_scan},
        utils::test::{
            media::{
                assert_album_artist_names, assert_album_names, assert_albums_artists_info,
                assert_albums_info, assert_artists_info, assert_song_artist_names,
                assert_songs_info,
            },
            setup::setup_songs_no_scan,
        },
    };

    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rand::seq::IteratorRandom;
    use std::path::PathBuf;

    async fn wrap_scan_full(
        pool: &DatabasePool,
        music_folders: &[music_folders::MusicFolder],
    ) -> (usize, usize, usize, usize) {
        let scan_started_at = start_scan(pool).await.unwrap();
        let (
            scanned_song_count,
            upserted_song_count,
            deleted_song_count,
            deleted_album_count,
            deleted_artist_count,
        ) = scan_full(pool, &scan_started_at, music_folders)
            .await
            .unwrap();
        finish_scan(pool, &scan_started_at, Ok(scanned_song_count))
            .await
            .unwrap();
        (
            upserted_song_count,
            deleted_song_count,
            deleted_album_count,
            deleted_artist_count,
        )
    }

    #[tokio::test]
    async fn test_simple_scan() {
        let n_song = 50_usize;
        let (temp_db, _temp_fs, music_folders, song_fs_info) =
            setup_songs_no_scan(1, &[n_song], None).await;
        let (upserted_song_count, deleted_song_count, _, _) =
            wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(temp_db.pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_update_same_path() {
        let n_song = 50_usize;
        let n_new_song = 20_usize;

        let (temp_db, temp_fs, music_folders, mut song_fs_info) =
            setup_songs_no_scan(1, &[n_song], None).await;
        let (upserted_song_count, deleted_song_count, _, _) =
            wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        song_fs_info.extend(
            temp_fs.create_nested_media_files(
                music_folders[0].id,
                &PathBuf::from(&music_folders[0].path),
                &song_fs_info
                    .keys()
                    .choose_multiple(&mut rand::thread_rng(), n_new_song)
                    .into_iter()
                    .map(|(_, path)| path)
                    .collect_vec(),
                fake::vec![SongTag; n_new_song],
            ),
        );

        let (upserted_song_count, deleted_song_count, _, _) =
            wrap_scan_full(temp_db.pool(), &music_folders).await;

        assert_eq!(upserted_song_count, n_new_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(temp_db.pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_delete() {
        let n_song = 50_usize;
        let n_delete_song = 10_usize;
        let n_new_song = 20_usize;

        let (temp_db, temp_fs, music_folders, mut song_fs_info) =
            setup_songs_no_scan(1, &[n_song], None).await;
        let (upserted_song_count, deleted_song_count, _, _) =
            wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        song_fs_info
            .keys()
            .cloned()
            .choose_multiple(&mut rand::thread_rng(), n_delete_song)
            .into_iter()
            .for_each(|key| {
                std::fs::remove_file(music_folder_path.join(&key.1)).unwrap();
                song_fs_info.remove(&key).unwrap();
            });

        song_fs_info.extend(
            temp_fs.create_nested_media_files(
                music_folder_id,
                &music_folder_path,
                &song_fs_info
                    .keys()
                    .choose_multiple(&mut rand::thread_rng(), n_new_song)
                    .into_iter()
                    .map(|(_, path)| path)
                    .collect_vec(),
                fake::vec![SongTag; n_new_song],
            ),
        );

        let (upserted_song_count, deleted_song_count, _, _) =
            wrap_scan_full(temp_db.pool(), &music_folders).await;

        assert_eq!(upserted_song_count, n_new_song);
        assert_eq!(deleted_song_count, n_delete_song);
        assert_songs_info(temp_db.pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_multiple_folders() {
        let n_song = 25_usize;

        let (temp_db, _temp_fs, music_folders, song_fs_info) =
            setup_songs_no_scan(2, &[n_song, n_song], None).await;
        let (upserted_song_count, deleted_song_count, _, _) =
            wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(upserted_song_count, n_song + n_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(temp_db.pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_scan_combine_album_artists() {
        let (temp_db, _temp_fs, music_folders, song_fs_info) = setup_songs_no_scan(
            1,
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
        wrap_scan_full(temp_db.pool(), &music_folders).await;

        assert_albums_artists_info(temp_db.pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_delete_old_albums() {
        let n_song = 10;
        let n_delete_song = 2;
        let n_new_song = 4;

        let (temp_db, temp_fs, music_folders, mut song_fs_info) =
            setup_songs_no_scan(1, &[n_song], None).await;
        let (_, _, deleted_album_count, _) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(deleted_album_count, 0);
        assert_albums_info(temp_db.pool(), &song_fs_info).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        song_fs_info
            .keys()
            .cloned()
            .choose_multiple(&mut rand::thread_rng(), n_delete_song)
            .into_iter()
            .for_each(|key| {
                std::fs::remove_file(music_folder_path.join(&key.1)).unwrap();
                song_fs_info.remove(&key).unwrap();
            });

        song_fs_info.extend(
            temp_fs.create_nested_media_files(
                music_folder_id,
                &music_folder_path,
                &song_fs_info
                    .keys()
                    .choose_multiple(&mut rand::thread_rng(), n_new_song)
                    .into_iter()
                    .map(|(_, path)| path)
                    .collect_vec(),
                fake::vec![SongTag; n_new_song],
            ),
        );

        let (_, _, deleted_album_count, _) = wrap_scan_full(temp_db.pool(), &music_folders).await;

        assert_eq!(deleted_album_count, n_delete_song + n_new_song);
        assert_albums_info(temp_db.pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_scan_delete_keep_album_with_songs() {
        let (temp_db, _temp_fs, music_folders, song_fs_info) = setup_songs_no_scan(
            1,
            &[2],
            vec![
                SongTag {
                    album: "album".to_owned(),
                    ..Faker.fake()
                },
                SongTag {
                    album: "album".to_owned(),
                    ..Faker.fake()
                },
            ],
        )
        .await;
        let (_, _, deleted_album_count, _) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(deleted_album_count, 0);
        assert_album_names(temp_db.pool(), &["album"]).await;

        std::fs::remove_file(
            PathBuf::from(&music_folders[0].path).join(&song_fs_info.keys().next().unwrap().1),
        )
        .unwrap();

        let (_, _, deleted_album_count, _) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(deleted_album_count, 0);
        assert_album_names(temp_db.pool(), &["album"]).await;
    }

    #[tokio::test]
    async fn test_scan_all_artist() {
        let n_song = 10;
        let n_delete_song = 2;
        let n_new_song = 4;

        let (temp_db, temp_fs, music_folders, mut song_fs_info) =
            setup_songs_no_scan(1, &[n_song], None).await;
        wrap_scan_full(temp_db.pool(), &music_folders).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        assert_artists_info(temp_db.pool(), &song_fs_info).await;

        song_fs_info
            .keys()
            .cloned()
            .choose_multiple(&mut rand::thread_rng(), n_delete_song)
            .into_iter()
            .for_each(|key| {
                std::fs::remove_file(music_folder_path.join(&key.1)).unwrap();
                song_fs_info.remove(&key).unwrap();
            });

        song_fs_info.extend(
            temp_fs.create_nested_media_files(
                music_folder_id,
                &music_folder_path,
                &song_fs_info
                    .keys()
                    .choose_multiple(&mut rand::thread_rng(), n_new_song)
                    .into_iter()
                    .map(|(_, path)| path)
                    .collect_vec(),
                fake::vec![SongTag; n_new_song],
            ),
        );

        wrap_scan_full(temp_db.pool(), &music_folders).await;

        assert_artists_info(temp_db.pool(), &song_fs_info).await;
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
            SongTag {
                artists: vec!["artist2".to_owned()],
                ..Faker.fake()
            },
            // not deleted nor scanned
            SongTag {
                artists: vec!["artist3".to_owned()],
                ..Faker.fake()
            },
        ];
        let first_song_title = song_tags[0].title.clone();

        let (temp_db, temp_fs, music_folders, song_fs_info) =
            setup_songs_no_scan(1, &[3], song_tags).await;
        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let first_song_path = song_fs_info
            .iter()
            .find_map(|(k, v)| {
                if v.title == first_song_title {
                    Some(music_folder_path.join(&k.1))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_song_artist_names(temp_db.pool(), &["artist1", "artist2", "artist3"]).await;

        temp_fs.create_nested_media_file(
            Some(&music_folder_path),
            &first_song_path,
            SongTag {
                artists: vec!["artist2".to_owned()],
                ..Faker.fake()
            },
        );

        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(deleted_artist_count, 1);
        assert_song_artist_names(temp_db.pool(), &["artist2", "artist3"]).await;
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
        let first_song_title = song_tags[0].title.clone();

        let (temp_db, _temp_fs, music_folders, song_fs_info) =
            setup_songs_no_scan(1, &[2], song_tags).await;
        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let first_song_path = song_fs_info
            .iter()
            .find_map(|(k, v)| {
                if v.title == first_song_title {
                    Some(music_folder_path.join(&k.1))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(temp_db.pool(), &["artist1", "artist2", "artist3"]).await;

        std::fs::remove_file(music_folder_path.join(&first_song_path)).unwrap();

        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(temp_db.pool(), &["artist2", "artist3"]).await;
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
        let first_song_title = song_tags[0].title.clone();

        let (temp_db, _temp_fs, music_folders, song_fs_info) =
            setup_songs_no_scan(1, &[3], song_tags).await;
        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let first_song_path = song_fs_info
            .iter()
            .find_map(|(k, v)| {
                if v.title == first_song_title {
                    Some(music_folder_path.join(&k.1))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(temp_db.pool(), &["artist1", "artist2", "artist3"]).await;

        std::fs::remove_file(music_folder_path.join(&first_song_path)).unwrap();

        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(temp_db.pool(), &["artist2", "artist3"]).await;
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

        let (temp_db, temp_fs, music_folders, song_fs_info) =
            setup_songs_no_scan(1, &[3], song_tags).await;
        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let first_song_path = song_fs_info
            .iter()
            .find_map(|(k, v)| {
                if v.title == first_song_tag.title {
                    Some(music_folder_path.join(&k.1))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(temp_db.pool(), &["artist1", "artist2", "artist3"]).await;

        temp_fs.create_nested_media_file(
            Some(&music_folder_path),
            &first_song_path,
            SongTag {
                artists: vec!["artist2".to_owned()],
                album_artists: vec!["artist2".to_owned()],
                ..first_song_tag
            },
        );

        let (_, _, _, deleted_artist_count) = wrap_scan_full(temp_db.pool(), &music_folders).await;
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(temp_db.pool(), &["artist2", "artist3"]).await;
    }
}
