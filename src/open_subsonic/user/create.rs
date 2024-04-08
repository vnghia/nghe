use anyhow::Result;
use axum::extract::State;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::super::permission::set_permission;
use crate::models::*;
use crate::utils::password::encrypt_password;
use crate::Database;

add_common_validate!(CreateUserParams, admin);
add_axum_response!(CreateUserBody);

pub async fn create_user_handler(
    State(database): State<Database>,
    req: CreateUserRequest,
) -> CreateUserJsonResponse {
    create_user(&database, req.params).await?;
    Ok(axum::Json(CreateUserBody {}.into()))
}

pub async fn create_user(
    Database { pool, key }: &Database,
    params: CreateUserParams,
) -> Result<users::User> {
    let CreateUserParams { username, password, email, role } = params;
    let password = encrypt_password(key, &password);

    let user = diesel::insert_into(users::table)
        .values(&users::NewUser {
            username: username.into(),
            password: password.into(),
            email: email.into(),
            role: role.into(),
        })
        .returning(users::User::as_returning())
        .get_result(&mut pool.get().await?)
        .await?;

    let music_folder_ids = music_folders::table
        .select(music_folders::id)
        .get_results::<Uuid>(&mut pool.get().await?)
        .await?;
    set_permission(pool, &[user.id], &music_folder_ids, true).await?;

    Ok(user)
}

#[cfg(test)]
mod tests {
    use diesel::ExpressionMethods;
    use itertools::Itertools;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_create_user_with_music_folders() {
        let infra = Infra::new().await.n_folder(2).await;

        // should re-trigger the refreshing of music folders
        let user = create_user(infra.database(), users::User::fake(None).into_create_params())
            .await
            .unwrap();

        let results = user_music_folder_permissions::table
            .select(user_music_folder_permissions::music_folder_id)
            .filter(user_music_folder_permissions::user_id.eq(user.id))
            .load::<Uuid>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();
        assert_eq!(infra.music_folder_ids(..), results);
    }
}
