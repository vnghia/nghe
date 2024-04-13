use nghe_proc_macros::{add_axum_response, add_common_validate};

add_common_validate!(LoginParams);
add_axum_response!(LoginBody);

pub async fn login_handler(req: LoginRequest) -> LoginJsonResponse {
    Ok(axum::Json(LoginBody { id: req.user_id, role: req.user_role.into() }.into()))
}
