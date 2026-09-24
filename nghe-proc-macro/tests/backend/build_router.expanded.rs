#[coverage(off)]
pub fn router(
    filesystem: crate::filesystem::Filesystem,
    extension: Extension,
    config_network: config::Network,
) -> axum::Router<crate::database::Database> {
    axum::Router::new()
        .route(
            <create::Request as nghe_api::common::EndpointURL>::URL,
            axum::routing::any(create::request_handler),
        )
        .route(
            <create::Request as nghe_api::common::EndpointURL>::URL_VIEW,
            axum::routing::any(create::request_handler),
        )
        .route(
            <read::Request as nghe_api::common::EndpointURL>::URL,
            axum::routing::any(read::request_handler),
        )
        .route(
            <read::Request as nghe_api::common::EndpointURL>::URL_VIEW,
            axum::routing::any(read::request_handler),
        )
        .route(
            <update::Request as nghe_api::common::EndpointURL>::URL,
            axum::routing::any(update::request_handler),
        )
        .route(
            <update::Request as nghe_api::common::EndpointURL>::URL_VIEW,
            axum::routing::any(update::request_handler),
        )
        .route(
            <delete::Request as nghe_api::common::EndpointURL>::URL,
            axum::routing::any(delete::request_handler),
        )
        .route(
            <delete::Request as nghe_api::common::EndpointURL>::URL_VIEW,
            axum::routing::any(delete::request_handler),
        )
        .layer(axum::Extension(filesystem))
        .layer(axum::Extension(extension))
        .layer(axum::Extension(config_network))
}
