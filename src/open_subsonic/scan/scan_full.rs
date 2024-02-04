use super::{
    album::upsert_album, album::upsert_album_artists, artist::build_artist_indices,
    artist::upsert_artists, song::upsert_song, song::upsert_song_artists,
};
use crate::{
    models::*,
    utils::{fs::files::scan_media_files, song::tag::SongTag},
    DatabasePool, OSResult,
};

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

pub async fn scan_full<S: AsRef<str>>(
    pool: &DatabasePool,
    ignored_prefixes: &[S],
    music_folders: &[music_folders::MusicFolder],
) -> OSResult<(usize, usize, usize, usize)> {
    let scan_start_time = time::OffsetDateTime::now_utc();

    let mut upserted_song_count: usize = 0;

    for music_folder in music_folders {
        let music_folder_path = music_folder.path.clone();
        for (song_absolute_path, song_relative_path, song_file_type, song_file_size) in
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

            let song_tag = SongTag::parse(&song_data, song_file_type)?;

            let artist_ids = upsert_artists(pool, &song_tag.artists).await?;
            let album_id = upsert_album(pool, std::borrow::Cow::Borrowed(&song_tag.album)).await?;

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

            upsert_album_artists(
                pool,
                album_id,
                song_id,
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
            diesel::delete(albums_artists::table)
                .filter(albums_artists::album_id.eq(album_id))
                .filter(albums_artists::song_id.eq(song_id))
                .filter(albums_artists::upserted_at.lt(scan_start_time))
                .execute(&mut pool.get().await?)
                .await?;

            upsert_song_artists(pool, song_id, &artist_ids).await?;
            diesel::delete(songs_artists::table)
                .filter(songs_artists::song_id.eq(song_id))
                .filter(songs_artists::upserted_at.lt(scan_start_time))
                .execute(&mut pool.get().await?)
                .await?;

            upserted_song_count += 1;
        }
    }

    let deleted_song_count = diesel::delete(songs::table)
        .filter(songs::scanned_at.lt(scan_start_time))
        .execute(&mut pool.get().await?)
        .await?;

    let albums_no_song = diesel::alias!(albums as albums_no_song);
    let deleted_album_count = diesel::delete(albums::table)
        .filter(
            albums::id.eq_any(
                albums_no_song
                    .left_join(songs::table)
                    .filter(songs::id.is_null())
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
                    .left_join(albums_artists::table)
                    .left_join(songs_artists::table)
                    .filter(albums_artists::album_id.is_null())
                    .filter(songs_artists::song_id.is_null())
                    .select(artists_no_song_no_album.field(artists::id)),
            ),
        )
        .execute(&mut pool.get().await?)
        .await?;

    build_artist_indices(pool, ignored_prefixes).await?;

    tracing::info!("done scanning songs");
    Ok((
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
        open_subsonic::browsing::test::setup_user_and_music_folders,
        utils::{
            song::file_type::{to_extension, to_extensions},
            test::media::{
                assert_album_artist_names, assert_album_names, assert_albums_artists_info,
                assert_albums_info, assert_artists_info, assert_song_artist_names,
                assert_songs_info,
            },
        },
    };

    use fake::{Fake, Faker};
    use itertools::{concat, Itertools};
    use lofty::FileType;
    use rand::seq::IteratorRandom;
    use std::{collections::HashMap, path::PathBuf};

    #[tokio::test]
    async fn test_simple_scan() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 50_usize;
        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let song_fs_info = temp_fs.create_nested_random_paths_media_files(
            music_folder_id,
            &music_folder_path,
            fake::vec![SongTag; n_song],
            &to_extensions(),
        );
        let (upserted_song_count, deleted_song_count, _, _) =
            scan_full::<&str>(db.get_pool(), &[], &music_folders)
                .await
                .unwrap();

        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(db.get_pool(), song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_update_same_path() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 50_usize;
        let n_new_song = 20_usize;
        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let song_fs_info = temp_fs.create_nested_random_paths_media_files(
            music_folder_id,
            &music_folder_path,
            fake::vec![SongTag; n_song],
            &to_extensions(),
        );
        let (upserted_song_count, deleted_song_count, _, _) =
            scan_full::<&str>(db.get_pool(), &[], &music_folders)
                .await
                .unwrap();
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        let song_fs_info = concat(vec![
            song_fs_info.clone(),
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
        ]);
        let (upserted_song_count, deleted_song_count, _, _) =
            scan_full::<&str>(db.get_pool(), &[], &music_folders)
                .await
                .unwrap();

        assert_eq!(upserted_song_count, n_new_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(db.get_pool(), song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_delete() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 50_usize;
        let n_delete_song = 10_usize;
        let n_new_song = 20_usize;
        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let mut song_fs_info = temp_fs.create_nested_random_paths_media_files(
            music_folder_id,
            &music_folder_path,
            fake::vec![SongTag; n_song],
            &to_extensions(),
        );
        let (upserted_song_count, deleted_song_count, _, _) =
            scan_full::<&str>(db.get_pool(), &[], &music_folders)
                .await
                .unwrap();
        assert_eq!(upserted_song_count, n_song);
        assert_eq!(deleted_song_count, 0);

        song_fs_info
            .clone()
            .into_keys()
            .choose_multiple(&mut rand::thread_rng(), n_delete_song)
            .into_iter()
            .for_each(|key| {
                std::fs::remove_file(music_folder_path.join(&key.1)).unwrap();
                song_fs_info.remove(&key).unwrap();
            });

        let song_fs_info = concat(vec![
            song_fs_info.clone(),
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
        ]);
        let (upserted_song_count, deleted_song_count, _, _) =
            scan_full::<&str>(db.get_pool(), &[], &music_folders)
                .await
                .unwrap();

        assert_eq!(upserted_song_count, n_new_song);
        assert_eq!(deleted_song_count, n_delete_song);
        assert_songs_info(db.get_pool(), song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_with_multiple_folders() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 2, &[]).await;

        let n_song = 25_usize;
        let song_fs_info = music_folders
            .iter()
            .flat_map(|music_folder| {
                let music_folder_id = music_folder.id;
                let music_folder_path = PathBuf::from(&music_folder.path);
                temp_fs.create_nested_random_paths_media_files(
                    music_folder_id,
                    &music_folder_path,
                    fake::vec![SongTag; n_song],
                    &to_extensions(),
                )
            })
            .collect::<HashMap<_, _>>();
        let (upserted_song_count, deleted_song_count, _, _) =
            scan_full::<&str>(db.get_pool(), &[], &music_folders)
                .await
                .unwrap();

        assert_eq!(upserted_song_count, n_song + n_song);
        assert_eq!(deleted_song_count, 0);
        assert_songs_info(db.get_pool(), song_fs_info).await;
    }

    #[tokio::test]
    async fn test_scan_combine_album_artists() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let song_tags = vec![
            SongTag {
                album: "album".to_string(),
                album_artists: vec!["artist1".to_string(), "artist2".to_string()],
                ..Faker.fake()
            },
            SongTag {
                album: "album".to_string(),
                album_artists: vec!["artist1".to_string(), "artist3".to_string()],
                ..Faker.fake()
            },
        ];
        let song_fs_info = temp_fs.create_nested_random_paths_media_files(
            music_folder_id,
            &music_folder_path,
            song_tags.clone(),
            &[to_extension(&FileType::Flac)],
        );
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        assert_albums_artists_info(db.get_pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_simple_scan_delete_old_albums() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 10;
        let n_delete_song = 2;
        let n_new_song = 4;
        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let mut song_fs_info = temp_fs.create_nested_random_paths_media_files(
            music_folder_id,
            &music_folder_path,
            fake::vec![SongTag; n_song as usize],
            &to_extensions(),
        );
        let (_, _, deleted_album_count, _) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        assert_eq!(deleted_album_count, 0);
        assert_albums_info(db.get_pool(), &song_fs_info).await;

        song_fs_info
            .clone()
            .into_keys()
            .choose_multiple(&mut rand::thread_rng(), n_delete_song)
            .into_iter()
            .for_each(|key| {
                std::fs::remove_file(music_folder_path.join(&key.1)).unwrap();
                song_fs_info.remove(&key).unwrap();
            });

        let song_fs_info = concat(vec![
            song_fs_info.clone(),
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
        ]);
        let (_, _, deleted_album_count, _) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        assert_eq!(deleted_album_count, n_delete_song + n_new_song);
        assert_albums_info(db.get_pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_scan_delete_keep_album_with_songs() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let song_tags = vec![
            SongTag {
                album: "album".to_string(),
                ..Faker.fake()
            },
            SongTag {
                album: "album".to_string(),
                ..Faker.fake()
            },
        ];
        let song_paths = temp_fs
            .create_nested_random_paths(
                Some(&music_folder_path),
                song_tags.len() as u8,
                3,
                &[to_extension(&FileType::Flac)],
            )
            .into_iter()
            .map(|(path, _)| path)
            .collect_vec();
        temp_fs.create_nested_media_files(
            music_folder_id,
            &music_folder_path,
            &song_paths,
            song_tags.clone(),
        );

        let (_, _, deleted_album_count, _) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_album_count, 0);
        assert_album_names(db.get_pool(), &["album"]).await;

        std::fs::remove_file(music_folder_path.join(&song_paths[0])).unwrap();

        let (_, _, deleted_album_count, _) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_album_count, 0);
        assert_album_names(db.get_pool(), &["album"]).await;
    }

    #[tokio::test]
    async fn test_scan_all_artist() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 10;
        let n_delete_song = 2;
        let n_new_song = 4;
        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let mut song_fs_info = temp_fs.create_nested_random_paths_media_files(
            music_folder_id,
            &music_folder_path,
            fake::vec![SongTag; n_song as usize],
            &to_extensions(),
        );
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        assert_artists_info(db.get_pool(), &song_fs_info).await;

        song_fs_info
            .clone()
            .into_keys()
            .choose_multiple(&mut rand::thread_rng(), n_delete_song)
            .into_iter()
            .for_each(|key| {
                std::fs::remove_file(music_folder_path.join(&key.1)).unwrap();
                song_fs_info.remove(&key).unwrap();
            });

        let song_fs_info = concat(vec![
            song_fs_info.clone(),
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
        ]);
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        assert_artists_info(db.get_pool(), &song_fs_info).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_song_artists() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let song_tags = vec![
            // deleted
            SongTag {
                artists: vec!["artist1".to_string()],
                album_artists: vec!["artist1".to_string()],
                ..Faker.fake()
            },
            // not deleted but scanned (artist2)
            SongTag {
                artists: vec!["artist2".to_string()],
                ..Faker.fake()
            },
            // not deleted nor scanned
            SongTag {
                artists: vec!["artist3".to_string()],
                ..Faker.fake()
            },
        ];
        let song_paths = temp_fs
            .create_nested_random_paths(
                Some(&music_folder_path),
                song_tags.len() as u8,
                3,
                &[to_extension(&FileType::Flac)],
            )
            .into_iter()
            .map(|(path, _)| path)
            .collect_vec();
        temp_fs.create_nested_media_files(
            music_folder_id,
            &music_folder_path,
            &song_paths,
            song_tags.clone(),
        );

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_song_artist_names(db.get_pool(), &["artist1", "artist2", "artist3"]).await;

        temp_fs.create_nested_media_file(
            Some(&music_folder_path),
            &song_paths[0],
            &SongTag {
                artists: vec!["artist2".to_string()],
                ..Faker.fake()
            },
        );

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 1);
        assert_song_artist_names(db.get_pool(), &["artist2", "artist3"]).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_album_artists() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let song_tags = vec![
            SongTag {
                album: "album1".to_string(),
                artists: vec!["artist2".to_string()],
                album_artists: vec!["artist1".to_string(), "artist2".to_string()],
                ..Faker.fake()
            },
            SongTag {
                album: "album2".to_string(),
                album_artists: vec!["artist2".to_string(), "artist3".to_string()],
                ..Faker.fake()
            },
        ];
        let song_paths = temp_fs
            .create_nested_random_paths(
                Some(&music_folder_path),
                song_tags.len() as u8,
                3,
                &[to_extension(&FileType::Flac)],
            )
            .into_iter()
            .map(|(path, _)| path)
            .collect_vec();
        temp_fs.create_nested_media_files(
            music_folder_id,
            &music_folder_path,
            &song_paths,
            song_tags.clone(),
        );

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(db.get_pool(), &["artist1", "artist2", "artist3"]).await;

        std::fs::remove_file(music_folder_path.join(&song_paths[0])).unwrap();

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(db.get_pool(), &["artist2", "artist3"]).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_combined_album_artists_with_delete() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let song_tags = vec![
            // deleted
            SongTag {
                album: "album".to_string(),
                artists: vec!["artist1".to_string(), "artist2".to_string()],
                album_artists: vec!["artist1".to_string()],
                ..Faker.fake()
            },
            // not deleted but scanned (artist2)
            SongTag {
                album: "album".to_string(),
                album_artists: vec!["artist2".to_string()],
                ..Faker.fake()
            },
            // not deleted nor scanned
            SongTag {
                album: "album".to_string(),
                album_artists: vec!["artist3".to_string()],
                ..Faker.fake()
            },
        ];
        let song_paths = temp_fs
            .create_nested_random_paths(
                Some(&music_folder_path),
                song_tags.len() as u8,
                3,
                &[to_extension(&FileType::Flac)],
            )
            .into_iter()
            .map(|(path, _)| path)
            .collect_vec();
        temp_fs.create_nested_media_files(
            music_folder_id,
            &music_folder_path,
            &song_paths,
            song_tags.clone(),
        );

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(db.get_pool(), &["artist1", "artist2", "artist3"]).await;

        std::fs::remove_file(music_folder_path.join(&song_paths[0])).unwrap();

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(db.get_pool(), &["artist2", "artist3"]).await;
    }

    #[tokio::test]
    async fn test_scan_delete_old_combined_album_artists_with_update() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let song_tags = vec![
            // deleted
            SongTag {
                album: "album".to_string(),
                artists: vec!["artist1".to_string(), "artist2".to_string()],
                album_artists: vec!["artist1".to_string()],
                ..Faker.fake()
            },
            // not deleted but scanned (artist2)
            SongTag {
                album: "album".to_string(),
                album_artists: vec!["artist2".to_string()],
                ..Faker.fake()
            },
            // not deleted nor scanned
            SongTag {
                album: "album".to_string(),
                album_artists: vec!["artist3".to_string()],
                ..Faker.fake()
            },
        ];
        let song_paths = temp_fs
            .create_nested_random_paths(
                Some(&music_folder_path),
                song_tags.len() as u8,
                3,
                &[to_extension(&FileType::Flac)],
            )
            .into_iter()
            .map(|(path, _)| path)
            .collect_vec();
        temp_fs.create_nested_media_files(
            music_folder_id,
            &music_folder_path,
            &song_paths,
            song_tags.clone(),
        );

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 0);
        assert_album_artist_names(db.get_pool(), &["artist1", "artist2", "artist3"]).await;

        temp_fs.create_nested_media_file(
            Some(&music_folder_path),
            &song_paths[0],
            &SongTag {
                artists: vec!["artist2".to_string()],
                album_artists: vec!["artist2".to_string()],
                ..song_tags[0].clone()
            },
        );

        let (_, _, _, deleted_artist_count) = scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(deleted_artist_count, 1);
        assert_album_artist_names(db.get_pool(), &["artist2", "artist3"]).await;
    }
}
