use crate::models::*;
use crate::utils::fs::folders::build_music_folders;
use crate::DatabasePool;

use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use std::path::Path;

pub async fn refresh_music_folders<P: AsRef<Path>>(
    pool: &DatabasePool,
    top_paths: &[P],
    depth_levels: &[u8],
) -> (Vec<music_folders::MusicFolder>, usize) {
    let scan_start_time = time::OffsetDateTime::now_utc();

    let upserted_folders = diesel::insert_into(music_folders::table)
        .values(
            build_music_folders(top_paths, depth_levels)
                .await
                .iter()
                .map(|path| music_folders::NewMusicFolder {
                    path: path.to_string_lossy(),
                })
                .collect_vec(),
        )
        .on_conflict(music_folders::path)
        .do_update()
        .set(music_folders::scanned_at.eq(scan_start_time))
        .returning(music_folders::MusicFolder::as_returning())
        .get_results(&mut pool.get().await.expect("can not checkout a connection"))
        .await
        .expect("can not upsert music folder");

    let deleted_folders = diesel::delete(music_folders::table)
        .filter(music_folders::scanned_at.lt(scan_start_time))
        .returning(music_folders::MusicFolder::as_returning())
        .get_results(&mut pool.get().await.expect("can not checkout a connection"))
        .await
        .expect("can not delete old music folder");

    for upserted_folder in &upserted_folders {
        tracing::info!("new music folder added: {}", &upserted_folder.path);
    }
    for deleted_folder in &deleted_folders {
        tracing::info!("old music folder deleted: {}", &deleted_folder.path);
    }

    (upserted_folders, deleted_folders.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::{db::TemporaryDatabase, fs::TemporaryFs};

    use std::path::PathBuf;

    #[tokio::test]
    async fn test_refresh_insert() {
        let db = TemporaryDatabase::new_from_env().await;

        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/").await;
        let dir_2 = temp_fs.create_dir("test2/").await;

        let inputs = vec![dir_1, dir_2];

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(db.get_pool(), &inputs, &[]).await;
        let results = upserted_folders
            .iter()
            .map(|model| PathBuf::from(&model.path))
            .collect_vec();

        assert_eq!(
            temp_fs.canonicalize_paths(&inputs.into_iter().sorted().collect_vec()),
            results.into_iter().sorted().collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_upsert() {
        let db = TemporaryDatabase::new_from_env().await;

        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/").await;
        let dir_2 = temp_fs.create_dir("test2/").await;

        diesel::insert_into(music_folders::table)
            .values(
                temp_fs
                    .canonicalize_paths(&[dir_1.clone()])
                    .iter()
                    .map(|path| music_folders::NewMusicFolder {
                        path: path.to_string_lossy(),
                    })
                    .collect_vec(),
            )
            .execute(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        let inputs = vec![dir_1, dir_2];

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(db.get_pool(), &inputs, &[]).await;
        let results = upserted_folders
            .iter()
            .map(|model| PathBuf::from(&model.path))
            .collect_vec();

        assert_eq!(
            temp_fs.canonicalize_paths(&inputs.into_iter().sorted().collect_vec()),
            results.into_iter().sorted().collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_delete() {
        let db = TemporaryDatabase::new_from_env().await;

        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/").await;
        let dir_2 = temp_fs.create_dir("test2/").await;
        let dir_3 = temp_fs.create_dir("test3/").await;

        diesel::insert_into(music_folders::table)
            .values(
                temp_fs
                    .canonicalize_paths(&[dir_1.clone(), dir_3])
                    .iter()
                    .map(|path| music_folders::NewMusicFolder {
                        path: path.to_string_lossy(),
                    })
                    .collect_vec(),
            )
            .execute(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        let inputs = vec![dir_1, dir_2];

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(db.get_pool(), &inputs, &[]).await;
        let results = upserted_folders
            .iter()
            .map(|model| PathBuf::from(&model.path))
            .collect_vec();

        assert_eq!(
            temp_fs.canonicalize_paths(&inputs.into_iter().sorted().collect_vec()),
            results.into_iter().sorted().collect_vec()
        );
        assert_eq!(deleted_folder_count, 1);
    }
}
