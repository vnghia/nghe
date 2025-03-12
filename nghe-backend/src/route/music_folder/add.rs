use diesel_async::RunQueryDsl;
pub use nghe_api::music_folder::add::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::error::Error;
use crate::filesystem::{self, Filesystem, Trait as _};
use crate::orm::music_folders;
use crate::route::permission;

async fn handler_impl(
    database: &Database,
    filesystem: filesystem::Impl<'_>,
    request: Request,
) -> Result<Response, Error> {
    filesystem.check_folder(request.path.as_str().into()).await?;

    let music_folder_id = diesel::insert_into(music_folders::table)
        .values(music_folders::Upsert::from(&request))
        .returning(music_folders::id)
        .get_result::<Uuid>(&mut database.get().await?)
        .await?;

    if request.allow {
        permission::add::handler(
            database,
            permission::add::Request {
                user_id: None,
                music_folder_id: Some(music_folder_id),
                permission: nghe_api::permission::Permission::default(),
            },
        )
        .await?;
    }

    Ok(Response { music_folder_id })
}

#[handler(role = admin, internal = true)]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    request: Request,
) -> Result<Response, Error> {
    handler_impl(database, filesystem.to_impl(request.ty)?, request).await
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use nghe_api::common::filesystem;
    use rstest::rstest;

    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_add(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
    ) {
        mock.add_music_folder().ty(ty).call().await;
    }
}
