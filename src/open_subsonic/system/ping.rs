use crate::OSResult;

use nghe_proc_macros::wrap_subsonic_response;

#[wrap_subsonic_response]
#[derive(Debug)]
pub struct PingBody {}

pub async fn ping_handler() -> OSResult<PingResponse> {
    Ok(PingBody {}.into())
}
