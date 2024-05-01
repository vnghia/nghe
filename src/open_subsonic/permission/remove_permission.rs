use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(RemovePermissionParams, admin);
add_axum_response!(RemovePermissionBody);

pub async fn remove_permission(
    pool: &DatabasePool,
    user_id: Option<Uuid>,
    music_folder_id: Option<Uuid>,
) -> Result<()> {
    if let Some(user_id) = user_id {
        if let Some(music_folder_id) = music_folder_id {
            diesel::delete(user_music_folder_permissions::table)
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(user_music_folder_permissions::music_folder_id.eq(music_folder_id))
                .execute(&mut pool.get().await?)
                .await?;
            Ok(())
        } else {
            diesel::delete(user_music_folder_permissions::table)
                .filter(user_music_folder_permissions::user_id.eq(user_id))
                .filter(
                    user_music_folder_permissions::music_folder_id
                        .eq_any(music_folders::table.select(music_folders::id)),
                )
                .execute(&mut pool.get().await?)
                .await?;
            Ok(())
        }
    } else if let Some(music_folder_id) = music_folder_id {
        diesel::delete(user_music_folder_permissions::table)
            .filter(user_music_folder_permissions::user_id.eq_any(users::table.select(users::id)))
            .filter(user_music_folder_permissions::music_folder_id.eq(music_folder_id))
            .execute(&mut pool.get().await?)
            .await?;
        Ok(())
    } else if cfg!(test) {
        diesel::delete(user_music_folder_permissions::table)
            .execute(&mut pool.get().await?)
            .await?;
        Ok(())
    } else {
        anyhow::bail!(OSError::InvalidParameter(
            "remove permission should have at lease user id or music folder id".into()
        ))
    }
}

pub async fn remove_permission_handler(
    State(database): State<Database>,
    req: RemovePermissionRequest,
) -> RemovePermissionJsonResponse {
    remove_permission(&database.pool, req.params.user_id, req.params.music_folder_id).await?;
    Ok(axum::Json(RemovePermissionBody {}.into()))
}

#[cfg(test)]
mod tests {
    use diesel::QueryDsl;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_remove_permission() {
        let infra = Infra::new().await.add_user_allow(None, true).await.n_folder(1).await;

        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 1);

        remove_permission(infra.pool(), Some(infra.user_id(0)), Some(infra.music_folder_id(0)))
            .await
            .unwrap();
        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_remove_user_id() {
        let infra =
            Infra::new().await.add_user_allow(None, true).await.n_folder(1).await.n_folder(1).await;

        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);

        remove_permission(infra.pool(), Some(infra.user_id(0)), None).await.unwrap();
        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_remove_music_folder_id() {
        let infra = Infra::new()
            .await
            .add_user_allow(None, true)
            .await
            .add_user_allow(None, true)
            .await
            .n_folder(1)
            .await;

        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);

        remove_permission(infra.pool(), None, Some(infra.music_folder_id(0))).await.unwrap();
        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
