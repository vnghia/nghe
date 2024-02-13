use diesel::{sql_function, sql_types};

sql_function! (fn album_query_song_by_music_folder(music_folder_ids: sql_types::Array<sql_types::Uuid>, album_id: sql_types::Uuid) -> sql_types::Uuid);

sql_function! (fn album_query_song_by_user(user_id: sql_types::Uuid, album_id: sql_types::Uuid) -> sql_types::Uuid);

sql_function! (fn album_count_song_by_music_folder(music_folder_ids: sql_types::Array<sql_types::Uuid>, album_id: sql_types::Uuid) -> sql_types::BigInt);

sql_function! (fn album_count_song_by_user(user_id: sql_types::Uuid, album_id: sql_types::Uuid) -> sql_types::BigInt);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::scan::album::upsert_album,
        utils::{
            song::tag::SongTag,
            test::{media::song_paths_to_ids, setup::setup_users_and_songs},
        },
    };

    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_empty_query() {
        let (db, _, _temp_fs, music_folders, _, _) =
            setup_users_and_songs(0, 1, &[], &[0], None).await;

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
        let n_song = 50_usize;
        let album_name = "album".to_owned();

        let (db, _, _temp_fs, music_folders, song_fs_info, _) = setup_users_and_songs(
            0,
            1,
            &[],
            &[n_song],
            (0..n_song)
                .map(|_| SongTag {
                    album: album_name.clone(),
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let album_id = upsert_album(db.get_pool(), (&album_name).into())
            .await
            .unwrap();
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
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
        let n_songs = [20_usize, 30_usize];
        let n_song: usize = n_songs.iter().sum();
        let album_name = "album".to_owned();

        let (db, _, _temp_fs, music_folders, song_fs_info, _) = setup_users_and_songs(
            0,
            2,
            &[],
            &n_songs,
            (0..n_song)
                .map(|_| SongTag {
                    album: album_name.clone(),
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let album_id = upsert_album(db.get_pool(), (&album_name).into())
            .await
            .unwrap();
        let music_folder_ids = music_folders
            .iter()
            .map(|music_folder| music_folder.id)
            .collect_vec();
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
        let n_songs = [20_usize, 30_usize];
        let n_song: usize = n_songs.iter().sum();
        let album_name = "album".to_owned();

        let (db, users, _temp_fs, music_folders, song_fs_info, _) = setup_users_and_songs(
            1,
            2,
            &[true, false],
            &n_songs,
            (0..n_song)
                .map(|_| SongTag {
                    album: album_name.clone(),
                    ..Faker.fake()
                })
                .collect_vec(),
        )
        .await;

        let album_id = upsert_album(db.get_pool(), (&album_name).into())
            .await
            .unwrap();
        let music_folder_id = music_folders[0].id;
        let song_fs_ids = song_paths_to_ids(
            db.get_pool(),
            &song_fs_info
                .into_iter()
                .filter(|(k, _)| k.0 == music_folder_id)
                .collect(),
        )
        .await;

        let song_ids = diesel::select(album_query_song_by_user(&users[0].id, &album_id))
            .get_results::<Uuid>(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();
        let song_count = diesel::select(album_count_song_by_user(&users[0].id, &album_id))
            .first::<i64>(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        assert_eq!(song_ids, song_fs_ids);
        assert_eq!(song_count as usize, n_songs[0]);
    }
}
