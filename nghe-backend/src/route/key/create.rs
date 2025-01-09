use diesel_async::RunQueryDsl;
pub use nghe_api::key::create::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;

#[handler(internal = true)]
pub async fn handler(database: &Database, user_id: Uuid) -> Result<Response, Error> {}
