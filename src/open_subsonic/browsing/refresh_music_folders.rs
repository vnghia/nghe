use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures::stream::{self, StreamExt};
use futures::TryStreamExt;
use tracing::instrument;

use crate::config::FolderConfig;
use crate::models::*;
use crate::utils::fs::folders::build_music_folders;
use crate::DatabasePool;

#[instrument(skip(pool))]
pub async fn refresh_music_folders(
    pool: &DatabasePool,
    folder_config: &FolderConfig,
) -> (Vec<music_folders::MusicFolder>, usize) {
    let scan_start_time = time::OffsetDateTime::now_utc();

    let upserted_folders = stream::iter(build_music_folders(folder_config))
        .then(|(folder_path, folder_name)| async move {
            let folder_path = folder_path.to_str().expect("non-utf8 path encountered");
            if let Some(music_folder) = music_folders::table
                .filter(music_folders::path.eq(&folder_path))
                .or_filter(music_folders::name.eq(&folder_name))
                .select(music_folders::MusicFolder::as_select())
                .get_result(&mut pool.get().await.map_err(anyhow::Error::from)?)
                .await
                .optional()
                .map_err(anyhow::Error::from)?
            {
                if music_folder.path != folder_path {
                    diesel::update(music_folders::table)
                        .filter(music_folders::name.eq(&folder_name))
                        .set(music_folders::path.eq(&folder_path))
                        .returning(music_folders::MusicFolder::as_select())
                        .get_result(&mut pool.get().await.map_err(anyhow::Error::from)?)
                        .await
                } else if music_folder.name != folder_name {
                    diesel::update(music_folders::table)
                        .filter(music_folders::path.eq(&folder_path))
                        .set(music_folders::name.eq(&folder_name))
                        .returning(music_folders::MusicFolder::as_select())
                        .get_result(&mut pool.get().await.map_err(anyhow::Error::from)?)
                        .await
                } else {
                    diesel::update(music_folders::table)
                        .filter(music_folders::id.eq(music_folder.id))
                        .set(music_folders::scanned_at.eq(scan_start_time))
                        .execute(&mut pool.get().await.map_err(anyhow::Error::from)?)
                        .await
                        .map_err(anyhow::Error::from)?;
                    Ok(music_folder)
                }
            } else {
                diesel::insert_into(music_folders::table)
                    .values(music_folders::NewMusicFolder {
                        path: folder_path.into(),
                        name: folder_name,
                    })
                    .returning(music_folders::MusicFolder::as_select())
                    .get_result(&mut pool.get().await?)
                    .await
            }
            .map_err(anyhow::Error::from)
        })
        .try_collect::<Vec<_>>()
        .await
        .expect("can not delete old music folder");

    let deleted_folders = diesel::delete(music_folders::table)
        .filter(music_folders::scanned_at.lt(scan_start_time))
        .filter(music_folders::updated_at.lt(scan_start_time))
        .returning(music_folders::MusicFolder::as_returning())
        .get_results(&mut pool.get().await.expect("can not checkout a connection"))
        .await
        .expect("can not upsert music folder");

    for upserted_folder in &upserted_folders {
        tracing::debug!(?upserted_folder);
    }
    for deleted_folder in &deleted_folders {
        tracing::debug!(?deleted_folder);
    }

    (upserted_folders, deleted_folders.len())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use itertools::Itertools;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_refresh_insert() {
        let infra = Infra::new().await;
        let dir_1 = infra.fs.create_dir("test1/");
        let dir_2 = infra.fs.create_dir("test2/");

        let config =
            FolderConfig { top_paths: vec![dir_1, dir_2], top_names: vec![], depth_levels: vec![] };
        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &config).await;
        assert_eq!(
            vec!["test1", "test2"],
            upserted_folders.iter().map(|f| f.name.as_str()).collect_vec()
        );
        assert_eq!(
            infra.fs.canonicalize_paths(&config.top_paths),
            upserted_folders.iter().map(|f| PathBuf::from(&f.path)).sorted().collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_update_path() {
        let infra = Infra::new().await;
        let folder_name = "name";

        let dir_1 = infra.fs.create_dir("test1/");
        let config = FolderConfig {
            top_paths: vec![dir_1],
            top_names: vec![folder_name.into()],
            depth_levels: vec![],
        };
        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &config).await;
        assert_eq!(
            vec![folder_name],
            upserted_folders.iter().map(|f| f.name.as_str()).collect_vec()
        );
        assert_eq!(
            infra.fs.canonicalize_paths(&config.top_paths),
            upserted_folders.iter().map(|f| PathBuf::from(&f.path)).collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);

        let dir_2 = infra.fs.create_dir("test2/");
        let config = FolderConfig {
            top_paths: vec![dir_2],
            top_names: vec![folder_name.into()],
            depth_levels: vec![],
        };
        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &config).await;
        assert_eq!(
            vec![folder_name],
            upserted_folders.iter().map(|f| f.name.as_str()).collect_vec()
        );
        assert_eq!(
            infra.fs.canonicalize_paths(&config.top_paths),
            upserted_folders.iter().map(|f| PathBuf::from(&f.path)).collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_update_name() {
        let infra = Infra::new().await;
        let dir = infra.fs.create_dir("test/");

        let folder_name_1 = "name1";
        let config = FolderConfig {
            top_paths: vec![dir.clone()],
            top_names: vec![folder_name_1.into()],
            depth_levels: vec![],
        };
        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &config).await;
        assert_eq!(
            vec![folder_name_1],
            upserted_folders.iter().map(|f| f.name.as_str()).collect_vec()
        );
        assert_eq!(
            infra.fs.canonicalize_paths(&config.top_paths),
            upserted_folders.iter().map(|f| PathBuf::from(&f.path)).collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);

        let folder_name_2 = "name2";
        let config = FolderConfig {
            top_paths: vec![dir.clone()],
            top_names: vec![folder_name_2.into()],
            depth_levels: vec![],
        };
        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &config).await;
        assert_eq!(
            vec![folder_name_2],
            upserted_folders.iter().map(|f| f.name.as_str()).collect_vec()
        );
        assert_eq!(
            infra.fs.canonicalize_paths(&config.top_paths),
            upserted_folders.iter().map(|f| PathBuf::from(&f.path)).collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_upsert() {
        let infra = Infra::new().await;
        let dir_1 = infra.fs.create_dir("test1/");
        let dir_2 = infra.fs.create_dir("test2/");

        let config = FolderConfig {
            top_paths: vec![dir_1.clone()],
            top_names: vec![],
            depth_levels: vec![],
        };
        refresh_music_folders(infra.pool(), &config).await;

        let config =
            FolderConfig { top_paths: vec![dir_1, dir_2], top_names: vec![], depth_levels: vec![] };
        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &config).await;
        assert_eq!(
            vec!["test1", "test2"],
            upserted_folders.iter().map(|f| f.name.as_str()).collect_vec()
        );
        assert_eq!(
            infra.fs.canonicalize_paths(&config.top_paths),
            upserted_folders.iter().map(|f| PathBuf::from(&f.path)).sorted().collect_vec()
        );
        assert_eq!(deleted_folder_count, 0);
    }

    #[tokio::test]
    async fn test_refresh_delete() {
        let infra = Infra::new().await;
        let dir_1 = infra.fs.create_dir("test1/");
        let dir_2 = infra.fs.create_dir("test2/");
        let dir_3 = infra.fs.create_dir("test3/");

        let config = FolderConfig {
            top_paths: vec![dir_1.clone(), dir_3],
            top_names: vec![],
            depth_levels: vec![],
        };
        refresh_music_folders(infra.pool(), &config).await;

        let config =
            FolderConfig { top_paths: vec![dir_1, dir_2], top_names: vec![], depth_levels: vec![] };
        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(infra.pool(), &config).await;
        assert_eq!(
            infra.fs.canonicalize_paths(&config.top_paths),
            upserted_folders.iter().map(|f| PathBuf::from(&f.path)).sorted().collect_vec()
        );
        assert_eq!(deleted_folder_count, 1);
    }
}
