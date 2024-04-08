use axum::extract::State;
use nghe_proc_macros::{add_axum_response, add_common_validate};

use crate::Database;

add_common_validate!(GetBookmarksParams);
add_axum_response!(BookmarksBody);

pub async fn get_bookmarks_handler(
    State(_): State<Database>,
    _: GetBookmarksRequest,
) -> BookmarksJsonResponse {
    Ok(axum::Json(BookmarksBody { bookmarks: BookmarksResult {} }.into()))
}
