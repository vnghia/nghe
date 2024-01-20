use crate::models::*;
use crate::{DbPool, OSResult};

use diesel::query_dsl::methods::SelectDsl;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

pub async fn refresh_user_music_folders(
    pool: &DbPool,
    user_ids: &[Uuid],
    music_folder_ids: &[Uuid],
) -> OSResult<()> {
    let new_user_music_folder_permissions = user_ids
        .iter()
        .copied()
        .cartesian_product(music_folder_ids.iter().copied())
        .map(|(user_id, music_folder_id)| {
            user_music_folder_permissions::NewUserMusicFolderPermission {
                user_id,
                music_folder_id,
                allow: true,
            }
        })
        .collect_vec();

    diesel::insert_into(user_music_folder_permissions::table)
        .values(new_user_music_folder_permissions)
        .on_conflict_do_nothing()
        .execute(&mut pool.get().await?)
        .await?;

    tracing::info!("done refreshing user music folders");
    Ok(())
}

pub async fn refresh_user_music_folders_all_users(
    pool: &DbPool,
    music_folder_ids: &[Uuid],
) -> OSResult<()> {
    let user_ids = users::table
        .select(users::id)
        .load::<Uuid>(&mut pool.get().await?)
        .await?;
    refresh_user_music_folders(pool, &user_ids, music_folder_ids).await
}

pub async fn refresh_user_music_folders_all_folders(
    pool: &DbPool,
    user_ids: &[Uuid],
) -> OSResult<()> {
    let music_folder_ids = music_folders::table
        .select(music_folders::id)
        .load::<Uuid>(&mut pool.get().await?)
        .await?;
    refresh_user_music_folders(pool, user_ids, &music_folder_ids).await
}

#[cfg(test)]
mod tests {
    use super::super::test::setup_user_and_music_folders;
    use super::*;

    #[tokio::test]
    async fn test_refresh_insert() {
        let (db, _, _, _temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(2, 2, &[true, true, true, true]).await;

        refresh_user_music_folders_all_users(
            db.get_pool(),
            &music_folders
                .iter()
                .map(|music_folder| music_folder.id)
                .collect_vec(),
        )
        .await
        .unwrap();

        let results = user_music_folder_permissions::table
            .load(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        assert_eq!(
            permissions.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_refresh_do_nothing() {
        let (db, _, _, _temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(2, 2, &[true, false, true, true]).await;

        diesel::insert_into(user_music_folder_permissions::table)
            .values(&[permissions[1].clone(), permissions[3].clone()])
            .execute(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        refresh_user_music_folders_all_users(
            db.get_pool(),
            &music_folders
                .iter()
                .map(|music_folder| music_folder.id)
                .collect_vec(),
        )
        .await
        .unwrap();

        let results = user_music_folder_permissions::table
            .load(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        assert_eq!(
            permissions.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }
}
