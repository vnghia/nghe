use nghe_proc_macro::handler;
#[tracing::instrument(name = "role", skip(database), ret(level = "debug"), err(Debug))]
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
#[automatically_derived]
pub async fn request_handler(
    axum::extract::State(database): axum::extract::State<crate::database::Database>,
    request: crate::http::extract::request::Authenticated<Request>,
) -> Result<
    crate::http::serializable::Response<
        <Request as nghe_api::common::Endpoint>::Response,
    >,
    crate::Error,
> {
    crate::orm::users::Role::check_admin(&database, request.user.id).await?;
    let body = handler(&database, request.user.id, request.validated.request).await?;
    Ok(crate::http::serializable::Response {
        ty: request.validated.ty,
        body,
    })
}
