use super::user::create_users;
use super::{TemporaryDatabase, TemporaryFs};
use crate::config::ArtistIndexConfig;
use crate::models::*;
use crate::open_subsonic::browsing::refresh_permissions;
use crate::open_subsonic::scan::{run_scan, ScanMode};
use crate::utils::song::file_type::to_extensions;
use crate::utils::song::test::SongTag;

use diesel_async::RunQueryDsl;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn setup_users_and_music_folders_no_refresh(
    n_user: usize,
    n_folder: usize,
    allows: &[bool],
) -> (
    TemporaryDatabase,
    Vec<users::User>,
    TemporaryFs,
    Vec<music_folders::MusicFolder>,
    Vec<user_music_folder_permissions::UserMusicFolderPermission>,
) {
    let (temp_db, users) = create_users(n_user, 0).await;
    let temp_fs = TemporaryFs::new();
    let upserted_folders = temp_fs.create_music_folders(temp_db.pool(), n_folder).await;
    let user_music_folder_permissions = (users.iter().cartesian_product(&upserted_folders))
        .zip(allows.iter())
        .map(|((user, upserted_folder), allow)| {
            user_music_folder_permissions::UserMusicFolderPermission {
                user_id: user.id,
                music_folder_id: upserted_folder.id,
                allow: *allow,
            }
        })
        .collect_vec();

    (
        temp_db,
        users,
        temp_fs,
        upserted_folders,
        user_music_folder_permissions,
    )
}

pub async fn setup_users_and_music_folders(
    n_user: usize,
    n_folder: usize,
    allows: &[bool],
) -> (
    TemporaryDatabase,
    Vec<users::User>,
    TemporaryFs,
    Vec<music_folders::MusicFolder>,
) {
    let (temp_db, users, temp_fs, music_folders, user_music_folder_permissions) =
        setup_users_and_music_folders_no_refresh(n_user, n_folder, allows).await;

    diesel::insert_into(user_music_folder_permissions::table)
        .values(&user_music_folder_permissions)
        .execute(&mut temp_db.pool().get().await.unwrap())
        .await
        .unwrap();

    refresh_permissions(temp_db.pool(), None, None)
        .await
        .unwrap();

    (temp_db, users, temp_fs, music_folders)
}

pub async fn setup_songs_no_scan<S: Into<Option<Vec<SongTag>>>>(
    n_folder: usize,
    n_songs: &[usize],
    song_tags: S,
) -> (
    TemporaryDatabase,
    TemporaryFs,
    Vec<music_folders::MusicFolder>,
    HashMap<(Uuid, PathBuf), SongTag>,
) {
    assert_eq!(n_songs.len(), n_folder);
    let (temp_db, _, temp_fs, music_folders) =
        setup_users_and_music_folders(0, n_folder, &[]).await;

    let n_song_total: usize = n_songs.iter().sum();
    let mut song_tags = match song_tags.into() {
        Some(song_tags) => song_tags,
        None => fake::vec![SongTag; n_song_total],
    };
    assert_eq!(song_tags.len(), n_song_total);

    let mut song_tags_vec = Vec::<Vec<SongTag>>::default();
    for n_song in n_songs.iter().rev().copied() {
        song_tags_vec.push(song_tags.split_off(song_tags.len() - n_song));
    }
    assert!(song_tags.is_empty());
    let song_tags_vec = song_tags_vec.into_iter().rev().collect_vec();

    let song_fs_info = music_folders
        .iter()
        .zip(song_tags_vec.into_iter())
        .flat_map(|(music_folder, song_tags)| {
            let music_folder_id = music_folder.id;
            let music_folder_path = PathBuf::from(&music_folder.path);
            temp_fs.create_nested_random_paths_media_files(
                music_folder_id,
                &music_folder_path,
                song_tags,
                &to_extensions(),
            )
        })
        .collect::<HashMap<_, _>>();

    (temp_db, temp_fs, music_folders, song_fs_info)
}

pub async fn setup_songs<S: Into<Option<Vec<SongTag>>>>(
    n_folder: usize,
    n_songs: &[usize],
    song_tags: S,
) -> (
    TemporaryDatabase,
    TemporaryFs,
    Vec<music_folders::MusicFolder>,
    HashMap<(Uuid, PathBuf), SongTag>,
) {
    let (temp_db, temp_fs, music_folders, song_fs_info) =
        setup_songs_no_scan(n_folder, n_songs, song_tags).await;
    run_scan(
        temp_db.pool(),
        ScanMode::Full,
        &ArtistIndexConfig::default(),
        &music_folders,
        &temp_fs.parsing_config,
    )
    .await
    .unwrap();
    (temp_db, temp_fs, music_folders, song_fs_info)
}
