use nghe_proc_macros::{add_common_convert, add_subsonic_response, add_types_derive};

#[add_types_derive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanMode {
    Full,
    Force,
}

#[add_common_convert]
#[derive(Debug)]
pub struct StartScanParams {
    pub scan_mode: ScanMode,
}

#[add_subsonic_response]
pub struct StartScanBody {}
