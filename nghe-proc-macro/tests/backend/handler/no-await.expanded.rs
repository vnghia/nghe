use nghe_proc_macro::handler;
#[tracing::instrument(
    name = "no-await",
    skip(database),
    ret(level = "debug"),
    err(Debug)
)]
pub fn handler(database: &Database, request: Request) -> Result<Response, Error> {
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
    let body = handler(&database, request.validated.request)
        .map_err(|error| crate::http::serializable::ErrorResponse {
            ty: request.validated.ty,
            error,
        })?;
    Ok(crate::http::serializable::Response {
        ty: request.validated.ty,
        body,
    })
}
