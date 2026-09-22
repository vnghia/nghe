use nghe_proc_macro::handler;
#[tracing::instrument(name = "header", skip(database), ret(level = "debug"), err(Debug))]
pub async fn handler(
    database: &Database,
    range: Option<Range>,
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
    range: Option<axum_extra::TypedHeader<Range>>,
    user: crate::http::extract::auth::Header<Request>,
    axum::Json(request): axum::Json<Request>,
) -> Result<
    axum::Json<<Request as nghe_api::common::JsonEndpoint>::Response>,
    crate::Error,
> {
    let response = handler(
            &database,
            range.map(|header| header.0),
            user.user.id,
            request,
        )
        .await?;
    Ok(axum::Json(response))
}
