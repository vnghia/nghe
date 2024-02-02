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

pub async fn scan_full<T: AsRef<str>>(
    pool: &DatabasePool,
    ignored_prefixes: &[T],
    music_folders: &[music_folders::MusicFolder],
) -> OSResult<()> {
    let scan_start_time = time::OffsetDateTime::now_utc();

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

            upsert_album_artists(
                pool,
                album_id,
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
                .filter(albums_artists::upserted_at.lt(scan_start_time))
                .execute(&mut pool.get().await?)
                .await?;

            let song_id = upsert_song(
                pool,
                song_id,
                song_tag.into_new_or_update_song(
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
            upsert_song_artists(pool, song_id, &artist_ids).await?;
            diesel::delete(songs_artists::table)
                .filter(songs_artists::song_id.eq(song_id))
                .filter(songs_artists::upserted_at.lt(scan_start_time))
                .execute(&mut pool.get().await?)
                .await?;
        }
    }

    let albums_no_song = diesel::alias!(albums as albums_no_song);
    diesel::delete(albums::table)
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

    build_artist_indices(pool, ignored_prefixes).await?;

    tracing::info!("done scanning songs");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::browsing::test::setup_user_and_music_folders,
        utils::{
            song::file_type::{to_extension, to_extensions},
            test::media::query_all_songs_information,
        },
    };

    use diesel::QueryDsl;
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use lofty::FileType;
    use std::{collections::HashMap, path::PathBuf};

    #[tokio::test]
    async fn test_simple_scan() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 50;
        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let song_fs_info = temp_fs
            .create_nested_random_paths(Some(&music_folder_path), n_song, 3, &to_extensions())
            .iter()
            .zip(fake::vec![SongTag; n_song as usize].into_iter())
            .map(|((path, _), song_tag)| {
                (
                    (
                        music_folder_id,
                        temp_fs
                            .create_nested_media_file(Some(&music_folder_path), path, &song_tag)
                            .strip_prefix(&music_folder_path)
                            .unwrap()
                            .to_path_buf(),
                    ),
                    song_tag,
                )
            })
            .collect::<HashMap<_, _>>();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        let mut song_db_info = query_all_songs_information(db.get_pool()).await;

        for (song_key, song_tag) in song_fs_info {
            let (song, album, artists, album_artists) = song_db_info.remove(&song_key).unwrap();
            assert_eq!(song_tag.title, song.title);
            assert_eq!(song_tag.album, album.name);
            assert_eq!(
                song_tag.artists.into_iter().sorted().collect_vec(),
                artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .sorted()
                    .collect_vec()
            );
            assert_eq!(
                song_tag.album_artists.into_iter().sorted().collect_vec(),
                album_artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .sorted()
                    .collect_vec()
            );
        }
        assert!(song_db_info.is_empty());
    }

    #[tokio::test]
    async fn test_simple_scan_with_update() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 50;
        let n_new_song = 20;
        let music_folder_id = music_folders[0].id;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let song_fs_info = temp_fs
            .create_nested_random_paths(Some(&music_folder_path), n_song, 3, &to_extensions())
            .iter()
            .zip(fake::vec![SongTag; n_song as usize].into_iter())
            .map(|((path, _), song_tag)| {
                (
                    temp_fs.create_nested_media_file(Some(&music_folder_path), path, &song_tag),
                    song_tag,
                )
            })
            .collect_vec();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        let song_fs_info = song_fs_info
            .into_iter()
            .enumerate()
            .map(|(i, (path, song_tag))| {
                let (path, song_tag) = if i < n_new_song {
                    let new_song_tag = Faker.fake::<SongTag>();
                    let new_path = temp_fs.create_nested_media_file(
                        Some(&music_folder_path),
                        &path,
                        &new_song_tag,
                    );
                    (new_path, new_song_tag)
                } else {
                    (path, song_tag)
                };
                (
                    (
                        music_folder_id,
                        path.strip_prefix(&music_folder_path).unwrap().to_path_buf(),
                    ),
                    song_tag,
                )
            })
            .collect::<HashMap<_, _>>();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        let mut song_db_info = query_all_songs_information(db.get_pool()).await;

        for (song_key, song_tag) in song_fs_info {
            let (song, album, artists, album_artists) = song_db_info.remove(&song_key).unwrap();
            assert_eq!(song_tag.title, song.title);
            assert_eq!(song_tag.album, album.name);
            assert_eq!(
                song_tag.artists.into_iter().sorted().collect_vec(),
                artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .sorted()
                    .collect_vec()
            );
            assert_eq!(
                song_tag.album_artists.into_iter().sorted().collect_vec(),
                album_artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .sorted()
                    .collect_vec()
            );
        }
        assert!(song_db_info.is_empty());
    }

    #[tokio::test]
    async fn test_simple_scan_with_multiple_folders() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 2, &[]).await;

        let n_song = 25;
        let song_fs_info = music_folders
            .iter()
            .flat_map(|music_folder| {
                let music_folder_id = music_folder.id;
                let music_folder_path = PathBuf::from(&music_folder.path);
                temp_fs
                    .create_nested_random_paths(
                        Some(&music_folder_path),
                        n_song,
                        3,
                        &to_extensions(),
                    )
                    .iter()
                    .zip(fake::vec![SongTag; n_song as usize].into_iter())
                    .map(|((path, _), song_tag)| {
                        (
                            (
                                music_folder_id,
                                temp_fs
                                    .create_nested_media_file(
                                        Some(&music_folder_path),
                                        path,
                                        &song_tag,
                                    )
                                    .strip_prefix(&music_folder_path)
                                    .unwrap()
                                    .to_path_buf(),
                            ),
                            song_tag,
                        )
                    })
                    .collect_vec()
            })
            .collect::<HashMap<_, _>>();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        let mut song_db_info = query_all_songs_information(db.get_pool()).await;

        for (song_key, song_tag) in song_fs_info {
            let (song, album, artists, album_artists) = song_db_info.remove(&song_key).unwrap();
            assert_eq!(song_tag.title, song.title);
            assert_eq!(song_tag.album, album.name);
            assert_eq!(
                song_tag.artists.into_iter().sorted().collect_vec(),
                artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .sorted()
                    .collect_vec()
            );
            assert_eq!(
                song_tag.album_artists.into_iter().sorted().collect_vec(),
                album_artists
                    .into_iter()
                    .map(|artist| artist.name)
                    .sorted()
                    .collect_vec()
            );
        }
        assert!(song_db_info.is_empty());
    }

    #[tokio::test]
    async fn test_scan_combine_album_artists() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let album_name = "album".to_owned();
        let song_tags = vec![
            SongTag {
                album: album_name.clone(),
                album_artists: ["artist1", "artist2"]
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect_vec(),
                ..Faker.fake()
            },
            SongTag {
                album: album_name.clone(),
                album_artists: ["artist1", "artist3"]
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect_vec(),
                ..Faker.fake()
            },
        ];
        temp_fs
            .create_nested_random_paths(
                Some(&music_folder_path),
                2,
                1,
                &[to_extension(&FileType::Flac)],
            )
            .iter()
            .zip(song_tags.iter())
            .for_each(|((path, _), song_tag)| {
                temp_fs.create_nested_media_file(Some(&music_folder_path), path, song_tag);
            });
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();

        assert_eq!(
            song_tags
                .into_iter()
                .flat_map(|song_tag| song_tag.album_artists)
                .unique()
                .sorted()
                .collect_vec(),
            albums_artists::table
                .inner_join(artists::table)
                .inner_join(albums::table)
                .select(artists::name)
                .filter(albums::name.eq(&album_name))
                .load::<String>(&mut db.get_pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec(),
        );
    }

    #[tokio::test]
    async fn test_simple_scan_delete_old_albums() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let n_song = 10;
        let n_new_song = 4;
        let music_folder_path = PathBuf::from(&music_folders[0].path);
        let song_fs_info = temp_fs
            .create_nested_random_paths(Some(&music_folder_path), n_song, 3, &to_extensions())
            .iter()
            .zip(fake::vec![SongTag; n_song as usize].into_iter())
            .map(|((path, _), song_tag)| {
                (
                    temp_fs.create_nested_media_file(Some(&music_folder_path), path, &song_tag),
                    song_tag,
                )
            })
            .collect_vec();
        let song_fs_albums = song_fs_info
            .iter()
            .map(|(_, song_tag)| song_tag.album.clone())
            .unique()
            .sorted()
            .collect_vec();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(
            song_fs_albums,
            albums::table
                .select(albums::name)
                .load::<String>(&mut db.get_pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec(),
        );

        let song_fs_albums = song_fs_info
            .into_iter()
            .enumerate()
            .map(|(i, (path, song_tag))| {
                if i < n_new_song {
                    let new_song_tag = Faker.fake::<SongTag>();
                    temp_fs.create_nested_media_file(Some(&music_folder_path), path, &new_song_tag);
                    new_song_tag.album
                } else {
                    song_tag.album
                }
            })
            .unique()
            .sorted()
            .collect_vec();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        assert_eq!(
            song_fs_albums,
            albums::table
                .select(albums::name)
                .load::<String>(&mut db.get_pool().get().await.unwrap())
                .await
                .unwrap()
                .into_iter()
                .sorted()
                .collect_vec(),
        );
    }
}
