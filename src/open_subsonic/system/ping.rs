use nghe_proc_macros::wrap_subsonic_response;

#[wrap_subsonic_response]
pub struct PingBody {}

pub async fn ping_handler() -> PingJsonResponse {
    PingBody {}.into()
}
