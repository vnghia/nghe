use nghe_proc_macros::{add_common_convert, add_subsonic_response};

#[add_common_convert]
pub struct PingParams {}

#[add_subsonic_response]
pub struct PingBody {}
