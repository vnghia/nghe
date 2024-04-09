use nghe_proc_macros::add_common_convert;
use uuid::Uuid;

#[add_common_convert]
#[derive(Debug)]
pub struct DownloadParams {
    pub id: Uuid,
}
