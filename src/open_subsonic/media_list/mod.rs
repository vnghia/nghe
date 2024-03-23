mod get_starred2;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/getStarred2", get(get_starred2::get_starred2_handler))
        .route("/rest/getStarred2.view", get(get_starred2::get_starred2_handler))
}
