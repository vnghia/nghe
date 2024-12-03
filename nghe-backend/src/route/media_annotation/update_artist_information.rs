pub use nghe_api::media_annotation::update_artist_information::{Request, Response};
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::integration::Informant;
use crate::{config, Error};

#[handler(role = admin, internal = true)]
pub async fn handler(
    database: &Database,
    config: config::CoverArt,
    informant: Informant,
    request: Request,
) -> Result<Response, Error> {
    informant.fetch_and_upsert_artist(database, &config, &request).await?;
    Ok(Response)
}
