use nghe_proc_macro::handler;
#[tracing::instrument(
    name = "internal",
    skip(database),
    ret(level = "debug"),
    err(Debug)
)]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let content = "function body should be kept";
    Ok(Response { a: true, b: 1, c: "c" })
}
#[coverage(off)]
#[axum::debug_handler]
pub async fn json_handler(
    axum::extract::State(database): axum::extract::State<crate::database::Database>,
    user: crate::http::extract::auth::Header<Request>,
    axum::Json(request): axum::Json<Request>,
) -> Result<
    axum::Json<<Request as nghe_api::common::JsonEndpoint>::Response>,
    crate::Error,
> {
    let response = handler(&database, user.user.id, request).await?;
    Ok(axum::Json(response))
}
