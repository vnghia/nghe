use concat_string::concat_string;
use futures::stream::{self, StreamExt, TryStreamExt};
use itertools::Itertools;
use std::path::{Path, PathBuf};
use tokio::fs::*;
use walkdir::WalkDir;

fn get_deepest_folders<P: AsRef<Path>>(root: P, max_depth: u8) -> Vec<PathBuf> {
    let entries = WalkDir::new(&root)
        .max_depth(max_depth.into())
        .into_iter()
        .filter_entry(|entry| {
            entry
                .metadata()
                .expect(&concat_string!(
                    "can not read metadata of ",
                    entry.path().to_string_lossy()
                ))
                .is_dir()
        })
        .collect::<Result<Vec<_>, _>>()
        .expect(&concat_string!(
            "can not traverse ",
            root.as_ref().to_string_lossy()
        ));

    let folders = (0..entries.len())
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
        .collect_vec();
    folders
}

pub async fn build_music_folders<P: AsRef<Path>>(
    top_paths: &[P],
    depth_levels: &[u8],
) -> Vec<PathBuf> {
    let canonicalized_top_paths = stream::iter(top_paths)
        .then(|path| async move {
            if !metadata(path).await?.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    concat_string!(path.as_ref().to_string_lossy(), " is not a directory"),
                ))
            } else {
                canonicalize(&path).await
            }
        })
        .try_collect::<Vec<_>>()
        .await
        .expect("top path is not a directory or it can not be canonicalized");

    for i in 0..canonicalized_top_paths.len() - 1 {
        for j in i + 1..canonicalized_top_paths.len() {
            if canonicalized_top_paths[i].starts_with(&canonicalized_top_paths[j])
                || canonicalized_top_paths[j].starts_with(&canonicalized_top_paths[i])
            {
                std::panic::panic_any(concat_string!(
                    &canonicalized_top_paths[i].to_string_lossy(),
                    " and ",
                    &canonicalized_top_paths[j].to_string_lossy(),
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

    let depth_levels = depth_levels.to_owned();
    tokio::task::spawn_blocking(move || {
        canonicalized_top_paths
            .iter()
            .zip(depth_levels.iter())
            .flat_map(|(root, depth)| get_deepest_folders(root, *depth))
            .collect_vec()
    })
    .await
    .expect("can not get deepest folders from top paths")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::fs::TemporaryFs;

    use futures::FutureExt;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_top_paths_non_existent() {
        let result = build_music_folders(&[PathBuf::from_str("/non-existent").unwrap()], &[0])
            .catch_unwind()
            .await;

        assert!(result
            .as_ref()
            .err()
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .contains("top path is not a directory or it can not be canonicalized"));

        assert!(result
            .as_ref()
            .err()
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .contains("NotFound"));
    }

    #[tokio::test]
    async fn test_top_paths_is_file() {
        let temp_fs = TemporaryFs::new();
        let file = temp_fs.create_file("test.txt");

        let result = build_music_folders(&[file.clone()], &[0])
            .catch_unwind()
            .await;

        assert!(result
            .err()
            .unwrap()
            .downcast_ref::<String>()
            .unwrap()
            .contains("is not a directory"));
    }

    #[tokio::test]
    async fn test_top_paths_nested() {
        let temp_fs = TemporaryFs::new();
        let parent = temp_fs.create_dir("test1/");
        let child = temp_fs.create_dir("test1/test2/");

        let result = build_music_folders(&[parent.clone(), child.clone()], &[0, 0])
            .catch_unwind()
            .await;

        assert_eq!(
            *result.err().unwrap().downcast_ref::<String>().unwrap(),
            concat_string!(
                &parent.canonicalize().unwrap().to_string_lossy(),
                " and ",
                &child.canonicalize().unwrap().to_string_lossy(),
                " contain each other"
            )
        );
    }

    #[tokio::test]
    async fn test_top_paths_depth_levels_empty() {
        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/");
        let dir_2 = temp_fs.create_dir("test2/");

        let inputs = vec![dir_1, dir_2];
        let results = build_music_folders(&inputs, &[]).await;

        assert_eq!(
            temp_fs.canonicalize_paths(&inputs.into_iter().sorted().collect_vec()),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_top_paths_depth_levels_neq_len() {
        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/");
        let dir_2 = temp_fs.create_dir("test2/");

        let result = build_music_folders(&[dir_1, dir_2], &[0, 0, 0])
            .catch_unwind()
            .await;

        assert_eq!(
            *result.err().unwrap().downcast_ref::<&str>().unwrap(),
            "depth levels and top paths must have the same length"
        );
    }

    #[tokio::test]
    async fn test_get_deepest_folder() {
        let temp_fs = TemporaryFs::new();
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
        let results = get_deepest_folders(temp_fs.get_root_path(), u8::MAX);

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_get_deepest_folder_max_depth() {
        let temp_fs = TemporaryFs::new();
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
        let results = get_deepest_folders(temp_fs.get_root_path(), 3);

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_get_deepest_folder_file() {
        let temp_fs = TemporaryFs::new();
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
        let results = get_deepest_folders(temp_fs.get_root_path(), 3);

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_build_music_folders() {
        let temp_fs = TemporaryFs::new();
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
        let results =
            build_music_folders(&temp_fs.join_paths(&["test1/", "test2/"]), &[1, 2]).await;

        assert_eq!(
            inputs.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }
}
