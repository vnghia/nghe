use crate::{models::*, DatabasePool, OSResult, OpenSubsonicError};

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
) -> OSResult<Cow<'a, [Uuid]>> {
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
            Err(OpenSubsonicError::Forbidden {
                message: Some("current user doesn't have access to these music folders".into()),
            })
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
    use crate::utils::test::setup::setup_users_and_music_folders;

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
        let (temp_db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(1, 2, &[true, true]).await;
        let music_folder_ids = check_user_music_folder_ids(temp_db.pool(), &users[0].id, None)
            .await
            .unwrap();
        assert_music_folder_ids(&music_folders, music_folder_ids);
    }

    #[tokio::test]
    async fn test_check_music_folder_ids_none_partial_allow() {
        let (temp_db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(1, 2, &[true, false]).await;
        let music_folder_ids = check_user_music_folder_ids(temp_db.pool(), &users[0].id, None)
            .await
            .unwrap();
        assert_music_folder_ids(&music_folders[0..1], music_folder_ids);
    }

    #[tokio::test]
    async fn test_check_music_folder_ids_all_allow() {
        let (temp_db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(1, 2, &[true, true]).await;
        let fs_music_folder_ids = music_folders.iter().map(|m| m.id).collect_vec();
        let music_folder_ids = check_user_music_folder_ids(
            temp_db.pool(),
            &users[0].id,
            Some(fs_music_folder_ids.into()),
        )
        .await
        .unwrap();
        assert_music_folder_ids(&music_folders, music_folder_ids);
    }

    #[tokio::test]
    async fn test_check_music_folder_ids_partial_allow() {
        let (temp_db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(1, 2, &[true, false]).await;
        let fs_music_folder_ids = music_folders.iter().map(|m| m.id).collect_vec();
        assert!(matches!(
            check_user_music_folder_ids(
                temp_db.pool(),
                &users[0].id,
                Some(fs_music_folder_ids.into()),
            )
            .await,
            Err(OpenSubsonicError::Forbidden { message: _ })
        ));
    }
}
