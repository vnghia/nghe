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
pub async fn form_handler(
    axum::extract::State(database): axum::extract::State<crate::database::Database>,
    axum::extract::Extension(filesystem): axum::extract::Extension<Filesystem>,
    range: Option<axum_extra::TypedHeader<Range>>,
    user: crate::http::extract::auth::Form<Request>,
) -> Result<crate::http::binary::Response, crate::Error> {
    handler(
            &database,
            &filesystem,
            range.map(|header| header.0),
            user.user.id,
            user.request,
        )
        .await
}
