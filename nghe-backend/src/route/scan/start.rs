pub use nghe_api::scan::start::{Request, Response};
use nghe_proc_macro::handler;
use tracing::Instrument;

use crate::database::Database;
use crate::filesystem::Filesystem;
use crate::integration::Informant;
use crate::scan::scanner;
use crate::Error;

#[handler(role = admin, internal = true)]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    config: scanner::Config,
    informant: Informant,
    request: Request,
) -> Result<Response, Error> {
    let scanner =
        scanner::Scanner::new(database, filesystem, config, informant, request.music_folder_id)
            .await?
            .into_owned();

    let span = tracing::Span::current();
    tokio::task::spawn(async move { scanner.run().await }.instrument(span));
    Ok(Response)
}
