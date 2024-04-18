use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::time::time_serde;

#[add_common_convert]
#[derive(Debug)]
pub struct GetScanStatusParams {
    pub id: Uuid,
}

#[add_types_derive]
#[derive(Debug, Clone, Copy)]
pub struct ScanStatus {
    #[serde(with = "time_serde::iso8601_datetime")]
    pub started_at: OffsetDateTime,
    #[serde(
        with = "time_serde::iso8601_datetime_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub finished_at: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub unrecoverable: Option<bool>,
}

#[add_subsonic_response]
pub struct GetScanStatusBody {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scan: Option<ScanStatus>,
}

add_request_types_test!(GetScanStatusParams);
