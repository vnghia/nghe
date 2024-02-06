use diesel::{sql_function, sql_types};

sql_function! (fn album_query_song_by_music_folder(music_folder_ids: sql_types::Array<sql_types::Uuid>, album_id: sql_types::Uuid) -> sql_types::Uuid);

sql_function! (fn album_query_song_by_user(user_id: sql_types::Uuid, album_id: sql_types::Uuid) -> sql_types::Uuid);

sql_function! (fn album_count_song_by_music_folder(music_folder_ids: sql_types::Array<sql_types::Uuid>, album_id: sql_types::Uuid) -> sql_types::BigInt);

sql_function! (fn album_count_song_by_user(user_id: sql_types::Uuid, album_id: sql_types::Uuid) -> sql_types::BigInt);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::*,
        open_subsonic::{
            browsing::{refresh_permissions, test::setup_user_and_music_folders},
            scan::{album::upsert_album, scan_full},
        },
        utils::{
            song::{file_type::to_extensions, tag::SongTag},
            test::media::song_paths_to_ids,
        },
    };

    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use std::{collections::HashMap, path::PathBuf};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_empty_query() {
        let (db, _, _, _temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
        let album_id = upsert_album(db.get_pool(), "album".into()).await.unwrap();
        let song_ids = diesel::select(album_query_song_by_music_folder(
            &music_folder_ids,
            &album_id,
        ))
        .get_results::<Uuid>(&mut db.get_pool().get().await.unwrap())
        .await
        .unwrap();
        let song_count = diesel::select(album_count_song_by_music_folder(
            &music_folder_ids,
            &album_id,
        ))
        .first::<i64>(&mut db.get_pool().get().await.unwrap())
        .await
        .unwrap();

        assert!(song_ids.is_empty());
        assert_eq!(song_count, 0);
    }

    #[tokio::test]
    async fn test_simple_query() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 1, &[]).await;

        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();

        let n_song = 50_usize;
        let music_folder_id = music_folder_ids[0];
        let music_folder_path = PathBuf::from(&music_folders[0].path);

        let album_name = "album".to_owned();
        let album_id = upsert_album(db.get_pool(), (&album_name).into())
            .await
            .unwrap();

        let song_fs_info = temp_fs.create_nested_random_paths_media_files(
            music_folder_id,
            &music_folder_path,
            (0..n_song)
                .map(|_| SongTag {
                    album: album_name.clone(),
                    ..Faker.fake()
                })
                .collect_vec(),
            &to_extensions(),
        );
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        let song_fs_ids = song_paths_to_ids(db.get_pool(), &song_fs_info).await;

        let song_ids = diesel::select(album_query_song_by_music_folder(
            &music_folder_ids,
            &album_id,
        ))
        .get_results::<Uuid>(&mut db.get_pool().get().await.unwrap())
        .await
        .unwrap()
        .into_iter()
        .sorted()
        .collect_vec();
        let song_count = diesel::select(album_count_song_by_music_folder(
            &music_folder_ids,
            &album_id,
        ))
        .first::<i64>(&mut db.get_pool().get().await.unwrap())
        .await
        .unwrap();

        assert_eq!(song_ids, song_fs_ids);
        assert_eq!(song_count as usize, n_song);
    }

    #[tokio::test]
    async fn test_simple_query_with_multiple_folders() {
        let (db, _, _, temp_fs, music_folders, _) = setup_user_and_music_folders(0, 2, &[]).await;

        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();

        let n_songs = [20_usize, 30_usize];

        let album_name = "album".to_owned();
        let album_id = upsert_album(db.get_pool(), (&album_name).into())
            .await
            .unwrap();

        let song_fs_info = music_folders
            .iter()
            .zip(n_songs.iter().copied())
            .flat_map(|(music_folder, n_song)| {
                let music_folder_id = music_folder.id;
                let music_folder_path = PathBuf::from(&music_folder.path);
                temp_fs.create_nested_random_paths_media_files(
                    music_folder_id,
                    &music_folder_path,
                    (0..n_song)
                        .map(|_| SongTag {
                            album: album_name.clone(),
                            ..Faker.fake()
                        })
                        .collect_vec(),
                    &to_extensions(),
                )
            })
            .collect::<HashMap<_, _>>();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        let song_fs_ids = song_paths_to_ids(db.get_pool(), &song_fs_info).await;

        let song_ids = diesel::select(album_query_song_by_music_folder(
            &music_folder_ids,
            &album_id,
        ))
        .get_results::<Uuid>(&mut db.get_pool().get().await.unwrap())
        .await
        .unwrap()
        .into_iter()
        .sorted()
        .collect_vec();
        let song_count = diesel::select(album_count_song_by_music_folder(
            &music_folder_ids,
            &album_id,
        ))
        .first::<i64>(&mut db.get_pool().get().await.unwrap())
        .await
        .unwrap();

        assert_eq!(song_ids, song_fs_ids);
        assert_eq!(song_count as usize, n_songs[0] + n_songs[1]);

        for (music_folder_id, n_song) in music_folder_ids.iter().zip(n_songs.iter().copied()) {
            let song_count = diesel::select(album_count_song_by_music_folder(
                [music_folder_id].as_slice(),
                &album_id,
            ))
            .first::<i64>(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();
            assert_eq!(song_count as usize, n_song);
        }
    }

    #[tokio::test]
    async fn test_simple_query_by_user() {
        let (db, _, user_tokens, temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(1, 2, &[true, false]).await;

        diesel::insert_into(user_music_folder_permissions::table)
            .values(&permissions)
            .execute(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        refresh_permissions(db.get_pool(), None, None)
            .await
            .unwrap();

        let n_songs = [20_usize, 30_usize];

        let album_name = "album".to_owned();
        let album_id = upsert_album(db.get_pool(), (&album_name).into())
            .await
            .unwrap();

        let song_fs_info = music_folders
            .iter()
            .zip(n_songs.iter().copied())
            .enumerate()
            .flat_map(|(i, (music_folder, n_song))| {
                let music_folder_id = music_folder.id;
                let music_folder_path = PathBuf::from(&music_folder.path);
                let song_fs_info = temp_fs.create_nested_random_paths_media_files(
                    music_folder_id,
                    &music_folder_path,
                    (0..n_song)
                        .map(|_| SongTag {
                            album: album_name.clone(),
                            ..Faker.fake()
                        })
                        .collect_vec(),
                    &to_extensions(),
                );
                if i == 0 {
                    song_fs_info
                } else {
                    HashMap::default()
                }
            })
            .collect::<HashMap<_, _>>();
        scan_full::<&str>(db.get_pool(), &[], &music_folders)
            .await
            .unwrap();
        let song_fs_ids = song_paths_to_ids(db.get_pool(), &song_fs_info).await;

        let song_ids = diesel::select(album_query_song_by_user(&user_tokens[0].0.id, &album_id))
            .get_results::<Uuid>(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();
        let song_count = diesel::select(album_count_song_by_user(&user_tokens[0].0.id, &album_id))
            .first::<i64>(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        assert_eq!(song_ids, song_fs_ids);
        assert_eq!(song_count as usize, n_songs[0]);
    }
}
