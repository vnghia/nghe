mod get_bookmarks;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        // get_bookmarks
        .route(
            "/rest/getBookmarks",
            get(get_bookmarks::get_bookmarks_handler),
        )
        .route(
            "/rest/getBookmarks.view",
            get(get_bookmarks::get_bookmarks_handler),
        )
}
