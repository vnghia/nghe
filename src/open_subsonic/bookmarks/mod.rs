mod get_bookmarks;

use axum::routing::get;
use axum::Router;

pub fn router() -> Router<crate::Database> {
    Router::new()
        .route("/rest/getBookmarks", get(get_bookmarks::get_bookmarks_handler))
        .route("/rest/getBookmarks.view", get(get_bookmarks::get_bookmarks_handler))
}
