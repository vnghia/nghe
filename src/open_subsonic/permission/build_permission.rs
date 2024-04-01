use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::set_permission;
use crate::models::*;
use crate::DatabasePool;

pub async fn build_permission(pool: &DatabasePool) -> Result<Vec<Uuid>> {
    let missing_folder_ids = music_folders::table
        .left_join(user_music_folder_permissions::table)
        .filter(user_music_folder_permissions::user_id.is_null())
        .select(music_folders::id)
        .get_results::<Uuid>(&mut pool.get().await?)
        .await?;
    if !missing_folder_ids.is_empty() {
        let user_ids =
            users::table.select(users::id).get_results::<Uuid>(&mut pool.get().await?).await?;
        set_permission(pool, &user_ids, &missing_folder_ids, true).await?;
    }
    Ok(missing_folder_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_build_nothing() {
        // Since `n_folder` is called before `add_user`, permission has been added already.
        let infra = Infra::new().await.n_folder(2).await.add_user(None).await;

        let missing_folder_ids = build_permission(infra.pool()).await.unwrap();
        assert_eq!(missing_folder_ids.len(), 0);

        let count = user_music_folder_permissions::table
            .filter(user_music_folder_permissions::allow.eq(true))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_build_missing() {
        let infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        infra.permissions(.., ..1, false).await;

        diesel::insert_into(music_folders::table)
            .values(&[music_folders::NewMusicFolder { path: "path".into() }])
            .returning(music_folders::id)
            .get_result::<Uuid>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();

        let missing_folder_ids = build_permission(infra.pool()).await.unwrap();
        assert_eq!(missing_folder_ids.len(), 1);

        let count_allow = user_music_folder_permissions::table
            .filter(user_music_folder_permissions::allow.eq(true))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count_allow, 2);

        // `build_permission` should not override exisiting permissions.
        let deny_ids = user_music_folder_permissions::table
            .filter(user_music_folder_permissions::allow.eq(false))
            .select(user_music_folder_permissions::music_folder_id)
            .get_results::<Uuid>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(infra.music_folder_ids(..1), deny_ids);
    }
}
