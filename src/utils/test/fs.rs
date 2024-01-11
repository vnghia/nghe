mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use fake::{Fake, Faker};
use std::path::{Path, PathBuf};
use tempdir::TempDir;
use tokio::{fs::*, io::AsyncWriteExt};

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
}
