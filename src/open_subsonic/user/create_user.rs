use anyhow::Result;
use axum::extract::State;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::super::permission::add_permission;
use crate::models::*;
use crate::utils::password::encrypt_password;
use crate::Database;

add_common_validate!(CreateUserParams, admin);
add_axum_response!(CreateUserBody);

pub async fn create_user_handler(
    State(database): State<Database>,
    req: CreateUserRequest,
) -> CreateUserJsonResponse {
    create_user(&database, &req.params).await?;
    Ok(axum::Json(CreateUserBody {}.into()))
}

pub async fn create_user(
    Database { pool, key }: &Database,
    params: &CreateUserParams,
) -> Result<Uuid> {
    let CreateUserParams {
        username,
        password,
        email,
        admin_role,
        stream_role,
        download_role,
        share_role,
        allow,
    } = params;
    let password = encrypt_password(key, hex::decode(password)?);

    let user_id = diesel::insert_into(users::table)
        .values(&users::NewUser {
            basic: users::BasicUser {
                username: username.into(),
                role: users::Role {
                    admin_role: *admin_role,
                    stream_role: *stream_role,
                    download_role: *download_role,
                    share_role: *share_role,
                },
            },
            password: password.into(),
            email: email.into(),
        })
        .returning(users::id)
        .get_result::<Uuid>(&mut pool.get().await?)
        .await?;

    if *allow {
        add_permission(pool, Some(user_id), None).await?;
    }
    Ok(user_id)
}

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl};
    use itertools::Itertools;

    use super::*;
    use crate::utils::test::{Infra, User};

    #[tokio::test]
    async fn test_create_user_with_music_folders() {
        let infra = Infra::new().await.n_folder(2).await;

        // should re-trigger the refreshing of music folders
        let user_id = create_user(infra.database(), &User::fake(None).into()).await.unwrap();

        let results = user_music_folder_permissions::table
            .select(user_music_folder_permissions::music_folder_id)
            .filter(user_music_folder_permissions::user_id.eq(user_id))
            .load::<Uuid>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();
        assert_eq!(infra.music_folder_ids(..), results);
    }

    #[tokio::test]
    async fn test_create_user_with_no_music_folders() {
        let infra = Infra::new().await.n_folder(2).await;

        // should re-trigger the refreshing of music folders
        let user_id = create_user(
            infra.database(),
            &CreateUserParams { allow: false, ..User::fake(None).into() },
        )
        .await
        .unwrap();

        let results = user_music_folder_permissions::table
            .select(user_music_folder_permissions::music_folder_id)
            .filter(user_music_folder_permissions::user_id.eq(user_id))
            .load::<Uuid>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();
        assert!(results.is_empty());
    }
}
