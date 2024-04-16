use anyhow::Result;
use axum::extract::State;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::utils::check_dir;
use crate::models::*;
use crate::open_subsonic::permission::add_permission;
use crate::{Database, DatabasePool};

add_common_validate!(AddMusicFolderParams, admin);
add_axum_response!(AddMusicFolderBody);

pub async fn add_music_folder(
    pool: &DatabasePool,
    name: &str,
    path: &str,
    allow: bool,
) -> Result<Uuid> {
    let id = diesel::insert_into(music_folders::table)
        .values(music_folders::UpsertMusicFolder {
            name: Some(name.into()),
            path: Some(check_dir(path).await?.to_str().expect("non utf-8 path encountered").into()),
        })
        .returning(music_folders::id)
        .get_result::<Uuid>(&mut pool.get().await?)
        .await?;

    if allow {
        add_permission(pool, None, Some(id)).await?;
    }
    Ok(id)
}

pub async fn add_music_folder_handler(
    State(database): State<Database>,
    req: AddMusicFolderRequest,
) -> AddMusicFolderJsonResponse {
    add_music_folder(&database.pool, &req.params.name, &req.params.path, req.params.allow).await?;
    Ok(axum::Json(AddMusicFolderBody {}.into()))
}

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl};

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_add_music_folder() {
        let infra = Infra::new().await.add_user(None).await.add_user(None).await;

        let path = infra.fs.create_dir("folder1/");
        let id =
            add_music_folder(infra.pool(), "folder1", path.to_str().unwrap(), true).await.unwrap();
        let count = user_music_folder_permissions::table
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
            .filter(user_music_folder_permissions::music_folder_id.eq(id))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
