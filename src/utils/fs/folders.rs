use std::borrow::Cow;
use std::fs::*;
use std::path::{Path, PathBuf};

use concat_string::concat_string;
use itertools::Itertools;
use walkdir::WalkDir;

use crate::config::FolderConfig;

fn get_deepest_folders<P: AsRef<Path> + Sync>(top_path: P, max_depth: usize) -> Vec<PathBuf> {
    let entries = WalkDir::new(&top_path)
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
            top_path.as_ref().to_str().expect("non utf-8 path encountered")
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

pub fn build_music_folders(folder_config: &FolderConfig) -> Vec<(PathBuf, Cow<str>)> {
    let FolderConfig { top_paths, top_names, depth_levels } = folder_config;
    let top_paths: Vec<PathBuf> = top_paths
        .iter()
        .map(|path| {
            if !metadata(path)?.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    concat_string!(
                        path.to_str().expect("non utf-8 path encountered"),
                        " is not a directory"
                    ),
                ))
            } else {
                canonicalize(path)
            }
        })
        .try_collect()
        .expect("top path is not a directory or it can not be canonicalized");

    for i in 0..top_paths.len() - 1 {
        for j in i + 1..top_paths.len() {
            if top_paths[i].starts_with(&top_paths[j]) || top_paths[j].starts_with(&top_paths[i]) {
                panic!(
                    "{} and {} contain each other",
                    &top_paths[i].to_str().expect("non utf-8 path encountered"),
                    &top_paths[j].to_str().expect("non utf-8 path encountered")
                )
            }
        }
    }

    let top_names: Vec<Cow<str>> = if top_names.is_empty() {
        top_paths
            .iter()
            .map(|p| {
                p.file_name()
                    .expect("file name should not be empty")
                    .to_str()
                    .expect("non utf-8 path encountered")
                    .to_string()
                    .into()
            })
            .collect_vec()
    } else if top_paths.len() == top_names.len() {
        top_names.iter().map(|n| n.into()).collect_vec()
    } else {
        panic!("top paths and top names must have the same length")
    };

    let depth_levels = if depth_levels.is_empty() {
        vec![0; top_paths.len()]
    } else if top_paths.len() == depth_levels.len() {
        depth_levels.to_vec()
    } else {
        panic!("top paths and depth levels must have the same length")
    };

    top_paths
        .into_iter()
        .zip(top_names)
        .zip(depth_levels.iter().copied())
        .flat_map(|((top_path, top_name), max_depth)| {
            if max_depth == 0 {
                vec![(top_path, top_name)]
            } else {
                let folders = get_deepest_folders(&top_path, max_depth);
                if folders.len() == 1 {
                    vec![(top_path, top_name)]
                } else {
                    folders
                        .into_iter()
                        .map(|f| {
                            let name = concat_string!(
                                top_name,
                                "-",
                                f.file_name()
                                    .expect("file name should not be empty")
                                    .to_str()
                                    .expect("non utf-8 path encountered")
                            );
                            (f, name.into())
                        })
                        .collect_vec()
                }
            }
        })
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use super::*;
    use crate::utils::test::fs::TemporaryFs;

    #[test]
    fn test_top_paths_non_existent() {
        let config = FolderConfig {
            top_paths: vec![PathBuf::from("/non-existent")],
            top_names: vec![],
            depth_levels: vec![0],
        };
        let result = catch_unwind(|| build_music_folders(&config));

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
            result.as_ref().unwrap_err().downcast_ref::<String>().unwrap().contains("NotFound")
        );
    }

    #[test]
    fn test_top_paths_is_file() {
        let temp_fs = TemporaryFs::default();
        let file = temp_fs.create_file("test.txt");

        let config =
            FolderConfig { top_paths: vec![file], top_names: vec![], depth_levels: vec![0] };
        let result = catch_unwind(|| build_music_folders(&config));

        assert!(
            result.unwrap_err().downcast_ref::<String>().unwrap().contains("is not a directory")
        );
    }

    #[test]
    fn test_top_paths_nested() {
        let temp_fs = TemporaryFs::default();
        let parent = temp_fs.create_dir("test1/");
        let child = temp_fs.create_dir("test1/test2/");

        let config = FolderConfig {
            top_paths: vec![parent, child],
            top_names: vec![],
            depth_levels: vec![0],
        };
        let result = catch_unwind(|| build_music_folders(&config));

        assert_eq!(
            *result.unwrap_err().downcast_ref::<String>().unwrap(),
            concat_string!(
                &config.top_paths[0]
                    .canonicalize()
                    .unwrap()
                    .to_str()
                    .expect("non utf-8 path encountered"),
                " and ",
                &config.top_paths[1]
                    .canonicalize()
                    .unwrap()
                    .to_str()
                    .expect("non utf-8 path encountered"),
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
        let config = FolderConfig { top_paths: inputs, top_names: vec![], depth_levels: vec![] };
        let results = build_music_folders(&config);

        assert_eq!(
            temp_fs.canonicalize_paths(&config.top_paths.iter().sorted().collect_vec()),
            results.into_iter().map(|r| r.0).sorted().collect_vec()
        );
    }

    #[test]
    fn test_top_paths_depth_levels_neq_len() {
        let temp_fs = TemporaryFs::default();
        let dir_1 = temp_fs.create_dir("test1/");
        let dir_2 = temp_fs.create_dir("test2/");

        let config = FolderConfig {
            top_paths: vec![dir_1, dir_2],
            top_names: vec![],
            depth_levels: vec![0, 0, 0],
        };
        let result = catch_unwind(|| build_music_folders(&config));

        assert_eq!(
            *result.unwrap_err().downcast_ref::<&str>().unwrap(),
            "top paths and depth levels must have the same length"
        );
    }

    #[test]
    fn test_top_paths_top_names_neq_len() {
        let temp_fs = TemporaryFs::default();
        let dir_1 = temp_fs.create_dir("test1/");
        let dir_2 = temp_fs.create_dir("test2/");

        let config = FolderConfig {
            top_paths: vec![dir_1, dir_2],
            top_names: vec!["name".to_string()],
            depth_levels: vec![0, 0],
        };
        let result = catch_unwind(|| build_music_folders(&config));

        assert_eq!(
            *result.unwrap_err().downcast_ref::<&str>().unwrap(),
            "top paths and top names must have the same length"
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
        temp_fs.create_dir("test3/test3.1/test3.2/");

        let inputs = temp_fs.canonicalize_paths(&temp_fs.join_paths(&[
            "test1/test1.1/",
            "test1/test1.2/",
            "test1/test1.3/",
            "test2/",
            "test3/",
        ]));
        let config = FolderConfig {
            top_paths: temp_fs.join_paths(&["test1/", "test2/", "test3/"]),
            top_names: vec!["name1".into(), "name2".into(), "name3".into()],
            depth_levels: vec![1, 2, 0],
        };
        let results = build_music_folders(&config);

        assert_eq!(
            inputs.iter().sorted().collect_vec(),
            results.iter().map(|r| &r.0).sorted().collect_vec()
        );
        assert_eq!(
            ["name1-test1.1", "name1-test1.2", "name1-test1.3", "name2", "name3"]
                .into_iter()
                .sorted()
                .collect_vec(),
            results.iter().map(|r| r.1.as_ref()).sorted().collect_vec()
        );
    }

    #[test]
    fn test_build_music_folders_default_name() {
        let temp_fs = TemporaryFs::default();
        temp_fs.create_file("test1/test1.1/test1.1.1/test1.1.1.1.txt");
        temp_fs.create_dir("test1/test1.1/test1.1.2/test1.1.2.1/test1.1.2.1.1/");
        temp_fs.create_dir("test1/test1.2/");
        temp_fs.create_dir("test1/test1.3/test1.3.1/test1.3.1.1/");
        temp_fs.create_file("test2/test2.1.txt");
        temp_fs.create_dir("test3/test3.1/test3.2/");

        let config = FolderConfig {
            top_paths: temp_fs.join_paths(&["test1/", "test2/", "test3/"]),
            top_names: vec![],
            depth_levels: vec![1, 2, 0],
        };
        let results = build_music_folders(&config);

        assert_eq!(
            ["test1-test1.1", "test1-test1.2", "test1-test1.3", "test2", "test3"]
                .into_iter()
                .sorted()
                .collect_vec(),
            results.iter().map(|r| r.1.as_ref()).sorted().collect_vec()
        );
    }
}
