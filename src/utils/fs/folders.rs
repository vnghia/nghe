use std::fs::*;
use std::path::{Path, PathBuf};

use concat_string::concat_string;
use itertools::Itertools;
use walkdir::WalkDir;

fn get_deepest_folders<P: AsRef<Path> + Sync>(root: P, max_depth: usize) -> Vec<PathBuf> {
    let entries = WalkDir::new(&root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|entry| {
            entry
                .metadata()
                .expect(&concat_string!(
                    "can not read metadata of ",
                    entry.path().to_str().expect("non utf-8 path encountered")
                ))
                .is_dir()
        })
        .collect::<Result<Vec<_>, _>>()
        .expect(&concat_string!(
            "can not traverse ",
            root.as_ref().to_str().expect("non utf-8 path encountered")
        ));

    (0..entries.len())
        .filter_map(|i| {
            if i == entries.len() - 1 || !entries[i + 1].path().starts_with(entries[i].path()) {
                // if it is not a children of the `previous_entry`,
                // it means that the `previous_entry` is a deepest folder,
                // add it to the result and `previous_entry` back to its parent.
                // The last one is always a deepest folder.
                Some(entries[i].path().to_path_buf())
            } else {
                None
            }
        })
        .collect_vec()
}

pub fn build_music_folders<P: AsRef<Path> + Sync>(
    top_paths: &[P],
    depth_levels: &[usize],
) -> Vec<PathBuf> {
    let canonicalized_top_paths: Vec<PathBuf> = top_paths
        .iter()
        .map(|path| {
            if !metadata(path)?.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    concat_string!(
                        path.as_ref().to_str().expect("non utf-8 path encountered"),
                        " is not a directory"
                    ),
                ))
            } else {
                canonicalize(path)
            }
        })
        .try_collect()
        .expect("top path is not a directory or it can not be canonicalized");

    for i in 0..canonicalized_top_paths.len() - 1 {
        for j in i + 1..canonicalized_top_paths.len() {
            if canonicalized_top_paths[i].starts_with(&canonicalized_top_paths[j])
                || canonicalized_top_paths[j].starts_with(&canonicalized_top_paths[i])
            {
                std::panic::panic_any(concat_string!(
                    &canonicalized_top_paths[i].to_str().expect("non utf-8 path encountered"),
                    " and ",
                    &canonicalized_top_paths[j].to_str().expect("non utf-8 path encountered"),
                    " contain each other"
                ))
            }
        }
    }

    if depth_levels.is_empty() {
        return canonicalized_top_paths;
    } else if depth_levels.len() != top_paths.len() {
        std::panic::panic_any("depth levels and top paths must have the same length")
    }

    canonicalized_top_paths
        .iter()
        .zip(depth_levels.iter().copied())
        .flat_map(|(root, depth)| get_deepest_folders(root, depth))
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;
    use std::str::FromStr;

    use super::*;
    use crate::utils::test::fs::TemporaryFs;

    #[test]
    fn test_top_paths_non_existent() {
        let result = catch_unwind(|| {
            build_music_folders(&[PathBuf::from_str("/non-existent").unwrap()], &[0])
        });

        assert!(
            result
                .as_ref()
                .err()
                .unwrap()
                .downcast_ref::<String>()
                .unwrap()
                .contains("top path is not a directory or it can not be canonicalized")
        );

        assert!(
            result.as_ref().err().unwrap().downcast_ref::<String>().unwrap().contains("NotFound")
        );
    }

    #[test]
    fn test_top_paths_is_file() {
        let temp_fs = TemporaryFs::default();
        let file = temp_fs.create_file("test.txt");

        let result = catch_unwind(|| build_music_folders(&[&file], &[0]));

        assert!(
            result.err().unwrap().downcast_ref::<String>().unwrap().contains("is not a directory")
        );
    }

    #[test]
    fn test_top_paths_nested() {
        let temp_fs = TemporaryFs::default();
        let parent = temp_fs.create_dir("test1/");
        let child = temp_fs.create_dir("test1/test2/");

        let result = catch_unwind(|| build_music_folders(&[&parent, &child], &[0, 0]));

        assert_eq!(
            *result.err().unwrap().downcast_ref::<String>().unwrap(),
            concat_string!(
                &parent.canonicalize().unwrap().to_str().expect("non utf-8 path encountered"),
                " and ",
                &child.canonicalize().unwrap().to_str().expect("non utf-8 path encountered"),
                " contain each other"
            )
        );
    }

    #[test]
    fn test_top_paths_depth_levels_empty() {
        let temp_fs = TemporaryFs::default();
        let dir_1 = temp_fs.create_dir("test1/");
        let dir_2 = temp_fs.create_dir("test2/");

        let inputs = vec![dir_1, dir_2];
        let results = build_music_folders(&inputs, &[]);

        assert_eq!(
            temp_fs.canonicalize_paths(&inputs.into_iter().sorted().collect_vec()),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[test]
    fn test_top_paths_depth_levels_neq_len() {
        let temp_fs = TemporaryFs::default();
        let dir_1 = temp_fs.create_dir("test1/");
        let dir_2 = temp_fs.create_dir("test2/");

        let result = catch_unwind(|| build_music_folders(&[dir_1, dir_2], &[0, 0, 0]));

        assert_eq!(
            *result.err().unwrap().downcast_ref::<&str>().unwrap(),
            "depth levels and top paths must have the same length"
        );
    }

    #[test]
    fn test_get_deepest_folder() {
        let temp_fs = TemporaryFs::default();
        temp_fs.create_dir("test1/test1.1/test1.1.1/");
        temp_fs.create_dir("test1/test1.1/test1.1.2/test1.1.2.1/test1.1.2.1.1/");
        temp_fs.create_dir("test1/test1.2/");
        temp_fs.create_dir("test1/test1.3/test1.3.1/test1.3.1.1/");
        temp_fs.create_dir("test2/");

        let inputs = temp_fs.join_paths(&[
            "test1/test1.1/test1.1.1/",
            "test1/test1.1/test1.1.2/test1.1.2.1/test1.1.2.1.1/",
            "test1/test1.2/",
            "test1/test1.3/test1.3.1/test1.3.1.1/",
            "test2/",
        ]);
        let results = get_deepest_folders(temp_fs.root_path(), usize::MAX);

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[test]
    fn test_get_deepest_folder_max_depth() {
        let temp_fs = TemporaryFs::default();
        temp_fs.create_dir("test1/test1.1/test1.1.1/");
        temp_fs.create_dir("test1/test1.1/test1.1.2/test1.1.2.1/test1.1.2.1.1/");
        temp_fs.create_dir("test1/test1.2/");
        temp_fs.create_dir("test1/test1.3/test1.3.1/test1.3.1.1/");
        temp_fs.create_dir("test2/");

        let inputs = temp_fs.join_paths(&[
            "test1/test1.1/test1.1.1/",
            "test1/test1.1/test1.1.2/",
            "test1/test1.2/",
            "test1/test1.3/test1.3.1/",
            "test2/",
        ]);
        let results = get_deepest_folders(temp_fs.root_path(), 3);

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[test]
    fn test_get_deepest_folder_file() {
        let temp_fs = TemporaryFs::default();
        temp_fs.create_file("test1/test1.1/test1.1.1/test1.1.1.1.txt");
        temp_fs.create_dir("test1/test1.1/test1.1.2/test1.1.2.1/test1.1.2.1.1/");
        temp_fs.create_dir("test1/test1.2/");
        temp_fs.create_dir("test1/test1.3/test1.3.1/test1.3.1.1/");
        temp_fs.create_file("test2/test2.1.txt");

        let inputs = temp_fs.join_paths(&[
            "test1/test1.1/test1.1.1/",
            "test1/test1.1/test1.1.2/",
            "test1/test1.2/",
            "test1/test1.3/test1.3.1/",
            "test2/",
        ]);
        let results = get_deepest_folders(temp_fs.root_path(), 3);

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[test]
    fn test_build_music_folders() {
        let temp_fs = TemporaryFs::default();
        temp_fs.create_file("test1/test1.1/test1.1.1/test1.1.1.1.txt");
        temp_fs.create_dir("test1/test1.1/test1.1.2/test1.1.2.1/test1.1.2.1.1/");
        temp_fs.create_dir("test1/test1.2/");
        temp_fs.create_dir("test1/test1.3/test1.3.1/test1.3.1.1/");
        temp_fs.create_file("test2/test2.1.txt");

        let inputs = temp_fs.canonicalize_paths(&temp_fs.join_paths(&[
            "test1/test1.1/",
            "test1/test1.2/",
            "test1/test1.3/",
            "test2/",
        ]));
        let results = build_music_folders(&temp_fs.join_paths(&["test1/", "test2/"]), &[1, 2]);

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }
}
