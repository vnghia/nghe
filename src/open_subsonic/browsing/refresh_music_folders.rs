use crate::entity::{prelude::*, *};
use crate::utils::fs::folders::build_music_folders;

use itertools::Itertools;
use sea_orm::{DatabaseConnection, EntityTrait, *};
use std::path::Path;

pub async fn refresh_music_folders<P: AsRef<Path>>(
    conn: &DatabaseConnection,
    top_paths: &[P],
    depth_levels: &[u8],
) -> (Vec<music_folder::Model>, u64) {
    let update_start_time = time::OffsetDateTime::now_utc();

    let music_folder_models = build_music_folders(top_paths, depth_levels)
        .await
        .iter()
        .map(|music_folder| music_folder::ActiveModel {
            path: Set(music_folder.to_string_lossy().to_string()),
            updated_at: Set(time::OffsetDateTime::now_utc()),
            ..Default::default()
        })
        .collect_vec();
    let music_folder_len = music_folder_models.len();

    // TODO: use `exec_with_retuning` with `insert_many`.
    // https://github.com/SeaQL/sea-orm/issues/1862
    MusicFolder::insert_many(music_folder_models)
        .on_conflict(
            sea_query::OnConflict::column(music_folder::Column::Path)
                .update_column(music_folder::Column::UpdatedAt)
                .to_owned(),
        )
        .exec(conn)
        .await
        .expect("can not upsert music folder");

    // TODO: return more information about what are deleted.
    // https://github.com/SeaQL/sea-orm/discussions/2059
    let deleted_folder_count = MusicFolder::delete_many()
        .filter(music_folder::Column::UpdatedAt.lt(update_start_time))
        .exec(conn)
        .await
        .expect("can not delete old music folder")
        .rows_affected;
    tracing::info!("{} old music folders deleted", deleted_folder_count);

    let upserted_folders = MusicFolder::find()
        .all(conn)
        .await
        .expect("can not get list of upserted folders");
    for upserted_folder in &upserted_folders {
        tracing::info!("new music folder added: {}", &upserted_folder.path);
    }
    assert_eq!(upserted_folders.len(), music_folder_len);

    (upserted_folders, deleted_folder_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::{db::TemporaryDatabase, fs::TemporaryFs};

    use std::path::PathBuf;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_refresh_insert() {
        let db = TemporaryDatabase::new_from_env().await;

        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/").await;
        let dir_2 = temp_fs.create_dir("test2/").await;

        let mut inputs = vec![dir_1, dir_2];

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(db.get_conn(), &inputs, &[]).await;
        let mut results = upserted_folders
            .iter()
            .map(|model| PathBuf::from(&model.path))
            .collect_vec();

        inputs.sort();
        let inputs = temp_fs.canonicalize_paths(&inputs);
        results.sort();

        assert_eq!(inputs, results);
        assert_eq!(deleted_folder_count, 0);

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_refresh_upsert() {
        let db = TemporaryDatabase::new_from_env().await;

        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/").await;
        let dir_2 = temp_fs.create_dir("test2/").await;

        db.insert(
            music_folder::Model {
                id: Uuid::new_v4(),
                path: tokio::fs::canonicalize(&dir_1)
                    .await
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                updated_at: time::OffsetDateTime::now_utc(),
            }
            .into_active_model(),
        )
        .await;

        let mut inputs = vec![dir_1, dir_2];

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(db.get_conn(), &inputs, &[]).await;
        let mut results = upserted_folders
            .iter()
            .map(|model| PathBuf::from(&model.path))
            .collect_vec();

        inputs.sort();
        let inputs = temp_fs.canonicalize_paths(&inputs);
        results.sort();

        assert_eq!(inputs, results);
        assert_eq!(deleted_folder_count, 0);

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_refresh_delete() {
        let db = TemporaryDatabase::new_from_env().await;

        let temp_fs = TemporaryFs::new();
        let dir_1 = temp_fs.create_dir("test1/").await;
        let dir_2 = temp_fs.create_dir("test2/").await;
        let dir_3 = temp_fs.create_dir("test3/").await;

        db.insert(
            music_folder::Model {
                id: Uuid::new_v4(),
                path: tokio::fs::canonicalize(&dir_1)
                    .await
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                updated_at: time::OffsetDateTime::now_utc(),
            }
            .into_active_model(),
        )
        .await
        .insert(
            music_folder::Model {
                id: Uuid::new_v4(),
                path: tokio::fs::canonicalize(&dir_3)
                    .await
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                updated_at: time::OffsetDateTime::now_utc(),
            }
            .into_active_model(),
        )
        .await;

        let mut inputs = vec![dir_1, dir_2];

        let (upserted_folders, deleted_folder_count) =
            refresh_music_folders(db.get_conn(), &inputs, &[]).await;
        let mut results = upserted_folders
            .iter()
            .map(|model| PathBuf::from(&model.path))
            .collect_vec();

        inputs.sort();
        let inputs = temp_fs.canonicalize_paths(&inputs);
        results.sort();

        assert_eq!(inputs, results);
        assert_eq!(deleted_folder_count, 1);

        db.async_drop().await;
    }
}
