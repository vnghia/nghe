use anyhow::Result;
use axum::extract::State;
use diesel::{sql_types, ExpressionMethods, IntoSql, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(SetPermissionParams, admin);
add_axum_response!(SetPermissionBody);

pub async fn set_permission(
    pool: &DatabasePool,
    user_id: Option<Uuid>,
    music_folder_id: Option<Uuid>,
    allow: bool,
) -> Result<()> {
    if let Some(user_id) = user_id {
        if let Some(music_folder_id) = music_folder_id {
            diesel::insert_into(user_music_folder_permissions::table)
                .values(user_music_folder_permissions::NewUserMusicFolderPermission {
                    user_id,
                    music_folder_id,
                    allow,
                })
                .on_conflict((
                    user_music_folder_permissions::user_id,
                    user_music_folder_permissions::music_folder_id,
                ))
                .do_update()
                .set(user_music_folder_permissions::allow.eq(allow))
                .execute(&mut pool.get().await?)
                .await?;
            Ok(())
        } else {
            let new_user_music_folder_permissions = music_folders::table.select((
                user_id.into_sql::<sql_types::Uuid>(),
                music_folders::id,
                allow.into_sql::<sql_types::Bool>(),
            ));

            diesel::insert_into(user_music_folder_permissions::table)
                .values(new_user_music_folder_permissions)
                .into_columns((
                    user_music_folder_permissions::user_id,
                    user_music_folder_permissions::music_folder_id,
                    user_music_folder_permissions::allow,
                ))
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
    } else if let Some(music_folder_id) = music_folder_id {
        let new_user_music_folder_permissions = users::table.select((
            users::id,
            music_folder_id.into_sql::<sql_types::Uuid>(),
            allow.into_sql::<sql_types::Bool>(),
        ));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new_user_music_folder_permissions)
            .into_columns((
                user_music_folder_permissions::user_id,
                user_music_folder_permissions::music_folder_id,
                user_music_folder_permissions::allow,
            ))
            .on_conflict((
                user_music_folder_permissions::user_id,
                user_music_folder_permissions::music_folder_id,
            ))
            .do_update()
            .set(user_music_folder_permissions::allow.eq(allow))
            .execute(&mut pool.get().await?)
            .await?;
        Ok(())
    } else {
        let new_user_music_folder_permissions = users::table
            .inner_join(music_folders::table.on(true.into_sql::<sql_types::Bool>()))
            .select((users::id, music_folders::id, allow.into_sql::<sql_types::Bool>()));

        diesel::insert_into(user_music_folder_permissions::table)
            .values(new_user_music_folder_permissions)
            .into_columns((
                user_music_folder_permissions::user_id,
                user_music_folder_permissions::music_folder_id,
                user_music_folder_permissions::allow,
            ))
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
}

pub async fn set_permission_handler(
    State(database): State<Database>,
    req: SetPermissionRequest,
) -> SetPermissionJsonResponse {
    set_permission(
        &database.pool,
        req.params.user_id,
        req.params.music_folder_id,
        req.params.allow,
    )
    .await?;
    Ok(axum::Json(SetPermissionBody {}.into()))
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
        set_permission(infra.pool(), None, None, true).await.unwrap();

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
        set_permission(infra.pool(), None, None, true).await.unwrap();
        set_permission(infra.pool(), None, Some(infra.music_folder_ids(..1)[0]), false)
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
