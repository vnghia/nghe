use crate::OSResult;

use itertools::Itertools;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MEDIA_EXTENSIONS: [&str; 2] = ["flac", "mp3"];

pub fn scan_media_files<P: AsRef<Path>>(root: P) -> OSResult<Vec<(PathBuf, u64)>> {
    Ok(WalkDir::new(&root)
        .into_iter()
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    if let Some(extension) = entry.path().extension() {
                        if MEDIA_EXTENSIONS.contains(&extension.to_string_lossy().as_ref()) {
                            match entry.metadata() {
                                Ok(metadata) => {
                                    if metadata.is_file() {
                                        return Some(Ok((
                                            entry.path().to_path_buf(),
                                            metadata.len(),
                                        )));
                                    }
                                }
                                Err(e) => {
                                    return Some(Err(e));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
            None
        })
        .try_collect()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::fs::TemporaryFs;

    use concat_string::concat_string;
    use fake::{Fake, Faker};

    #[test]
    fn test_scan_media_files_no_filter() {
        let fs = TemporaryFs::new();

        let media_paths = MEDIA_EXTENSIONS
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
            .map(|result| result.1)
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
    fn test_scan_media_files_filter_extension() {
        let fs = TemporaryFs::new();

        let media_paths = [MEDIA_EXTENSIONS, ["txt", "rs"]]
            .concat()
            .iter()
            .cartesian_product(0..3)
            .filter_map(|(extension, _)| {
                let path = fs.create_file(concat_string!(Faker.fake::<String>(), ".", extension));
                if MEDIA_EXTENSIONS.contains(extension) {
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

        let media_paths = MEDIA_EXTENSIONS
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
