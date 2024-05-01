use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::permission::add_permission;
use crate::utils::fs::{FsTrait, LocalFs, S3Fs};
use crate::{Database, DatabasePool};

add_common_validate!(AddMusicFolderParams, admin);
add_axum_response!(AddMusicFolderBody);

pub async fn add_music_folder(
    pool: &DatabasePool,
    local_fs: &LocalFs,
    s3_fs: Option<&S3Fs>,
    name: &str,
    path: &str,
    allow: bool,
    fs_type: music_folders::FsType,
) -> Result<Uuid> {
    let id = diesel::insert_into(music_folders::table)
        .values(music_folders::UpsertMusicFolder {
            name: Some(name.into()),
            path: Some(
                match fs_type {
                    music_folders::FsType::Local => local_fs.check_folder(path.as_ref()).await?,
                    music_folders::FsType::S3 => {
                        S3Fs::unwrap(s3_fs)?.check_folder(path.as_ref()).await?
                    }
                }
                .into(),
            ),
            fs_type,
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
    Extension(local_fs): Extension<LocalFs>,
    Extension(s3_fs): Extension<Option<S3Fs>>,
    req: AddMusicFolderRequest,
) -> AddMusicFolderJsonResponse {
    add_music_folder(
        &database.pool,
        &local_fs,
        s3_fs.as_ref(),
        &req.params.name,
        &req.params.path,
        req.params.allow,
        req.params.fs_type.into(),
    )
    .await?;
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

        let path = infra.fs.mkdir(0, "folder1/").await;
        let id = add_music_folder(
            infra.pool(),
            infra.fs.local(),
            infra.fs.s3_option(),
            "folder1",
            &path.to_string(),
            true,
            music_folders::FsType::Local,
        )
        .await
        .unwrap();
        let count = user_music_folder_permissions::table
            .filter(user_music_folder_permissions::music_folder_id.eq(id))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 2);

        let path = infra.fs.mkdir(0, "folder2/").await;
        let id = add_music_folder(
            infra.pool(),
            infra.fs.local(),
            infra.fs.s3_option(),
            "folder2",
            &path.to_string(),
            false,
            music_folders::FsType::Local,
        )
        .await
        .unwrap();
        let count = user_music_folder_permissions::table
            .filter(user_music_folder_permissions::music_folder_id.eq(id))
            .count()
            .get_result::<i64>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
