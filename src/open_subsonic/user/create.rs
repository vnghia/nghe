use anyhow::Result;
use axum::extract::State;
use diesel::QueryDsl;
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
    create_user(&database, &req.params).await?;
    Ok(axum::Json(CreateUserBody {}.into()))
}

pub async fn create_user(
    Database { pool, key }: &Database,
    params: &CreateUserParams,
) -> Result<Uuid> {
    let CreateUserParams { basic, password, email } = params;
    let password = encrypt_password(key, hex::decode(password)?);

    let user_id = diesel::insert_into(users::table)
        .values(&users::NewUser {
            basic: basic.into(),
            password: password.into(),
            email: email.into(),
        })
        .returning(users::id)
        .get_result::<Uuid>(&mut pool.get().await?)
        .await?;

    let music_folder_ids = music_folders::table
        .select(music_folders::id)
        .get_results::<Uuid>(&mut pool.get().await?)
        .await?;
    set_permission(pool, &[user_id], &music_folder_ids, true).await?;

    Ok(user_id)
}

#[cfg(test)]
mod tests {
    use diesel::ExpressionMethods;
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
}
