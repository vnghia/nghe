use nghe_proc_macros::{
    add_common_convert, add_request_types_test, add_subsonic_response, add_types_derive,
};
use uuid::Uuid;

use super::get_scan_status::ScanStatus;

#[add_types_derive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanMode {
    Quick,
    Full,
    Force,
}

#[add_common_convert]
#[derive(Debug)]
pub struct StartScanParams {
    pub id: Uuid,
    pub mode: ScanMode,
}

#[add_subsonic_response]
pub struct StartScanBody {
    pub scan: Option<ScanStatus>,
}

add_request_types_test!(StartScanParams);
