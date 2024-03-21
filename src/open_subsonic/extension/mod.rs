mod get_open_subsonic_extensions;

use axum::{routing::get, Router};

pub fn router() -> Router<crate::Database> {
    Router::new()
        // get_open_subsonic_extensions
        .route(
            "/rest/getOpenSubsonicExtensions",
            get(get_open_subsonic_extensions::get_open_subsonic_extensions_handler),
        )
        .route(
            "/rest/getOpenSubsonicExtensions.view",
            get(get_open_subsonic_extensions::get_open_subsonic_extensions_handler),
        )
}
