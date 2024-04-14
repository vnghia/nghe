use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use uuid::Uuid;

#[add_common_convert]
pub struct ScrobbleParams {
    #[serde(rename = "id")]
    pub ids: Vec<Uuid>,
    #[serde(rename = "time")]
    pub times: Option<Vec<u64>>,
    pub submission: Option<bool>,
}

#[add_subsonic_response]
pub struct ScrobbleBody {}

add_request_types_test!(ScrobbleParams);
