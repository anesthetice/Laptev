use crate::{
    data::{external::EncryptedMessage, internal::SharedState},
    error::Error,
};
use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, State},
    response::IntoResponse,
    routing::{get, delete},
    Router,
};
use tokio::io::AsyncReadExt;
use std::net::SocketAddr;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

pub fn routes_handler(state: SharedState) -> Router {
    Router::new()
        .route("/sync", get(synchronize))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

async fn synchronize(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse
{
    // checks that the client is authenticated and gets their cipher

    let read_state = state.read().await;
    let cipher = &read_state.db.get(&addr.ip()).unwrap().cipher;
    
    // prepares the response
    let mut body: Vec<(u64, Vec<u8>)> = Vec::new();

    if let Ok(mut read_dir) = tokio::fs::read_dir("./data").await {
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let entry = entry.path();
            if entry.extension().is_none() {continue}
            if entry.extension().unwrap().to_str().is_none() {continue}
            if entry.extension().unwrap().to_str().unwrap() != "jpg" {continue}
            
            if entry.file_stem().is_none() {continue}
            if entry.file_stem().unwrap().to_str().is_none() {continue}

            if let Ok(timestamp) = entry.file_stem().unwrap().to_str().unwrap().parse::<u64>() {
                
                if let Ok(mut file) = tokio::fs::OpenOptions::new()
                    .create(false)
                    .read(true)
                    .open(entry)
                    .await
                {
                    let mut data: Vec<u8> = Vec::new();
                    if file.read_to_end(&mut data).await.is_ok() {body.push((timestamp, data))}
                }
            }
        }
    }

    let response = EncryptedMessage::new(&bincode::serialize(&body).unwrap(), cipher).unwrap();

    Ok::<_, Error>(Bytes::from(response.into_bytes()))
}
