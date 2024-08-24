use axum::body::Body;
use axum::http::Request;
use axum::Router;
use tower_http::trace::TraceLayer;

mod auth;
mod common;
mod error;
mod state;
mod user;

pub fn build() -> Router {
    Router::new().merge(user::router()).with_state(state::App::new()).layer(
        TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
            tracing::info_span!(
                "request", method = %request.method(), uri = %request.uri()
            )
        }),
    )
}
