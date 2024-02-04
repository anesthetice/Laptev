use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

mod config;
mod data;
use data::internal::{AppState, SharedState};
mod error;
mod utils;
mod web;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .compact()
        .init();

    let shared_state: SharedState = Arc::new(RwLock::new(AppState::new().await));

    let router = Router::new()
        .merge(crate::web::status::routes_status(shared_state.clone()))
        .merge(
            crate::web::handshake::routes_handshake(shared_state.clone())
                .merge(crate::web::handler::routes_handler(shared_state.clone())),
        );

    let bindaddr: SocketAddr =
        SocketAddr::from(([127, 0, 0, 1], shared_state.read().await.config.port));
    let listener = tokio::net::TcpListener::bind(bindaddr).await.unwrap();
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
