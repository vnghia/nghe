use nghe_proc_macros::{add_common_convert, add_response_derive, add_subsonic_response};

#[add_common_convert]
#[derive(Debug)]
pub struct GetStarred2Params {}

#[add_response_derive]
pub struct Starred2Result {}

#[add_subsonic_response]
pub struct Starred2Body {
    pub starred2: Starred2Result,
}
