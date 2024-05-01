use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::utils::fs::{FsTrait, LocalFs, S3Fs};
use crate::{Database, DatabasePool};

add_common_validate!(UpdateMusicFolderParams, admin);
add_axum_response!(UpdateMusicFolderBody);

pub async fn update_music_folder(
    pool: &DatabasePool,
    local_fs: &LocalFs,
    s3_fs: Option<&S3Fs>,
    id: Uuid,
    name: &Option<String>,
    path: &Option<String>,
    fs_type: music_folders::FsType,
) -> Result<()> {
    let path = if let Some(ref path) = path {
        Some(
            match fs_type {
                music_folders::FsType::Local => local_fs.check_folder(path.as_ref()).await?,
                music_folders::FsType::S3 => {
                    S3Fs::unwrap(s3_fs)?.check_folder(path.as_ref()).await?
                }
            }
            .into(),
        )
    } else {
        None
    };

    diesel::update(music_folders::table.filter(music_folders::id.eq(id)))
        .set(music_folders::UpsertMusicFolder {
            name: name.as_ref().map(|s| s.into()),
            path,
            fs_type,
        })
        .execute(&mut pool.get().await?)
        .await?;

    Ok(())
}

pub async fn update_music_folder_handler(
    State(database): State<Database>,
    Extension(local_fs): Extension<LocalFs>,
    Extension(s3_fs): Extension<Option<S3Fs>>,
    req: UpdateMusicFolderRequest,
) -> UpdateMusicFolderJsonResponse {
    update_music_folder(
        &database.pool,
        &local_fs,
        s3_fs.as_ref(),
        req.params.id,
        &req.params.name,
        &req.params.path,
        req.params.fs_type.into(),
    )
    .await?;
    Ok(axum::Json(UpdateMusicFolderBody {}.into()))
}

#[cfg(test)]
mod tests {
    use super::super::add_music_folder::add_music_folder;
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_update_music_folder() {
        let infra = Infra::new().await.add_user(None).await.add_user(None).await;

        let path = infra.fs.mkdir(music_folders::FsType::Local, "folder/").await;
        let id = add_music_folder(
            infra.pool(),
            infra.fs.local(),
            infra.fs.s3_option(),
            "folder",
            &path.to_string(),
            true,
            music_folders::FsType::Local,
        )
        .await
        .unwrap();
        update_music_folder(
            infra.pool(),
            infra.fs.local(),
            infra.fs.s3_option(),
            id,
            &Some("new-folder".into()),
            &None,
            music_folders::FsType::Local,
        )
        .await
        .unwrap();

        let new_id = music_folders::table
            .filter(music_folders::name.eq("new-folder"))
            .select(music_folders::id)
            .get_result::<Uuid>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert_eq!(new_id, id);
    }
}
