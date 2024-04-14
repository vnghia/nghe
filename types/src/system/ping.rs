use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};

#[add_common_convert]
pub struct PingParams {}

#[add_subsonic_response]
pub struct PingBody {}

add_request_types_test!(PingParams);
