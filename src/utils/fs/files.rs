use super::super::song::file_type::SONG_FILE_TYPES;
use crate::{
    utils::song::file_type::{to_extension, to_glob_pattern},
    OSResult,
};

use ignore::{types::TypesBuilder, WalkBuilder};
use itertools::Itertools;
use lofty::FileType;
use std::path::{Path, PathBuf};

pub fn scan_media_files<P: AsRef<Path> + Clone + Send>(
    root: P,
) -> OSResult<Vec<(PathBuf, String, FileType, u64)>> {
    let (tx, rx) = crossbeam_channel::unbounded::<(PathBuf, String, FileType, u64)>();

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
                                    .to_string_lossy()
                                    .to_string(),
                                FileType::from_ext(
                                    entry_path
                                        .extension()
                                        .expect("the file should have an extension"),
                                )
                                .expect("the extension should be covered in file type"),
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

    use concat_string::concat_string;
    use fake::{Fake, Faker};

    #[test]
    fn test_scan_media_files_no_filter() {
        let fs = TemporaryFs::new();

        let media_paths = to_extensions()
            .iter()
            .cartesian_product(0..3)
            .map(|(extension, _)| {
                fs.create_file(concat_string!(Faker.fake::<String>(), ".", extension))
            })
            .collect_vec();

        let scanned_results = scan_media_files(fs.get_root_path()).unwrap();
        let scanned_lens = scanned_results
            .iter()
            .cloned()
            .map(|result| result.3)
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

        let media_paths = to_extensions()
            .iter()
            .cartesian_product(0..3)
            .map(|(extension, _)| {
                let relative_path =
                    PathBuf::from(concat_string!(Faker.fake::<String>(), ".", extension));
                fs.create_file(&relative_path);
                relative_path
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

        let media_paths = [supported_extensions.as_slice(), &["txt", "rs"]]
            .concat()
            .iter()
            .cartesian_product(0..3)
            .filter_map(|(extension, _)| {
                let path = fs.create_file(concat_string!(Faker.fake::<String>(), ".", extension));
                if supported_extensions.contains(extension) {
                    Some(path)
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

        let media_paths = to_extensions()
            .iter()
            .cartesian_product(0..5)
            .filter_map(|(extension, i)| {
                if i < 3 {
                    Some(fs.create_file(concat_string!(Faker.fake::<String>(), ".", extension)))
                } else {
                    fs.create_dir(concat_string!(Faker.fake::<String>(), ".", extension));
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
