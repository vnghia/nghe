use std::path::Path;
use std::slice::SliceIndex;

use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use fake::{Fake, Faker};
use futures::stream::{self, StreamExt};
use itertools::Itertools;
use uuid::Uuid;

use super::fs::SongFsInformation;
use super::{random, TemporaryDatabase, TemporaryFs};
use crate::config::{ArtConfig, ArtistIndexConfig, ScanConfig};
use crate::database::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::browsing::refresh_music_folders;
use crate::open_subsonic::scan::{start_scan, ScanMode, ScanStatistic};
use crate::open_subsonic::test::CommonParams;
use crate::open_subsonic::user::set_music_folder_permissions;
use crate::utils::song::file_type::to_extensions;
use crate::utils::song::test::SongTag;
use crate::{Database, DatabasePool};

pub struct Infra {
    pub db: TemporaryDatabase,
    pub fs: TemporaryFs,
    pub users: Vec<users::User>,
    pub music_folders: Vec<music_folders::MusicFolder>,
    pub song_fs_infos_vec: Vec<Vec<SongFsInformation>>,
}

impl Infra {
    pub async fn new() -> Self {
        let db = TemporaryDatabase::new_from_env().await;
        let fs = TemporaryFs::new();
        Self { db, fs, users: vec![], music_folders: vec![], song_fs_infos_vec: vec![] }
    }

    pub async fn add_user(mut self, admin_role: Option<bool>) -> Self {
        self.users.push(users::User::fake(admin_role).create(self.database()).await);
        let user_index = self.users.len() - 1;
        if !self.music_folders.is_empty() {
            self.permissions(user_index..=user_index, .., true).await;
        }
        self
    }

    pub async fn n_folder(mut self, n_folder: usize) -> Self {
        if !self.music_folders.is_empty() {
            panic!("n_folder should be called only once")
        } else {
            let music_folder_paths =
                (0..n_folder).map(|_| self.fs.create_dir(Faker.fake::<String>())).collect_vec();
            let (upserted_folders, _) =
                refresh_music_folders(self.pool(), &music_folder_paths, &[]).await;
            self.music_folders = upserted_folders;
            self.song_fs_infos_vec = vec![vec![]; n_folder];
            self
        }
    }

