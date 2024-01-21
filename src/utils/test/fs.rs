mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use crate::models::*;
use crate::{open_subsonic::browsing::refresh_music_folders, DatabasePool};

use fake::{Fake, Faker};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use std::{fs::*, io::Write};
use tempdir::TempDir;

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

    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.root.path().join(path);
        create_dir_all(&path).expect("can not create temporary dir");
        path
    }

    pub fn create_file<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = self.root.path().join(path);
        self.create_dir(path.parent().unwrap());

        File::create(&path)
            .expect("can not open temporary file")
            .write_all(Faker.fake::<String>().as_bytes())
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
        pool: &DatabasePool,
        n_folder: u8,
    ) -> Vec<music_folders::MusicFolder> {
        let music_folder_paths = (0..n_folder)
            .map(|_| self.create_dir(Faker.fake::<String>()))
            .collect_vec();
        let (upserted_folders, _) = refresh_music_folders(pool, &music_folder_paths, &[]).await;
        upserted_folders
    }
}
