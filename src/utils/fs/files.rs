use std::path::{Path, PathBuf};

use crossfire::channel::MPSCShared;
use crossfire::mpsc;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use tracing::instrument;

use super::super::song::file_type::SONG_FILE_TYPES;
use crate::utils::song::file_type::{to_extension, to_glob_pattern};

#[cfg_attr(test, derive(Clone))]
pub struct ScannedMediaFile {
    pub song_absolute_path: PathBuf,
    pub song_relative_path: String,
    pub song_file_size: u64,
}

impl ScannedMediaFile {
    pub fn new<P: AsRef<Path>>(root: P, song_absolute_path: PathBuf, song_file_size: u64) -> Self {
        let song_relative_path = song_absolute_path
            .strip_prefix(&root)
            .expect("this path should always contains the root path")
            .to_str()
            .expect("non utf-8 path encountered")
            .to_string();
        Self { song_absolute_path, song_relative_path, song_file_size }
    }
}

#[instrument(skip(tx))]
pub fn scan_media_files<P: AsRef<Path> + Clone + Send + std::fmt::Debug, S: MPSCShared>(
    root: P,
    tx: mpsc::TxBlocking<ScannedMediaFile, S>,
) {
    tracing::debug!("start scanning");

    let types = match try {
        let mut types = TypesBuilder::new();
        for song_file_type in SONG_FILE_TYPES {
            types.add(to_extension(&song_file_type), to_glob_pattern(&song_file_type))?;
        }
        types.select("all").build()?
    } {
        Ok(r) => r,
        Err::<_, anyhow::Error>(e) => {
            tracing::error!(building_scan_pattern = ?e);
            return;
        }
    };

    WalkBuilder::new(&root).types(types).build_parallel().run(|| {
        let span = tracing::Span::current();
        let tx = tx.clone();
        let root = root.clone();

        Box::new(move |entry| {
            let _enter = span.enter();

            match try {
                let entry = entry?;
                let metadata = entry.metadata()?;
                let path = entry.path();
                if metadata.is_file()
                    && let Err(e) =
                        tx.send(ScannedMediaFile::new(&root, path.to_path_buf(), metadata.len()))
                {
                    tracing::error!(sending_walkdir_result = ?e);
                    ignore::WalkState::Quit
                } else {
                    ignore::WalkState::Continue
                }
            } {
                Ok(r) => r,
                Err::<_, anyhow::Error>(e) => {
                    tracing::error!(walking_media_directory = ?e);
                    ignore::WalkState::Continue
                }
            }
        })
    });

    tracing::debug!("finish scanning");
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::utils::song::file_type::to_extensions;
    use crate::utils::test::fs::TemporaryFs;

    async fn wrap_scan_media_file(fs: &TemporaryFs) -> Vec<ScannedMediaFile> {
        let (tx, rx) = mpsc::bounded_tx_blocking_rx_future(100);
        let root_path = fs.root_path().to_path_buf();

        let scan_thread = tokio::task::spawn_blocking(move || scan_media_files(&root_path, tx));
        let mut result = vec![];
        while let Ok(r) = rx.recv().await {
            result.push(r);
        }

        scan_thread.await.unwrap();
        result
    }

    #[tokio::test]
    async fn test_scan_media_files_no_filter() {
        let fs = TemporaryFs::new();

        let media_paths = TemporaryFs::create_random_relative_paths(50, 3, &to_extensions())
            .into_iter()
            .map(|path| fs.create_file(path))
            .collect_vec();

        let scanned_results = wrap_scan_media_file(&fs).await;
        let scanned_lens =
            scanned_results.iter().cloned().map(|result| result.song_file_size).collect_vec();
        let scanned_paths =
            scanned_results.iter().cloned().map(|result| result.song_absolute_path).collect_vec();

        assert_eq!(
            media_paths
                .iter()
                .map(|path| std::fs::metadata(path).unwrap().len())
                .sorted()
                .collect_vec(),
            scanned_lens.into_iter().sorted().collect_vec()
        );
        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_relative_path() {
        let fs = TemporaryFs::new();

        let media_paths = TemporaryFs::create_random_relative_paths(50, 3, &to_extensions())
            .into_iter()
            .map(|path| fs.create_file(path).strip_prefix(fs.root_path()).unwrap().to_path_buf())
            .collect_vec();

        let scanned_paths = wrap_scan_media_file(&fs)
            .await
            .iter()
            .cloned()
            .map(|result| PathBuf::from(result.song_relative_path))
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_extension() {
        let fs = TemporaryFs::new();

        let supported_extensions = to_extensions();

        let media_paths = TemporaryFs::create_random_relative_paths(
            50,
            3,
            &[supported_extensions.as_slice(), &["txt", "rs"]].concat(),
        )
        .into_iter()
        .filter_map(|path| {
            let path = fs.create_file(path);
            let ext = lofty::FileType::from_path(&path);
            if let Some(ext) = ext {
                if SONG_FILE_TYPES.contains(&ext) { Some(path) } else { None }
            } else {
                None
            }
        })
        .collect_vec();

        let scanned_paths = wrap_scan_media_file(&fs)
            .await
            .into_iter()
            .map(|result| result.song_absolute_path)
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_dir() {
        let fs = TemporaryFs::new();

        let media_paths = TemporaryFs::create_random_relative_paths(50, 3, &to_extensions())
            .into_iter()
            .filter_map(|path| {
                if rand::random::<bool>() {
                    Some(fs.create_file(&path))
                } else {
                    fs.create_dir(&path);
                    None
                }
            })
            .collect_vec();

        let scanned_paths = wrap_scan_media_file(&fs)
            .await
            .into_iter()
            .map(|result| result.song_absolute_path)
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }
}
