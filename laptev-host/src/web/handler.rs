use crate::{
    data::{external::EncryptedMessage, internal::SharedState},
    error::Error,
};
use axum::{
    body::Bytes, extract::{ConnectInfo, Path, State}, http::StatusCode, response::IntoResponse, routing::{get, delete as del}, Router
};
use tokio::io::AsyncReadExt;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

pub fn routes_handler(state: SharedState) -> Router {
    Router::new()
        .route("/synchronize", get(synchronize))
        .with_state(state.clone())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .route("/download/:id", get(download))
        .with_state(state.clone())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .route("/delete/:id", del(download))
        .with_state(state.clone())
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
    let cipher = match read_state.db.get(&addr.ip()) {
        Some(data) => {
            if data.is_authenticated() {
                Ok(&data.cipher)
            } else {
                Err(Error::NotAuthenticated)
            }
        }
        None => Err(Error::NotAuthenticated)
    }?;

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

    // unwrapping because this should never fail
    let response = EncryptedMessage::new(&bincode::serialize(&body).unwrap(), cipher).unwrap();

    Ok::<_, Error>(Bytes::from(response.into_bytes()))
}

async fn delete(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<u64>,
) -> impl IntoResponse
{
    // checks that the client is authenticated
    if let Some(client) = state.read().await.db.get(&addr.ip()) {
        if !client.is_authenticated() {
            return Err(Error::NotAuthenticated);
        }
    } else {
        return Err(Error::NotAuthenticated)
    }

    // logs the request
    tracing::info!("DELETE REQUEST FOR ENTRY {} FROM {:?}", id, addr);

    // attempts to delete the request
    let filepaths = [PathBuf::from(format!("./data/{}.jpg", id)), PathBuf::from(format!("./data/{}.h264", id))];
    if tokio::fs::remove_file(&filepaths[0]).await.is_ok() && tokio::fs::remove_file(&filepaths[1]).await.is_ok() {
        Ok(StatusCode::OK)
    }
    else {
        Err(Error::InternalError)
    }
}

async fn download(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<u64>,
) -> impl IntoResponse
{
    // checks that the client is authenticated and gets their cipher
    let read_state = state.read().await;
    let cipher = match read_state.db.get(&addr.ip()) {
        Some(data) => {
            if data.is_authenticated() {
                Ok(&data.cipher)
            } else {
                Err(Error::NotAuthenticated)
            }
        }
        None => Err(Error::NotAuthenticated)
    }?;

    // prepares the response
    let mut body: Vec<u8> = Vec::new();
    tokio::fs::OpenOptions::new()
        .create(false)
        .read(true)
        .open(format!("./data/{}.h264", id))
        .await
        .or_else(|error| {
            tracing::warn!("{}", error);
            Err(Error::InternalError)
        })?
        .read_to_end(&mut body)
        .await
        .or_else(|error|{
            tracing::warn!("{}", error);
            Err(Error::InternalError)
        })?;
    
    // unwrapping because this should never fail
    let response = EncryptedMessage::new(&body, &cipher).unwrap();
    Ok::<_, Error>(Bytes::from(response.into_bytes()))
}