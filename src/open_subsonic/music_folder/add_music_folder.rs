use anyhow::Result;
use axum::extract::State;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::permission::set_permission;
use crate::{Database, DatabasePool};

add_common_validate!(AddMusicFolderParams, admin);
add_axum_response!(AddMusicFolderBody);

pub async fn add_music_folder(
    pool: &DatabasePool,
    name: &str,
    path: &str,
    permission: bool,
) -> Result<Uuid> {
    let id = diesel::insert_into(music_folders::table)
        .values(music_folders::UpsertMusicFolder {
            name: Some(name.into()),
            path: Some(
                tokio::fs::canonicalize(path)
                    .await?
                    .to_str()
                    .expect("non utf-8 path encountered")
                    .into(),
            ),
        })
        .returning(music_folders::id)
        .get_result::<Uuid>(&mut pool.get().await?)
        .await?;

    let user_ids =
        users::table.select(users::id).get_results::<Uuid>(&mut pool.get().await?).await?;
    set_permission(pool, &user_ids, &[id], permission).await?;

    Ok(id)
}

pub async fn add_music_folder_handler(
    State(database): State<Database>,
    req: AddMusicFolderRequest,
) -> AddMusicFolderJsonResponse {
    add_music_folder(&database.pool, &req.params.name, &req.params.path, req.params.permission)
        .await?;
    Ok(axum::Json(AddMusicFolderBody {}.into()))
}

#[cfg(test)]
mod tests {
    use diesel::dsl::not;
    use diesel::ExpressionMethods;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_add_music_folder() {
        let infra = Infra::new().await.add_user(None).await.add_user(None).await;

        let path = infra.fs.create_dir("folder1/");
        let id =
            add_music_folder(infra.pool(), "folder1", path.to_str().unwrap(), true).await.unwrap();
        let count = user_music_folder_permissions::table
            .filter(user_music_folder_permissions::allow)
            .filter(user_music_folder_permissions::music_folder_id.eq(id))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);

        let path = infra.fs.create_dir("folder2/");
        let id =
            add_music_folder(infra.pool(), "folder2", path.to_str().unwrap(), false).await.unwrap();
        let count = user_music_folder_permissions::table
            .filter(not(user_music_folder_permissions::allow))
            .filter(user_music_folder_permissions::music_folder_id.eq(id))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
