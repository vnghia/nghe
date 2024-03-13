use crate::models::*;
use crate::DatabasePool;

use anyhow::Result;
use diesel::query_dsl::methods::SelectDsl;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use std::borrow::Cow;
use uuid::Uuid;

pub async fn refresh_permissions(
    pool: &DatabasePool,
    user_ids: Option<&[Uuid]>,
    music_folder_ids: Option<&[Uuid]>,
) -> Result<()> {
    let user_ids: Cow<[Uuid]> = match user_ids {
        Some(user_ids) => user_ids.into(),
        None => users::table
            .select(users::id)
            .load::<Uuid>(&mut pool.get().await?)
            .await?
            .into(),
    };

    let music_folder_ids: Cow<[Uuid]> = match music_folder_ids {
        Some(music_folder_ids) => music_folder_ids.into(),
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
    use super::*;
    use crate::utils::test::setup::TestInfra;

    #[tokio::test]
    async fn test_refresh_insert() {
        let (test_infra, permissions) =
            TestInfra::setup_users_and_music_folders_no_refresh(2, 2, &[true, true, true, true])
                .await;

        refresh_permissions(
            test_infra.pool(),
            None,
            Some(&test_infra.music_folder_ids(..)),
        )
        .await
        .unwrap();

        let results = user_music_folder_permissions::table
            .load(&mut test_infra.pool().get().await.unwrap())
            .await
            .unwrap();

        assert_eq!(
            permissions.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_refresh_do_nothing() {
        let (test_infra, permissions) =
            TestInfra::setup_users_and_music_folders_no_refresh(2, 2, &[true, false, true, true])
                .await;

        diesel::insert_into(user_music_folder_permissions::table)
            .values(&[permissions[1], permissions[3]])
            .execute(&mut test_infra.pool().get().await.unwrap())
            .await
            .unwrap();

        refresh_permissions(
            test_infra.pool(),
            None,
            Some(&test_infra.music_folder_ids(..)),
        )
        .await
        .unwrap();

        let results = user_music_folder_permissions::table
            .load(&mut test_infra.pool().get().await.unwrap())
            .await
            .unwrap();

        assert_eq!(
            permissions.into_iter().sorted().collect_vec(),
            results.into_iter().sorted().collect_vec()
        );
    }
}
