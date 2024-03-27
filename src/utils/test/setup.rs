use std::path::PathBuf;
use std::slice::SliceIndex;

use axum::extract::State;
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use itertools::Itertools;
use uuid::Uuid;

use super::fs::SongFsInformation;
use super::user::create_users;
use super::{TemporaryDatabase, TemporaryFs};
use crate::config::{ArtistIndexConfig, ScanConfig};
use crate::models::*;
use crate::open_subsonic::browsing::{refresh_music_folders, refresh_permissions};
use crate::open_subsonic::scan::{start_scan, ScanMode};
use crate::utils::song::file_type::to_extensions;
use crate::utils::song::test::SongTag;
use crate::{Database, DatabasePool};

pub struct TestInfra {
    pub db: TemporaryDatabase,
    pub fs: TemporaryFs,
    pub users: Vec<users::User>,
    pub music_folders: Vec<music_folders::MusicFolder>,
}

impl TestInfra {
    pub async fn setup_users_and_music_folders_no_refresh(
        n_user: usize,
        n_folder: usize,
        allows: &[bool],
    ) -> (Self, Vec<user_music_folder_permissions::UserMusicFolderPermission>) {
        let (db, users) = create_users(n_user, 0).await;
        let fs = TemporaryFs::new();
        let music_folders = Self::setup_music_folders(&db, &fs, n_folder).await;

        let user_music_folder_permissions = (users.iter().cartesian_product(&music_folders))
            .zip(allows.iter())
            .map(|((user, music_folder), allow)| {
                user_music_folder_permissions::UserMusicFolderPermission {
                    user_id: user.id,
                    music_folder_id: music_folder.id,
                    allow: *allow,
                }
            })
            .collect_vec();

        (Self { db, fs, users, music_folders }, user_music_folder_permissions)
    }

    pub async fn setup_users_and_music_folders(
        n_user: usize,
        n_folder: usize,
        allows: &[bool],
    ) -> Self {
        let (test_infra, user_music_folder_permissions) =
            Self::setup_users_and_music_folders_no_refresh(n_user, n_folder, allows).await;

        diesel::insert_into(user_music_folder_permissions::table)
            .values(&user_music_folder_permissions)
            .execute(&mut test_infra.pool().get().await.unwrap())
            .await
            .unwrap();

        refresh_permissions(test_infra.pool(), None, None).await.unwrap();

        test_infra
    }

    pub async fn setup_songs_no_scan<S: Into<Option<Vec<SongTag>>>>(
        n_songs: &[usize],
        song_tags: S,
    ) -> (Self, Vec<SongFsInformation>) {
        let n_folder = n_songs.len();
        let test_infra = Self::setup_users_and_music_folders(0, n_folder, &[]).await;

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

        let song_fs_infos = test_infra
            .music_folders
            .iter()
            .zip(song_tags_vec.into_iter())
            .flat_map(|(music_folder, song_tags)| {
                let music_folder_path = PathBuf::from(&music_folder.path);
                test_infra.fs.create_random_paths_media_files(
                    music_folder_path,
                    song_tags,
                    &to_extensions(),
                )
            })
            .collect();

        (test_infra, song_fs_infos)
    }

    pub async fn setup_songs<S: Into<Option<Vec<SongTag>>>>(
        n_songs: &[usize],
        song_tags: S,
    ) -> (Self, Vec<SongFsInformation>) {
        let (test_infra, song_fs_infos) = Self::setup_songs_no_scan(n_songs, song_tags).await;
        start_scan(
            test_infra.pool(),
            ScanMode::Full,
            &test_infra.music_folders,
            &ArtistIndexConfig::default(),
            &test_infra.fs.parsing_config,
            &ScanConfig::default(),
        )
        .await
        .unwrap();
        (test_infra, song_fs_infos)
    }

    async fn setup_music_folders(
        db: &TemporaryDatabase,
        fs: &TemporaryFs,
        n_folder: usize,
    ) -> Vec<music_folders::MusicFolder> {
        let music_folder_paths =
            (0..n_folder).map(|_| fs.create_dir(Faker.fake::<String>())).collect_vec();
        let (upserted_folders, _) =
            refresh_music_folders(db.pool(), &music_folder_paths, &[]).await;
        upserted_folders
    }

    pub fn database(&self) -> &Database {
        self.db.database()
    }

    pub fn pool(&self) -> &DatabasePool {
        self.db.pool()
    }

    pub fn state(&self) -> State<Database> {
        self.db.state()
    }

    pub fn music_folder_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        self.music_folders[slice].as_ref().iter().map(|music_folder| music_folder.id).collect_vec()
    }
}
