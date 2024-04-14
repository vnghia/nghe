use nghe_proc_macros::{add_common_convert, add_request_types_test};
use uuid::Uuid;

#[add_common_convert]
#[derive(Debug)]
pub struct DownloadParams {
    pub id: Uuid,
}

add_request_types_test!(DownloadParams);
