use axum::Router;
use std::{
    net::SocketAddr,
    sync::Arc
};
use tokio::{
    sync::RwLock
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod data;
use data::{AppState, SharedState};
mod error;
mod utils;
mod web;

#[tokio::main]
async fn main() {

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
    // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    // will be written to stdout.
    .with_max_level(Level::DEBUG)
    // completes the builder.
    .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let shared_state: SharedState = Arc::new(RwLock::new(AppState::new().await));

    let router = Router::new()
        .merge(crate::web::status::routes_status())
        .merge(crate::web::handshake::routes_handshake(shared_state.clone()));

    let bindaddr: SocketAddr = SocketAddr::from(([0,0,0,0], shared_state.read().await.config.port));
    let listener = tokio::net::TcpListener::bind(bindaddr).await.unwrap();
    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

