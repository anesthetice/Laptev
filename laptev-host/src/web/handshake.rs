use crate::{
    data::{external::EncryptedMessage, internal::SharedState},
    error::Error,
};
use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, State},
    response::IntoResponse,
    routing::put,
    Router,
};
use rand::{rngs::StdRng, SeedableRng};
use std::net::SocketAddr;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub fn routes_handshake(state: SharedState) -> Router {
    Router::new()
        .route("/handshake/:id", put(handshake))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

async fn handshake(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<u8>,
    body: Bytes,
) -> impl IntoResponse {
    if id == 0 {
        let mut client_public_key: [u8; 32] = [0; 32];
        if body.len() != 32 {
            return Err(Error::HandshakeFailed);
        }
        for (idx, byte) in body.into_iter().enumerate() {
            client_public_key[idx] = byte;
        }
        let client_public_key = PublicKey::from(client_public_key);
        let server_private_key = EphemeralSecret::random_from_rng(StdRng::from_entropy());
        let server_public_key = PublicKey::from(&server_private_key);

        state.write().await.add_client(
            addr.ip(),
            server_private_key
                .diffie_hellman(&client_public_key)
                .as_bytes(),
        );
        return Ok(Bytes::copy_from_slice(server_public_key.as_bytes()));
    }
    if id == 1 {
        let mut valid: bool = false;
        if let Some(client_data) = state.read().await.db.get(&addr.ip()) {
            if let Ok(encrypted_message) = EncryptedMessage::try_from_bytes(&body) {
                if let Ok(decrypted_password_provided_by_client) =
                    encrypted_message.try_decrypt(&client_data.cipher)
                {
                    if decrypted_password_provided_by_client == state.read().await.config.password {
                        tracing::info!("{:?} authenticated", &addr.ip());
                        valid = true;
                    }
                }
            }
        }
        if valid {
            if let Some(client_data) = state.write().await.db.get_mut(&addr.ip()) {
                client_data.authenticate();
                return Ok(Bytes::new());
            }
        }
    }
    Err(Error::HandshakeFailed)
}