    pub async fn permissions<SU, SM>(&self, user_slice: SU, music_folder_slice: SM, allow: bool)
    where
        SU: SliceIndex<[users::User], Output = [users::User]>,
        SM: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        set_music_folder_permissions(
            self.pool(),
            &self.user_ids(user_slice),
            &self.music_folder_ids(music_folder_slice),
            allow,
        )
        .await
        .unwrap();
    }

    pub async fn scan<S>(&self, slice: S, scan_mode: Option<ScanMode>) -> ScanStatistic
    where
        S: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        start_scan(
            self.pool(),
            scan_mode.unwrap_or(ScanMode::Full),
            self.music_folders[slice].as_ref(),
            &ArtistIndexConfig::default(),
            &self.fs.parsing_config,
            &ScanConfig::default(),
            &ArtConfig::default(),
        )
        .await
        .unwrap()
    }

    pub fn add_n_song(&mut self, index: usize, n_song: usize) -> &mut Self {
        self.add_songs(index, fake::vec![SongTag; n_song])
    }

    pub fn add_songs(&mut self, index: usize, song_tags: Vec<SongTag>) -> &mut Self {
        self.song_fs_infos_vec[index].extend(self.fs.create_random_paths_media_files(
            &self.music_folders[index].path,
            song_tags,
            &to_extensions(),
        ));
        self
    }

    pub fn delete_songs(&mut self, index: usize, delete_mask: &[bool]) -> &mut Self {
        self.song_fs_infos_vec[index] = delete_mask
            .iter()
            .copied()
            .zip(std::mem::take(&mut self.song_fs_infos_vec[index]))
            .filter_map(|(d, s)| {
                if d {
                    std::fs::remove_file(s.absolute_path()).unwrap();
                    None
                } else {
                    Some(s)
                }
            })
            .collect();
        self
    }

    pub fn delete_song(&mut self, music_folder_index: usize, song_index: usize) -> &mut Self {
        let mut delete_mask = vec![false; self.song_fs_infos_vec[music_folder_index].len()];
        delete_mask[song_index] = true;
        self.delete_songs(music_folder_index, &delete_mask)
    }

    pub fn delete_n_song(&mut self, index: usize, n_song: usize) -> &mut Self {
        self.delete_songs(
            index,
            &random::gen_bool_mask(self.song_fs_infos_vec[index].len(), n_song),
        )
    }

    pub fn update_songs(
        &mut self,
        index: usize,
        update_mask: &[bool],
        song_tags: Vec<SongTag>,
    ) -> &mut Self {
        let new_song_fs_infos = self.fs.create_media_files(
            &self.music_folders[index].path,
            update_mask
                .iter()
                .copied()
                .zip(self.song_fs_infos_vec[index].iter())
                .filter_map(|(u, s)| if u { Some(s.relative_path.clone()) } else { None })
                .collect(),
            song_tags,
        );
        update_mask
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(i, u)| if u { Some(i) } else { None })
            .zip(new_song_fs_infos.into_iter())
            .for_each(|(i, s)| {
                self.song_fs_infos_vec[index][i] = s;
            });
        self
    }

    pub fn update_song(
        &mut self,
        music_folder_index: usize,
        song_index: usize,
        song_tag: SongTag,
    ) -> &mut Self {
        let mut update_mask = vec![false; self.song_fs_infos_vec[music_folder_index].len()];
        update_mask[song_index] = true;
        self.update_songs(music_folder_index, &update_mask, vec![song_tag])
    }

    pub fn update_n_song(&mut self, index: usize, n_song: usize) -> &mut Self {
        self.update_songs(
            index,
            &random::gen_bool_mask(self.song_fs_infos_vec[index].len(), n_song),
            fake::vec![SongTag; n_song],
        )
    }

    pub fn copy_song<P: AsRef<Path>>(
        &mut self,
        music_folder_index: usize,
        src_index: usize,
        dst_path: P,
    ) -> &mut Self {
        let music_folder_path = Path::new(&self.music_folders[music_folder_index].path);

        let old_song_tag = self.song_fs_infos_vec[music_folder_index][src_index].clone();
        let old_song_path = Path::new(&old_song_tag.relative_path);

        let new_song_path = dst_path.as_ref().with_extension(old_song_path.extension().unwrap());
        assert!(!new_song_path.is_absolute());

        std::fs::copy(
            music_folder_path.join(&old_song_path),
            music_folder_path.join(&new_song_path),
        )
        .unwrap();

        self.song_fs_infos_vec[music_folder_index].push(SongFsInformation {
            relative_path: new_song_path.to_str().unwrap().to_owned(),
            ..old_song_tag
        });
        self
    }

    pub fn database(&self) -> &Database {
        self.db.database()
    }

    pub fn pool(&self) -> &DatabasePool {
        &self.database().pool
    }

    pub fn key(&self) -> &EncryptionKey {
        &self.database().key
    }

    pub fn state(&self) -> State<Database> {
        self.db.state()
    }

    pub fn user_id(&self, index: usize) -> Uuid {
        self.user_ids(index..=index)[0]
    }

    pub fn user_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[users::User], Output = [users::User]>,
    {
        self.users[slice].as_ref().iter().map(|u| u.id).sorted().collect_vec()
    }

    pub fn to_common_params(&self, index: usize) -> CommonParams {
        self.users[index].to_common_params(&self.key())
    }

    pub fn music_folder_id(&self, index: usize) -> Uuid {
        self.music_folder_ids(index..=index)[0]
    }

    pub fn music_folder_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[music_folders::MusicFolder], Output = [music_folders::MusicFolder]>,
    {
        self.music_folders[slice].as_ref().iter().map(|f| f.id).sorted().collect_vec()
    }

    pub fn song_fs_infos<S>(&self, slice: S) -> Vec<SongFsInformation>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        self.song_fs_infos_vec[slice].as_ref().iter().flat_map(|v| v.clone()).collect()
    }

    pub async fn song_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        stream::iter(self.song_fs_infos(slice))
            .then(|song_fs_info| async move {
                songs::table
                    .select(songs::id)
                    .inner_join(music_folders::table)
                    .filter(
                        music_folders::path.eq(&song_fs_info.music_folder_path.to_str().unwrap()),
                    )
                    .filter(songs::file_hash.eq(song_fs_info.file_hash as i64))
                    .filter(songs::file_size.eq(song_fs_info.file_size as i64))
                    .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                    .await
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .sorted()
            .collect()
    }

    pub async fn artist_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        let artist_names = self
            .song_fs_infos(slice)
            .iter()
            .flat_map(|s| [s.tag.album_artists.clone(), s.tag.artists.clone()].concat())
            .unique()
            .collect_vec();
        artists::table
            .select(artists::id)
            .filter(artists::name.eq_any(&artist_names))
            .get_results::<Uuid>(&mut self.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec()
    }

    pub async fn album_ids<S>(&self, slice: S) -> Vec<Uuid>
    where
        S: SliceIndex<[Vec<SongFsInformation>], Output = [Vec<SongFsInformation>]>,
    {
        stream::iter(self.song_fs_infos(slice))
            .then(|song_fs_info| async move {
                songs::table
                    .select(songs::album_id)
                    .inner_join(music_folders::table)
                    .filter(
                        music_folders::path.eq(&song_fs_info.music_folder_path.to_str().unwrap()),
                    )
                    .filter(songs::file_hash.eq(song_fs_info.file_hash as i64))
                    .filter(songs::file_size.eq(song_fs_info.file_size as i64))
                    .get_result::<Uuid>(&mut self.pool().get().await.unwrap())
                    .await
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .unique()
            .sorted()
            .collect_vec()
    }
}
