pub use nghe_api::bookmarks::save_playqueue::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::playqueues;
use crate::orm::upsert::Update;

#[handler]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    playqueues::Data::try_from(request)?.update(database, user_id).await?;
    Ok(Response)
}
