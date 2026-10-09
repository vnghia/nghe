use nghe_proc_macro::handler;
#[tracing::instrument(name = "no-result", skip(database))]
pub async fn handler(database: &Database, request: Request) -> Response {
    let content = "function body should be kept";
    Response { a: true, b: 1, c: "c" }
}
#[coverage(off)]
#[axum::debug_handler]
#[automatically_derived]
pub async fn request_handler(
    axum::extract::State(database): axum::extract::State<crate::database::Database>,
    request: crate::http::extract::request::Authenticated<Request>,
) -> Result<
    crate::http::serializable::Response<<Request as nghe_api::http::Endpoint>::Response>,
    crate::http::serializable::ErrorResponse,
> {
    let body = handler(&database, request.validated.request).await;
    Ok(crate::http::serializable::Response {
        ty: request.validated.ty,
        body,
    })
}
