#[coverage(off)]
pub fn router(
    filesystem: crate::filesystem::Filesystem,
    extension: Extension,
    config_network: config::Network,
) -> axum::Router<crate::database::Database> {
    axum::Router::new()
        .route(
            <create::Request as nghe_api::common::JsonURL>::URL_JSON,
            axum::routing::post(create::json_handler),
        )
        .route(
            <read::Request as nghe_api::common::FormURL>::URL_FORM,
            axum::routing::get(read::form_handler).post(read::form_handler),
        )
        .route(
            <read::Request as nghe_api::common::FormURL>::URL_FORM_VIEW,
            axum::routing::get(read::form_handler).post(read::form_handler),
        )
        .route(
            <update::Request as nghe_api::common::FormURL>::URL_FORM,
            axum::routing::get(update::form_handler).post(update::form_handler),
        )
        .route(
            <update::Request as nghe_api::common::FormURL>::URL_FORM_VIEW,
            axum::routing::get(update::form_handler).post(update::form_handler),
        )
        .route(
            <delete::Request as nghe_api::common::JsonURL>::URL_JSON,
            axum::routing::post(delete::json_handler),
        )
        .layer(axum::Extension(filesystem))
        .layer(axum::Extension(extension))
        .layer(axum::Extension(config_network))
}
