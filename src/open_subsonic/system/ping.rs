use nghe_proc_macros::{add_axum_response, add_subsonic_response};

#[add_subsonic_response]
pub struct PingBody {}
add_axum_response!(PingBody);

pub async fn ping_handler() -> PingJsonResponse {
    PingBody {}.into()
}
