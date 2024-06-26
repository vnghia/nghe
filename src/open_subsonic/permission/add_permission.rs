use anyhow::Result;
use axum::extract::State;
use diesel::{sql_types, IntoSql, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(AddPermissionParams, admin);
add_axum_response!(AddPermissionBody);

pub async fn add_permission(
    pool: &DatabasePool,
    user_id: Option<Uuid>,
    music_folder_id: Option<Uuid>,
) -> Result<()> {
    if let Some(user_id) = user_id {
        if let Some(music_folder_id) = music_folder_id {
            diesel::insert_into(user_music_folder_permissions::table)
                .values(user_music_folder_permissions::NewUserMusicFolderPermission {
                    user_id,
                    music_folder_id,
                })
                .on_conflict_do_nothing()
                .execute(&mut pool.get().await?)
                .await?;
            Ok(())
        } else {
            let new_user_music_folder_permissions = music_folders::table
                .select((user_id.into_sql::<sql_types::Uuid>(), music_folders::id));

            diesel::insert_into(user_music_folder_permissions::table)
                .values(new_user_music_folder_permissions)
                .into_columns((
                    user_music_folder_permissions::user_id,
                    user_music_folder_permissions::music_folder_id,
                ))
                .on_conflict_do_nothing()
                .execute(&mut pool.get().await?)
                .await?;
            Ok(())
        }
    } else if let Some(music_folder_id) = music_folder_id {
        let new_user_music_folder_permissions =
            users::table.select((users::id, music_folder_id.into_sql::<sql_types::Uuid>()));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new_user_music_folder_permissions)
            .into_columns((
                user_music_folder_permissions::user_id,
                user_music_folder_permissions::music_folder_id,
            ))
            .on_conflict_do_nothing()
            .execute(&mut pool.get().await?)
            .await?;
        Ok(())
    } else if cfg!(test) {
        let new_user_music_folder_permissions = users::table
            .inner_join(music_folders::table.on(true.into_sql::<sql_types::Bool>()))
            .select((users::id, music_folders::id));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new_user_music_folder_permissions)
            .into_columns((
                user_music_folder_permissions::user_id,
                user_music_folder_permissions::music_folder_id,
            ))
            .on_conflict_do_nothing()
            .execute(&mut pool.get().await?)
            .await?;
        Ok(())
    } else {
        anyhow::bail!(OSError::InvalidParameter(
            "add permission should have at lease user id or music folder id".into()
        ))
    }
}

pub async fn add_permission_handler(
    State(database): State<Database>,
    req: AddPermissionRequest,
) -> AddPermissionJsonResponse {
    add_permission(&database.pool, req.params.user_id, req.params.music_folder_id).await?;
    Ok(axum::Json(AddPermissionBody {}.into()))
}

#[cfg(test)]
mod tests {
    use diesel::QueryDsl;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_add_permission() {
        let infra = Infra::new().await.n_folder(1).await.add_user_allow(None, false).await;

        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);

        add_permission(infra.pool(), Some(infra.user_id(0)), Some(infra.music_folder_id(0)))
            .await
            .unwrap();
        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_add_user_id() {
        let infra = Infra::new().await.n_folder(2).await.add_user_allow(None, false).await;

        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);

        add_permission(infra.pool(), Some(infra.user_id(0)), None).await.unwrap();
        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_add_music_folder_id() {
        let infra = Infra::new()
            .await
            .n_folder(1)
            .await
            .add_user_allow(None, false)
            .await
            .add_user_allow(None, false)
            .await;

        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);

        add_permission(infra.pool(), None, Some(infra.music_folder_id(0))).await.unwrap();
        let count = user_music_folder_permissions::table
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
