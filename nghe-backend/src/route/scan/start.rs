pub use nghe_api::scan::start::{Request, Response};
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::filesystem::Filesystem;
use crate::scan::scanner;
use crate::Error;

#[handler(role = admin)]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    config: scanner::Config,
    request: Request,
) -> Result<Response, Error> {
    let scanner = scanner::Scanner::new(database, filesystem, config, request.music_folder_id)
        .await?
        .into_owned();
    tokio::task::spawn(async move { scanner.run().await });
    Ok(Response)
}
