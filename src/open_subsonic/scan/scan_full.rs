use super::{
    album::upsert_album, artist::upsert_artists, song::upsert_song, song::upsert_song_artists,
};
use crate::{
    models::*,
    utils::{fs::files::scan_media_files, song::tag::SongTag},
    DatabasePool, OSResult,
};

use diesel::{ExpressionMethods, OptionalExtension};
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

            let artist_ids = upsert_artists(pool, ignored_prefixes, &song_tag.artists).await?;
            let album_id = upsert_album(pool, std::borrow::Cow::Borrowed(&song_tag.album)).await?;

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

    tracing::info!("done scanning songs");
    Ok(())
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use itertools::Itertools;

    use super::*;
    use crate::{
        open_subsonic::browsing::test::setup_user_and_music_folders,
        utils::{song::file_type::to_extensions, test::song::query_all_songs_information},
    };

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
            .zip(fake::vec![SongTag; n_song as usize].iter().cloned())
            .map(|((path, _), song_tag)| {
                (
                    (
                        music_folder_id,
                        temp_fs
                            .create_nested_media_file(
                                Some(&music_folder_path),
                                path,
                                song_tag.clone(),
                            )
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
        let song_db_info = query_all_songs_information(db.get_pool()).await;

        assert_eq!(
            song_fs_info.keys().into_iter().sorted().collect_vec(),
            song_db_info.keys().into_iter().sorted().collect_vec(),
        );

        for (song_key, song_tag) in song_fs_info {
            let (song, album, artists) = song_db_info.get(&song_key).unwrap();
            assert_eq!(song_tag.title, song.title);
            assert_eq!(song_tag.album, album.name);
            assert_eq!(
                song_tag.artists.into_iter().sorted().collect_vec(),
                artists
                    .into_iter()
                    .map(|artist| artist.name.clone())
                    .sorted()
                    .collect_vec()
            );
        }
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
            .zip(fake::vec![SongTag; n_song as usize].iter().cloned())
            .map(|((path, _), song_tag)| {
                (
                    temp_fs.create_nested_media_file(
                        Some(&music_folder_path),
                        path,
                        song_tag.clone(),
                    ),
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
                        new_song_tag.clone(),
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

        let song_db_info = query_all_songs_information(db.get_pool()).await;

        assert_eq!(
            song_fs_info.keys().into_iter().sorted().collect_vec(),
            song_db_info.keys().into_iter().sorted().collect_vec(),
        );

        for (song_key, song_tag) in song_fs_info {
            let (song, album, artists) = song_db_info.get(&song_key).unwrap();
            assert_eq!(song_tag.title, song.title);
            assert_eq!(song_tag.album, album.name);
            assert_eq!(
                song_tag.artists.into_iter().sorted().collect_vec(),
                artists
                    .into_iter()
                    .map(|artist| artist.name.clone())
                    .sorted()
                    .collect_vec()
            );
        }
    }

    #[tokio::test]
    async fn test_simple_scan_with_mutiple_folders() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 2, &[]).await;

        let n_song = 25;
        let song_fs_info = music_folders
            .iter()
            .map(|music_folder| {
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
                    .zip(fake::vec![SongTag; n_song as usize].iter().cloned())
                    .map(|((path, _), song_tag)| {
                        (
                            (
                                music_folder_id,
                                temp_fs
                                    .create_nested_media_file(
                                        Some(&music_folder_path),
                                        path,
                                        song_tag.clone(),
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
            .flatten()
            .collect::<HashMap<_, _>>();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        let song_db_info = query_all_songs_information(db.get_pool()).await;

        assert_eq!(
            song_fs_info.keys().into_iter().sorted().collect_vec(),
            song_db_info.keys().into_iter().sorted().collect_vec(),
        );

        for (song_key, song_tag) in song_fs_info {
            let (song, album, artists) = song_db_info.get(&song_key).unwrap();
            assert_eq!(song_tag.title, song.title);
            assert_eq!(song_tag.album, album.name);
            assert_eq!(
                song_tag.artists.into_iter().sorted().collect_vec(),
                artists
                    .into_iter()
                    .map(|artist| artist.name.clone())
                    .sorted()
                    .collect_vec()
            );
        }
    }
}
