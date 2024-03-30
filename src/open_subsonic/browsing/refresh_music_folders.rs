use std::path::Path;

use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use tracing::instrument;

use crate::models::*;
use crate::utils::fs::folders::build_music_folders;
use crate::DatabasePool;

#[instrument(skip(pool))]
pub async fn refresh_music_folders<P: AsRef<Path> + Sync + std::fmt::Debug>(
    pool: &DatabasePool,
    top_paths: &[P],
    depth_levels: &[usize],
) -> (Vec<music_folders::MusicFolder>, usize) {
    let scan_start_time = time::OffsetDateTime::now_utc();

    let upserted_folders = diesel::insert_into(music_folders::table)
        .values(
            build_music_folders(top_paths, depth_levels)
                .iter()
                .map(|path| music_folders::NewMusicFolder {
                    path: path.to_str().expect("non utf-8 path encountered").into(),
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
        tracing::info!(
            upserted_folder.id = %upserted_folder.id,
            upserted_folder.path = upserted_folder.path
        );
    }
    for deleted_folder in &deleted_folders {
        tracing::info!(
            deleted_folder.id = %deleted_folder.id,
            deleted_folder.path = deleted_folder.path
        );
    }

    (upserted_folders, deleted_folders.len())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_refresh_insert() {
        let infra = Infra::new().await;
        let dir_1 = infra.fs.create_dir("test1/");
        let dir_2 = infra.fs.create_dir("test2/");
        let inputs = vec![dir_1, dir_2].into_iter().sorted().collect_vec();

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &inputs, &[]).await;
        let upserted_folders =
            upserted_folders.iter().map(|model| PathBuf::from(&model.path)).sorted().collect_vec();

        assert_eq!(infra.fs.canonicalize_paths(&inputs), upserted_folders);
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_upsert() {
        let infra = Infra::new().await;
        let dir_1 = infra.fs.create_dir("test1/");
        let dir_2 = infra.fs.create_dir("test2/");

        diesel::insert_into(music_folders::table)
            .values(
                infra
                    .fs
                    .canonicalize_paths(&[&dir_1])
                    .iter()
                    .map(|path| music_folders::NewMusicFolder {
                        path: path.to_str().expect("non utf-8 path encountered").into(),
                    })
                    .collect_vec(),
            )
            .execute(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();

        let inputs = vec![dir_1, dir_2].into_iter().sorted().collect_vec();

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &inputs, &[]).await;
        let upserted_folders =
            upserted_folders.iter().map(|model| PathBuf::from(&model.path)).sorted().collect_vec();

        assert_eq!(infra.fs.canonicalize_paths(&inputs), upserted_folders);
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_delete() {
        let infra = Infra::new().await;
        let dir_1 = infra.fs.create_dir("test1/");
        let dir_2 = infra.fs.create_dir("test2/");
        let dir_3 = infra.fs.create_dir("test3/");

        diesel::insert_into(music_folders::table)
            .values(
                infra
                    .fs
                    .canonicalize_paths(&[&dir_1, &dir_3])
                    .iter()
                    .map(|path| music_folders::NewMusicFolder {
                        path: path.to_str().expect("non utf-8 path encountered").into(),
                    })
                    .collect_vec(),
            )
            .execute(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();

        let inputs = vec![dir_1, dir_2].into_iter().sorted().collect_vec();

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &inputs, &[]).await;
        let upserted_folders =
            upserted_folders.iter().map(|model| PathBuf::from(&model.path)).sorted().collect_vec();

        assert_eq!(infra.fs.canonicalize_paths(&inputs), upserted_folders);
        assert_eq!(deleted_folder_count, 1);
    }
}
