use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};

#[add_common_convert]
#[derive(Debug)]
pub struct GetBookmarksParams {}

#[add_types_derive]
pub struct BookmarksResult {}

#[add_subsonic_response]
pub struct BookmarksBody {
    pub bookmarks: BookmarksResult,
}
