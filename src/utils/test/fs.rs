mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use fake::{Fake, Faker};
use sea_orm::DatabaseConnection;
use std::path::{Path, PathBuf};
use tempdir::TempDir;
use tokio::{fs::*, io::AsyncWriteExt};

use crate::entity::*;
use crate::open_subsonic::browsing::refresh_music_folders;

pub struct TemporaryFs {
    root: TempDir,
}

#[allow(clippy::new_without_default)]
impl TemporaryFs {
    pub fn new() -> Self {
        Self {
            root: TempDir::new(built_info::PKG_NAME).expect("can not create temporary directory"),
        }
    }

    pub async fn create_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.root.path().join(path);
        create_dir_all(&path)
            .await
            .expect("can not create temporary dir");
        path
    }

    pub async fn create_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.root.path().join(path);
        self.create_dir(path.parent().unwrap()).await;

        File::create(&path)
            .await
            .expect("can not open temporary file")
            .write_all(Faker.fake::<String>().as_bytes())
            .await
            .expect("can not write to temporary file");
        path
    }

    pub fn join_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<PathBuf> {
        paths
            .iter()
            .map(|path| self.root.path().join(path))
            .collect()
    }

    pub fn canonicalize_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Vec<PathBuf> {
        paths
            .iter()
            .map(std::fs::canonicalize)
            .collect::<Result<Vec<_>, _>>()
            .expect("can not canonicalize temp path")
    }

    pub fn get_root_path(&self) -> &Path {
        self.root.path()
    }

    pub async fn create_music_folders(
        &self,
        conn: &DatabaseConnection,
        n_folder: u8,
    ) -> Vec<music_folder::Model> {
        let mut music_folder_paths = Vec::<PathBuf>::default();
        for _ in 0..n_folder {
            let dir_path = Faker.fake::<String>();
            music_folder_paths.push(self.create_dir(&dir_path).await);
        }
        let (upserted_folders, _) = refresh_music_folders(conn, &music_folder_paths, &[]).await;
        upserted_folders
    }
}
