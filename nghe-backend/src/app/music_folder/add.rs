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
    let Request { name, path, filesystem_type, allow } = request;
    filesystem.check_folder(path.as_str().into()).await?;

    let music_folder_id = diesel::insert_into(music_folders::table)
        .values(music_folders::Upsert {
            name: Some(name.into()),
            path: Some(path.as_str().into()),
            filesystem_type: Some(filesystem_type.into()),
        })
        .returning(music_folders::schema::id)
        .get_result::<Uuid>(&mut database.get().await?)
        .await?;

    if allow {
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
    handler_impl(database, filesystem.to_impl(request.filesystem_type), request).await
}
