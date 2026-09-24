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
pub async fn form_handler(
    axum::extract::State(database): axum::extract::State<crate::database::Database>,
    user: crate::http::extract::auth::Form<Request>,
) -> Result<
    axum::Json<
        nghe_api::common::SubsonicResponse<
            <Request as nghe_api::common::FormEndpoint>::Response,
        >,
    >,
    crate::Error,
> {
    crate::orm::users::Role::check_admin(&database, user.user.id).await?;
    let response = handler(&database, user.user.id, user.request).await?;
    Ok(axum::Json(nghe_api::common::SubsonicResponse::new(response)))
}
