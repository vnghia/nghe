use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::utils::check_dir;
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(UpdateMusicFolderParams, admin);
add_axum_response!(UpdateMusicFolderBody);

pub async fn update_music_folder(
    pool: &DatabasePool,
    id: Uuid,
    name: &Option<String>,
    path: &Option<String>,
) -> Result<()> {
    let path = if let Some(ref path) = path {
        Some(
            check_dir(path)
                .await?
                .into_os_string()
                .into_string()
                .expect("non utf-8 path encountered")
                .into(),
        )
    } else {
        None
    };

    diesel::update(music_folders::table.filter(music_folders::id.eq(id)))
        .set(music_folders::UpsertMusicFolder { name: name.as_ref().map(|s| s.into()), path })
        .execute(&mut pool.get().await?)
        .await?;

    Ok(())
}

pub async fn update_music_folder_handler(
    State(database): State<Database>,
    req: UpdateMusicFolderRequest,
) -> UpdateMusicFolderJsonResponse {
    update_music_folder(&database.pool, req.params.id, &req.params.name, &req.params.path).await?;
    Ok(axum::Json(UpdateMusicFolderBody {}.into()))
}

#[cfg(test)]
mod tests {
    use super::super::add_music_folder::add_music_folder;
    use super::*;
    use crate::utils::path::LocalPath;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_update_music_folder() {
        let infra = Infra::new().await.add_user(None).await.add_user(None).await;

        let path = infra.fs.mkdir::<LocalPath>("folder/").await;
        let id = add_music_folder(infra.pool(), "folder", &path.to_string(), true).await.unwrap();
        update_music_folder(infra.pool(), id, &Some("new-folder".into()), &None).await.unwrap();

        let new_id = music_folders::table
            .filter(music_folders::name.eq("new-folder"))
            .select(music_folders::id)
            .get_result::<Uuid>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(new_id, id);
    }
}
