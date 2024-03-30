use std::slice::SliceIndex;

use axum::extract::State;
use fake::{Fake, Faker};
use itertools::Itertools;
use uuid::Uuid;

use super::{TemporaryDatabase, TemporaryFs};
use crate::database::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::browsing::refresh_music_folders;
use crate::open_subsonic::test::CommonParams;
use crate::open_subsonic::user::test::set_music_folder_permissions;
use crate::{Database, DatabasePool};

pub struct Infra {
    pub db: TemporaryDatabase,
    pub fs: TemporaryFs,
    pub users: Vec<users::User>,
    pub music_folders: Vec<music_folders::MusicFolder>,
}

impl Infra {
    pub async fn new() -> Self {
        let db = TemporaryDatabase::new_from_env().await;
        let fs = TemporaryFs::new();
        Self { db, fs, users: vec![], music_folders: vec![] }
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
}
