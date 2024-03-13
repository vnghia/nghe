use crate::{models::*, DatabasePool, OSError};

use anyhow::Result;
use diesel::{
    dsl::{exists, not},
    select, ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use std::borrow::Cow;
use uuid::Uuid;

pub async fn check_user_music_folder_ids<'a>(
    pool: &DatabasePool,
    user_id: &Uuid,
    music_folder_ids: Option<Cow<'a, [Uuid]>>,
) -> Result<Cow<'a, [Uuid]>> {
    if let Some(music_folder_ids) = music_folder_ids {
        if select(not(exists(
            user_music_folder_permissions::table
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(
                    user_music_folder_permissions::music_folder_id
                        .eq_any(music_folder_ids.as_ref()),
                )
                .filter(not(user_music_folder_permissions::allow)),
        )))
        .first::<bool>(&mut pool.get().await?)
        .await?
        {
            Ok(music_folder_ids)
        } else {
            anyhow::bail!(OSError::Forbidden("access to these music folders".into()))
        }
    } else {
        Ok(user_music_folder_permissions::table
            .select(user_music_folder_permissions::music_folder_id)
            .filter(user_music_folder_permissions::user_id.eq(user_id))
            .filter(user_music_folder_permissions::allow)
            .get_results::<Uuid>(&mut pool.get().await?)
            .await?
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::setup::TestInfra;

    use itertools::Itertools;

    fn assert_music_folder_ids(
        music_folders: &[music_folders::MusicFolder],
        music_folder_ids: Cow<'_, [Uuid]>,
    ) {
        assert_eq!(
            music_folders.iter().map(|m| &m.id).sorted().collect_vec(),
            music_folder_ids.iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_check_music_folder_ids_none_all_allow() {
        let test_infra = TestInfra::setup_users_and_music_folders(1, 2, &[true, true]).await;
        let music_folder_ids =
            check_user_music_folder_ids(test_infra.pool(), &test_infra.users[0].id, None)
                .await
                .unwrap();
        assert_music_folder_ids(&test_infra.music_folders, music_folder_ids);
    }

    #[tokio::test]
    async fn test_check_music_folder_ids_none_partial_allow() {
        let test_infra = TestInfra::setup_users_and_music_folders(1, 2, &[true, false]).await;
        let music_folder_ids =
            check_user_music_folder_ids(test_infra.pool(), &test_infra.users[0].id, None)
                .await
                .unwrap();
        assert_music_folder_ids(&test_infra.music_folders[0..1], music_folder_ids);
    }

    #[tokio::test]
    async fn test_check_music_folder_ids_all_allow() {
        let test_infra = TestInfra::setup_users_and_music_folders(1, 2, &[true, true]).await;
        let fs_music_folder_ids = test_infra.music_folder_ids(..);
        let music_folder_ids = check_user_music_folder_ids(
            test_infra.pool(),
            &test_infra.users[0].id,
            Some(fs_music_folder_ids.into()),
        )
        .await
        .unwrap();
        assert_music_folder_ids(&test_infra.music_folders, music_folder_ids);
    }

    #[tokio::test]
    async fn test_check_music_folder_ids_partial_allow() {
        let test_infra = TestInfra::setup_users_and_music_folders(1, 2, &[true, false]).await;
        let fs_music_folder_ids = test_infra.music_folder_ids(..);
        assert!(matches!(
            check_user_music_folder_ids(
                test_infra.pool(),
                &test_infra.users[0].id,
                Some(fs_music_folder_ids.into()),
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::Forbidden(_)
        ));
    }
}
