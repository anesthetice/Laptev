use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, State},
    response::IntoResponse,
    Router,
    routing::put
};
use rand::{
    rngs::StdRng,
    SeedableRng
};
use std::{
    net::SocketAddr
};
use x25519_dalek::{
    EphemeralSecret,
    PublicKey,
};
use crate::{
    data::SharedState,
    error::Error
};

/* 
1. the client posts their public key and receives the server's public key as a response
2. the client and the server each create a symetric cipher with the shared secret
3. the client sends the encrypted password to the server 
*/

pub fn routes_handshake(state: SharedState) -> Router 
{
    Router::new()
        .route("/handshake/:id", put(handshake))
        .with_state(state)
}

async fn handshake(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(id): Path<u8>,
    body: Bytes,
) -> impl IntoResponse
{   
    if id == 0 {
        let mut client_public_key: [u8; 32] = [0; 32];
        if body.len() != 32 {return Err(Error::AuthenticationFailed)}
        for (idx, byte) in body.into_iter().enumerate() {
            client_public_key[idx] = byte;
        }
        let client_public_key = PublicKey::from(client_public_key);    
        let server_private_key = EphemeralSecret::random_from_rng(StdRng::from_entropy());
        let server_public_key = PublicKey::from(&server_private_key);
        
        state.write().await.add_client(addr.ip(), server_private_key.diffie_hellman(&client_public_key).as_bytes());
        println!("{:?}", state.read().await);
        Ok(Bytes::copy_from_slice(server_public_key.as_bytes()))
    }
    else {
        Err(Error::AuthenticationFailed)
    }
    
}