use crate::Database;

use axum::extract::State;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;

#[add_validate]
#[derive(Debug)]
pub struct GetBookmarksParams {}

#[derive(Serialize)]
pub struct BookmarksResult {}

#[wrap_subsonic_response]
pub struct BookmarksBody {
    bookmarks: BookmarksResult,
}

pub async fn get_bookmarks_handler(
    State(_): State<Database>,
    _: GetBookmarksRequest,
) -> BookmarksJsonResponse {
    BookmarksBody {
        bookmarks: BookmarksResult {},
    }
    .into()
}
