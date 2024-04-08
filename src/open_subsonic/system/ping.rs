use nghe_proc_macros::add_axum_response;

add_axum_response!(PingBody);

pub async fn ping_handler() -> PingJsonResponse {
    Ok(axum::Json(PingBody {}.into()))
}
