use super::super::song::file_type::SONG_FILE_TYPES;
use crate::utils::song::file_type::{to_extension, to_glob_pattern};

use anyhow::Result;
use ignore::{types::TypesBuilder, WalkBuilder};
use itertools::Itertools;
use std::path::{Path, PathBuf};

pub fn scan_media_files<P: AsRef<Path> + Clone + Send>(
    root: P,
) -> Result<Vec<(PathBuf, String, u64)>> {
    let (tx, rx) = crossbeam_channel::unbounded::<(PathBuf, String, u64)>();

    let mut types = TypesBuilder::new();
    for song_file_type in SONG_FILE_TYPES {
        types.add(
            to_extension(&song_file_type),
            to_glob_pattern(&song_file_type),
        )?;
    }
    let types = types.select("all").build()?;

    WalkBuilder::new(&root)
        .types(types)
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            let root = root.clone();
            Box::new(move |entry| match entry {
                Ok(entry) => match entry.metadata() {
                    Ok(metadata) => {
                        if metadata.is_file() {
                            let entry_path = entry.path();
                            if let Err(e) = tx.send((
                                entry_path.into(),
                                entry_path
                                    .strip_prefix(&root)
                                    .expect("this path should always contains the root path")
                                    .to_str()
                                    .expect("non utf-8 path encountered")
                                    .to_string(),
                                metadata.len(),
                            )) {
                                tracing::info!("error {} while scanning for media files", e);
                            }
                        }
                        ignore::WalkState::Continue
                    }
                    Err(e) => {
                        tracing::info!("error {} while scanning for media files", e);
                        ignore::WalkState::Continue
                    }
                },
                Err(e) => {
                    tracing::info!("error {} while scanning for media files", e);
                    ignore::WalkState::Continue
                }
            })
        });
    drop(tx);

    Ok(rx.into_iter().collect_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{song::file_type::to_extensions, test::fs::TemporaryFs};
    use std::path::PathBuf;

    #[test]
    fn test_scan_media_files_no_filter() {
        let fs = TemporaryFs::new();

        let media_paths = fs
            .create_random_paths(50, 3, &to_extensions())
            .into_iter()
            .map(|path| fs.create_file(path))
            .collect_vec();

        let scanned_results = scan_media_files(fs.get_root_path()).unwrap();
        let scanned_lens = scanned_results
            .iter()
            .cloned()
            .map(|result| result.2)
            .collect_vec();
        let scanned_paths = scanned_results
            .iter()
            .cloned()
            .map(|result| result.0)
            .collect_vec();

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

    #[test]
    fn test_scan_media_files_relative_path() {
        let fs = TemporaryFs::new();

        let media_paths = fs
            .create_random_paths(50, 3, &to_extensions())
            .into_iter()
            .map(|path| {
                fs.create_file(path)
                    .strip_prefix(fs.get_root_path())
                    .unwrap()
                    .to_path_buf()
            })
            .collect_vec();

        let scanned_paths = scan_media_files(fs.get_root_path())
            .unwrap()
            .iter()
            .cloned()
            .map(|result| PathBuf::from(result.1))
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[test]
    fn test_scan_media_files_filter_extension() {
        let fs = TemporaryFs::new();

        let supported_extensions = to_extensions();

        let media_paths = fs
            .create_random_paths(
                50,
                3,
                &[supported_extensions.as_slice(), &["txt", "rs"]].concat(),
            )
            .into_iter()
            .filter_map(|path| {
                let path = fs.create_file(path);
                let ext = lofty::FileType::from_path(&path);
                if let Some(ext) = ext {
                    if SONG_FILE_TYPES.contains(&ext) {
                        Some(path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect_vec();

        let scanned_paths = scan_media_files(fs.get_root_path())
            .unwrap()
            .into_iter()
            .map(|result| result.0)
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[test]
    fn test_scan_media_files_filter_dir() {
        let fs = TemporaryFs::new();

        let media_paths = fs
            .create_random_paths(50, 3, &to_extensions())
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

        let scanned_paths = scan_media_files(fs.get_root_path())
            .unwrap()
            .into_iter()
            .map(|result| result.0)
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }
}
