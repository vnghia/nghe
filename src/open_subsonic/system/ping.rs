use crate::OSResult;

use serde::Serialize;

use nghe_proc_macros::wrap_subsonic_response;

#[wrap_subsonic_response]
#[derive(Debug, Serialize)]
pub struct PingBody {}

pub async fn ping_handler() -> OSResult<PingResponse> {
    Ok(PingBody {}.into())
}
