pub use nghe_api::scan::start::{Request, Response};
use nghe_proc_macro::handler;
use tracing::Instrument;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::filesystem::Filesystem;
use crate::integration::Informant;
use crate::orm::user_music_folder_permissions;
use crate::scan::scanner;

#[handler(internal = true)]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    user_id: Uuid,
    config: scanner::Config,
    informant: Informant,
    request: Request,
) -> Result<Response, Error> {
    user_music_folder_permissions::Permission::check_owner(
        database,
        user_id,
        request.music_folder_id,
    )
    .await?;

    let scanner =
        scanner::Scanner::new(database, filesystem, config, informant, request).await?.into_owned();

    let span = tracing::Span::current();
    tokio::task::spawn(async move { scanner.run().await }.instrument(span));
    Ok(Response)
}
