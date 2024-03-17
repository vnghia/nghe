mod download;
mod utils;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new().route("/rest/download", get(download::download_handler))
}
