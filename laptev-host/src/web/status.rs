use crate::{data::internal::SharedState, error::Error};
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

pub fn routes_status(state: SharedState) -> Router {
    Router::new()
        .route("/status", get(status))
        .with_state(state.clone())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

/// returns OK only if the client is authenticated, otherwise returns FORBIDDEN
async fn status(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    if let Some(client) = state.read().await.db.get(&addr.ip()) {
        if !client.is_authenticated() {
            return Err(Error::NotAuthenticated);
        }
    } else {
        return Err(Error::NotAuthenticated);
    }

    Ok(StatusCode::OK)
}
