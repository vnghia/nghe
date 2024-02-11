use super::db::TemporaryDatabase;
use super::fs::TemporaryFs;
use super::user::create_db_key_users;
use crate::config::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::browsing::refresh_permissions;
use crate::open_subsonic::scan::scan_full;
use crate::open_subsonic::user::password::MD5Token;
use crate::utils::song::file_type::to_extensions;
use crate::utils::song::tag::SongTag;

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
    EncryptionKey,
    Vec<(users::User, Vec<u8>, MD5Token)>,
    TemporaryFs,
    Vec<music_folders::MusicFolder>,
    Vec<user_music_folder_permissions::UserMusicFolderPermission>,
) {
    let (db, key, user_tokens) = create_db_key_users(n_user, 0).await;
    let temp_fs = TemporaryFs::new();
    let upserted_folders = temp_fs.create_music_folders(db.get_pool(), n_folder).await;
    let user_music_folder_permissions = (user_tokens.iter().cartesian_product(&upserted_folders))
        .zip(allows.iter())
        .map(|((user_token, upserted_folder), allow)| {
            user_music_folder_permissions::UserMusicFolderPermission {
                user_id: user_token.0.id,
                music_folder_id: upserted_folder.id,
                allow: *allow,
            }
        })
        .collect_vec();

    (
        db,
        key,
        user_tokens,
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
    EncryptionKey,
    Vec<(users::User, Vec<u8>, EncryptionKey)>,
    TemporaryFs,
    Vec<music_folders::MusicFolder>,
) {
    let (db, key, user_tokens, temp_fs, music_folders, user_music_folder_permissions) =
        setup_users_and_music_folders_no_refresh(n_user, n_folder, allows).await;

    diesel::insert_into(user_music_folder_permissions::table)
        .values(&user_music_folder_permissions)
        .execute(&mut db.get_pool().get().await.unwrap())
        .await
        .unwrap();

    refresh_permissions(db.get_pool(), None, None)
        .await
        .unwrap();

    (db, key, user_tokens, temp_fs, music_folders)
}

pub async fn setup_users_and_songs<S: Into<Option<Vec<SongTag>>>>(
    n_user: usize,
    n_folder: usize,
    allows: &[bool],
    n_songs: &[usize],
    song_tags: S,
) -> (
    TemporaryDatabase,
    Vec<(users::User, Vec<u8>, MD5Token)>,
    TemporaryFs,
    Vec<music_folders::MusicFolder>,
    HashMap<(Uuid, PathBuf), SongTag>,
    (usize, usize, usize, usize),
) {
    assert_eq!(n_songs.len(), n_folder);
    let (db, _, user_tokens, temp_fs, music_folders) =
        setup_users_and_music_folders(n_user, n_folder, allows).await;

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

    let scan_statistics = scan_full::<&str>(db.get_pool(), &[], &music_folders)
        .await
        .unwrap();

    (
        db,
        user_tokens,
        temp_fs,
        music_folders,
        song_fs_info,
        scan_statistics,
    )
}
