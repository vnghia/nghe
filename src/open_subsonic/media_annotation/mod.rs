mod scrobble;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/scrobble", get(scrobble::scrobble_handler))
        .route("/rest/scrobble.view", get(scrobble::scrobble_handler))
}
