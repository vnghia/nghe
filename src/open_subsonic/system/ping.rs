use nghe_proc_macros::{add_axum_response, add_common_validate};

add_common_validate!(PingParams);
add_axum_response!(PingBody);

pub async fn ping_handler(_: PingRequest) -> PingJsonResponse {
    Ok(axum::Json(PingBody {}.into()))
}
