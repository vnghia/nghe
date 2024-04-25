use std::path::Path;

use flume::Sender;
use ignore::types::TypesBuilder;
use ignore::{DirEntry, Error, WalkBuilder};
use tracing::instrument;

use super::super::song::file_type::SONG_FILE_TYPES;
use crate::utils::path::LocalPath;
use crate::utils::song::file_type::{to_extension, to_glob_pattern};

fn process_dir_entry<P: AsRef<Path>>(
    root: P,
    tx: &Sender<LocalPath>,
    entry: Result<DirEntry, Error>,
) -> ignore::WalkState {
    match try {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let path = entry.path();
        if metadata.is_file()
            && let Err(e) = tx.send(LocalPath::new(root, path.to_path_buf(), metadata))
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
}

#[instrument(skip(tx))]
pub fn scan_media_files<P: AsRef<Path> + Clone + Send + std::fmt::Debug>(
    root: P,
    tx: Sender<LocalPath>,
    scan_parallel: bool,
) {
    tracing::info!("start scanning media files");

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

    if scan_parallel {
        WalkBuilder::new(&root).types(types).build_parallel().run(|| {
            let span = tracing::Span::current();
            let tx = tx.clone();
            let root = root.clone();
            Box::new(move |entry| {
                let _enter = span.enter();
                process_dir_entry(&root, &tx, entry)
            })
        });
    } else {
        for entry in WalkBuilder::new(&root).types(types).build() {
            process_dir_entry(&root, &tx, entry);
        }
    }

    tracing::info!("finish scanning media files");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use itertools::Itertools;
    use lofty::file::FileType;

    use super::*;
    use crate::utils::song::file_type::to_extensions;
    use crate::utils::test::Infra;

    async fn wrap_scan_media_file(infra: &Infra, scan_parallel: bool) -> Vec<LocalPath> {
        let (tx, rx) = flume::bounded(100);
        let root_path = infra.fs.root_path().to_path_buf();

        let scan_thread =
            tokio::task::spawn_blocking(move || scan_media_files(&root_path, tx, scan_parallel));
        let mut result = vec![];
        while let Ok(r) = rx.recv_async().await {
            result.push(r);
        }

        scan_thread.await.unwrap();
        result
    }

    #[tokio::test]
    async fn test_scan_media_files_no_filter() {
        let infra = Infra::new().await;

        let media_paths = Infra::create_random_relative_paths(50, 3, &to_extensions())
            .into_iter()
            .map(|path| infra.fs.create_file(path))
            .collect_vec();

        let scanned_results = wrap_scan_media_file(&infra, false).await;
        let scanned_lens =
            scanned_results.iter().cloned().map(|result| result.metadata.len()).collect_vec();
        let scanned_paths =
            scanned_results.iter().cloned().map(|result| result.absolute_path).collect_vec();

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
        let infra = Infra::new().await;

        let media_paths = Infra::create_random_relative_paths(50, 3, &to_extensions())
            .into_iter()
            .map(|path| {
                infra.fs.create_file(path).strip_prefix(infra.fs.root_path()).unwrap().to_path_buf()
            })
            .collect_vec();

        let scanned_paths = wrap_scan_media_file(&infra, false)
            .await
            .iter()
            .cloned()
            .map(|result| PathBuf::from(result.relative_path))
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_extension() {
        let infra = Infra::new().await;

        let supported_extensions = to_extensions();

        let media_paths = Infra::create_random_relative_paths(
            50,
            3,
            &[supported_extensions.as_slice(), &["txt", "rs"]].concat(),
        )
        .into_iter()
        .filter_map(|path| {
            let path = infra.fs.create_file(path);
            let ext = FileType::from_path(&path);
            if let Some(ext) = ext {
                if SONG_FILE_TYPES.contains(&ext) { Some(path) } else { None }
            } else {
                None
            }
        })
        .collect_vec();

        let scanned_paths = wrap_scan_media_file(&infra, false)
            .await
            .into_iter()
            .map(|result| result.absolute_path)
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_dir() {
        let infra = Infra::new().await;

        let media_paths = Infra::create_random_relative_paths(50, 3, &to_extensions())
            .into_iter()
            .filter_map(|path| {
                if rand::random::<bool>() {
                    Some(infra.fs.create_file(&path))
                } else {
                    infra.fs.create_dir(&path);
                    None
                }
            })
            .collect_vec();

        let scanned_paths = wrap_scan_media_file(&infra, false)
            .await
            .into_iter()
            .map(|result| result.absolute_path)
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_parallel() {
        let infra = Infra::new().await;

        let media_paths = Infra::create_random_relative_paths(50, 3, &to_extensions())
            .into_iter()
            .map(|path| infra.fs.create_file(path))
            .collect_vec();

        let scanned_results = wrap_scan_media_file(&infra, true).await;
        let scanned_lens =
            scanned_results.iter().cloned().map(|result| result.metadata.len()).collect_vec();
        let scanned_paths =
            scanned_results.iter().cloned().map(|result| result.absolute_path).collect_vec();

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
}
