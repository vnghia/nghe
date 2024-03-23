pub mod search3;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/search3", get(search3::search3_handler))
        .route("/rest/search3.view", get(search3::search3_handler))
}
