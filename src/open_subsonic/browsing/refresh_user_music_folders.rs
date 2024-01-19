use crate::entity::{prelude::*, *};
use crate::{OSResult, OpenSubsonicError};

use itertools::Itertools;
use sea_orm::{DatabaseConnection, EntityTrait, *};
use uuid::Uuid;

pub async fn refresh_user_music_folders(
    conn: &DatabaseConnection,
    user_ids: &[Uuid],
    music_folder_ids: &[Uuid],
) -> OSResult<()> {
    let user_music_folder_models = user_ids
        .iter()
        .copied()
        .cartesian_product(music_folder_ids.iter().copied())
        .map(
            |(user_id, music_folder_id)| user_music_folder::ActiveModel {
                user_id: Set(user_id),
                music_folder_id: Set(music_folder_id),
                ..Default::default()
            },
        );

    UserMusicFolder::insert_many(user_music_folder_models)
        .on_conflict(
            sea_query::OnConflict::columns([
                user_music_folder::Column::UserId,
                user_music_folder::Column::MusicFolderId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .on_empty_do_nothing()
        .exec(conn)
        .await
        .expect("can not set permission for in user music folder");

    tracing::info!("done refreshing user music folders");
    Ok(())
}

pub async fn refresh_user_music_folders_all_users(
    conn: &DatabaseConnection,
    music_folder_ids: &[Uuid],
) -> OSResult<()> {
    let user_ids = User::find()
        .select_column(user::Column::Id)
        .all(conn)
        .await
        .map_err(|e| OpenSubsonicError::Generic { source: e.into() })?
        .iter()
        .map(|user| user.id)
        .collect_vec();
    refresh_user_music_folders(conn, &user_ids, music_folder_ids).await
}

pub async fn refresh_user_music_folders_all_folders(
    conn: &DatabaseConnection,
    user_ids: &[Uuid],
) -> OSResult<()> {
    let music_folder_ids = MusicFolder::find()
        .select_column(music_folder::Column::Id)
        .all(conn)
        .await
        .map_err(|e| OpenSubsonicError::Generic { source: e.into() })?
        .iter()
        .map(|music_folder: &music_folder::Model| music_folder.id)
        .collect_vec();
    refresh_user_music_folders(conn, user_ids, &music_folder_ids).await
}

#[cfg(test)]
mod tests {
    use super::super::test::setup_user_and_music_folders;
    use super::*;

    fn sort_models(models: Vec<user_music_folder::Model>) -> Vec<user_music_folder::Model> {
        models
            .into_iter()
            .sorted_by_key(|model| model.user_id)
            .sorted_by_key(|model| model.music_folder_id)
            .collect_vec()
    }

    #[tokio::test]
    async fn test_refresh_insert() {
        let (db, _, _, _temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(2, 2, &[true, true, true, true]).await;
        refresh_user_music_folders_all_users(
            db.get_conn(),
            &music_folders
                .iter()
                .map(|music_folder| music_folder.id)
                .collect_vec(),
        )
        .await
        .unwrap();

        let results = UserMusicFolder::find().all(db.get_conn()).await.unwrap();
        let permissions = sort_models(permissions);
        let results = sort_models(results);
        assert_eq!(permissions, results);

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_refresh_do_nothing() {
        let (db, _, _, _temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(2, 2, &[true, false, true, true]).await;
        db.insert(permissions[1].clone().into_active_model()).await;
        db.insert(permissions[3].clone().into_active_model()).await;
        refresh_user_music_folders_all_users(
            db.get_conn(),
            &music_folders
                .iter()
                .map(|music_folder| music_folder.id)
                .collect_vec(),
        )
        .await
        .unwrap();

        let results = UserMusicFolder::find().all(db.get_conn()).await.unwrap();
        let permissions = sort_models(permissions);
        let results = sort_models(results);
        assert_eq!(permissions, results);

        db.async_drop().await;
    }
}
