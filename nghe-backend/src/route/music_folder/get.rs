use diesel_async::RunQueryDsl;
pub use nghe_api::music_folder::get::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::error::Error;
use crate::filesystem::{self, Filesystem, Trait as _};
use crate::orm::{music_folders, user_music_folder_permissions};
use crate::route::permission;

#[handler(internal = true)]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    user_music_folder_permissions::Permission::check_owner(database, user_id, request.id).await?;
}
