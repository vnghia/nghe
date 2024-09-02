use diesel_async::RunQueryDsl;
pub use nghe_api::music_folder::add::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::app::state::Database;
use crate::app::{permission, state};
use crate::error::Error;
use crate::filesystem::{self, Trait as _};
use crate::orm::music_folders;

async fn handler_impl<'fs>(
    database: &Database,
    filesystem: filesystem::Impl<'fs>,
    request: Request,
) -> Result<Response, Error> {
    filesystem.check_folder(request.path.as_str().into()).await?;

    let music_folder_id = diesel::insert_into(music_folders::table)
        .values(music_folders::Upsert::from(&request))
        .returning(music_folders::schema::id)
        .get_result::<Uuid>(&mut database.get().await?)
        .await?;

    if request.allow {
        permission::add::handler(
            database,
            permission::add::Request { user_id: None, music_folder_id: Some(music_folder_id) },
        )
        .await?;
    }

    Ok(Response { music_folder_id })
}

#[handler(role = admin)]
pub async fn handler(
    database: &Database,
    filesystem: &state::Filesystem,
    request: Request,
) -> Result<Response, Error> {
    handler_impl(database, filesystem.to_impl(request.filesystem_type)?, request).await
}

#[cfg(test)]
mod tests {
    use nghe_api::music_folder::FilesystemType;
    use rstest::rstest;

    use crate::test::{mock, Mock};

    #[rstest]
    #[case(FilesystemType::Local)]
    #[case(FilesystemType::S3)]
    #[tokio::test]
    async fn test_add(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[case] filesystem_type: FilesystemType,
    ) {
        mock.add_music_folder().filesystem_type(filesystem_type).call().await;
    }
}
