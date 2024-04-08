use axum::extract::State;
use nghe_proc_macros::{
    add_axum_response, add_common_convert, add_common_validate, add_subsonic_response,
};
use serde::Serialize;

use crate::Database;

#[add_common_convert]
#[derive(Debug)]
pub struct GetBookmarksParams {}
add_common_validate!(GetBookmarksParams);

#[derive(Serialize)]
pub struct BookmarksResult {}

#[add_subsonic_response]
pub struct BookmarksBody {
    bookmarks: BookmarksResult,
}
add_axum_response!(BookmarksBody);

pub async fn get_bookmarks_handler(
    State(_): State<Database>,
    _: GetBookmarksRequest,
) -> BookmarksJsonResponse {
    BookmarksBody { bookmarks: BookmarksResult {} }.into()
}
