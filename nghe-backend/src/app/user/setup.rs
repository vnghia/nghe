#![allow(clippy::unused_async)]

pub use nghe_api::user::setup::{Request, Response};
use nghe_proc_macro::handler;

use crate::app::error::Error;
use crate::app::state::Database;

#[handler]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    Ok(Response {})
}
