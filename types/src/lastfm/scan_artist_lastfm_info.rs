use nghe_proc_macros::{add_common_convert, add_request_types_test, add_subsonic_response};
use time::OffsetDateTime;

use crate::time::time_serde;

#[add_common_convert]
pub struct ScanArtistLastfmInfoParams {
    #[serde(
        with = "time_serde::iso8601_datetime_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub artist_updated_at: Option<OffsetDateTime>,
}

#[add_subsonic_response]
pub struct ScanArtistLastfmInfoBody {}

add_request_types_test!(ScanArtistLastfmInfoParams);
