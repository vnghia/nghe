use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use uuid::Uuid;

use crate::models::*;
use crate::DatabasePool;

pub async fn set_permission(
    pool: &DatabasePool,
    user_ids: &[Uuid],
    music_folder_ids: &[Uuid],
    allow: bool,
) -> Result<()> {
    let new_user_music_folder_permissions = user_ids
        .iter()
        .copied()
        .cartesian_product(music_folder_ids.iter().copied())
        .map(|(user_id, music_folder_id)| {
            user_music_folder_permissions::NewUserMusicFolderPermission {
                user_id,
                music_folder_id,
                allow,
            }
        })
        .collect_vec();
    diesel::insert_into(user_music_folder_permissions::table)
        .values(new_user_music_folder_permissions)
        .on_conflict((
            user_music_folder_permissions::user_id,
            user_music_folder_permissions::music_folder_id,
        ))
        .do_update()
        .set(user_music_folder_permissions::allow.eq(allow))
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use diesel::dsl::not;
    use diesel::QueryDsl;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_set_all() {
        let infra = Infra::new().await.add_user(None).await.add_user(None).await.n_folder(4).await;
        set_permission(infra.pool(), &infra.user_ids(..), &infra.music_folder_ids(..), true)
            .await
            .unwrap();

        let count = user_music_folder_permissions::table
            .filter(user_music_folder_permissions::allow)
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 8);
    }

    #[tokio::test]
    async fn test_set_overwrite() {
        let infra = Infra::new().await.add_user(None).await.add_user(None).await.n_folder(4).await;
        set_permission(infra.pool(), &infra.user_ids(..), &infra.music_folder_ids(..), true)
            .await
            .unwrap();
        set_permission(infra.pool(), &infra.user_ids(..), &infra.music_folder_ids(..1), false)
            .await
            .unwrap();

        let count = user_music_folder_permissions::table
            .filter(not(user_music_folder_permissions::allow))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
