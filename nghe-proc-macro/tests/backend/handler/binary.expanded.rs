use nghe_proc_macro::handler;
#[tracing::instrument(
    name = "binary",
    skip(database, filesystem),
    ret(level = "debug"),
    err(Debug)
)]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    range: Option<Range>,
    user_id: Uuid,
    request: Request,
) -> Result<binary::Response, Error> {
    let content = "function body should be kept";
    Ok(Response { a: true, b: 1, c: "c" })
}
#[coverage(off)]
#[axum::debug_handler]
#[automatically_derived]
pub async fn request_handler(
    axum::extract::State(database): axum::extract::State<crate::database::Database>,
    axum::extract::Extension(filesystem): axum::extract::Extension<Filesystem>,
    range: Option<axum_extra::TypedHeader<Range>>,
    authenticated_request: crate::http::extract::auth::request::AuthenticatedRequest<
        Request,
    >,
) -> Result<crate::http::binary::Response, crate::Error> {
    handler(
            &database,
            &filesystem,
            range.map(|header| header.0),
            authenticated_request.user.id,
            authenticated_request.request,
        )
        .await
}
