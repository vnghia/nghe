use crate::models::*;
use crate::{DatabasePool, OSResult};

use diesel::query_dsl::methods::SelectDsl;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use std::borrow::Cow;
use uuid::Uuid;

pub async fn refresh_permissions(
    pool: &DatabasePool,
    user_ids: Option<&[Uuid]>,
    music_folder_ids: Option<&[Uuid]>,
) -> OSResult<()> {
    let user_ids: Cow<[Uuid]> = match user_ids {
        Some(user_ids) => Cow::Borrowed(user_ids),
        None => users::table
            .select(users::id)
            .load::<Uuid>(&mut pool.get().await?)
            .await?
            .into(),
    };

    let music_folder_ids: Cow<[Uuid]> = match music_folder_ids {
        Some(music_folder_ids) => Cow::Borrowed(music_folder_ids),
        None => music_folders::table
            .select(music_folders::id)
            .load::<Uuid>(&mut pool.get().await?)
            .await?
            .into(),
    };

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

#[cfg(test)]
mod tests {
    use super::super::test::setup_user_and_music_folders;
    use super::*;

    #[tokio::test]
    async fn test_refresh_insert() {
        let (db, _, _, _temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(2, 2, &[true, true, true, true]).await;

        refresh_permissions(
            db.get_pool(),
            None,
            Some(
                &music_folders
                    .iter()
                    .map(|music_folder| music_folder.id)
                    .collect_vec(),
            ),
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

        refresh_permissions(
            db.get_pool(),
            None,
            Some(
                &music_folders
                    .iter()
                    .map(|music_folder| music_folder.id)
                    .collect_vec(),
            ),
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
