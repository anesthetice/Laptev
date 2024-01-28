use axum::{
    extract::{State, ConnectInfo, Query},
    http::Response,
    Router,
    routing::post
};
use rand::{
    rngs::StdRng,
    SeedableRng
};
use std::{
    collections::HashMap,
    net::SocketAddr
};
use x25519_dalek::{
    EphemeralSecret,
    PublicKey,
};
use crate::{
    data::SharedState,
    error::{Error, Result}
};

/* 
1. the client posts their public key and receives the server's public key as a response
2. the client and the server each create a symetric cipher with the shared secret
3. the client sends the encrypted password to the server 
*/

pub fn routes_handshake(state: SharedState) -> Router 
{
    Router::new()
        .route("/handshake", post(handshake))
        .with_state(state)
}

async fn handshake(
    Query(params): Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<SharedState>
) -> Result<Response<String>>
{   
    if let Some(string) = params.get("key-exchange") {
        let mut client_public_key: [u8; 32] = [0; 32];
        for (idx, subslice) in string.split("-").enumerate() {
            if idx < 32 {
                if let Ok(num) = subslice.parse::<u8>() {
                    client_public_key[idx] = num;
                    continue;
                }
            }
            return Err(Error::CryptographyFailed);
        }
        let client_public_key = PublicKey::from(client_public_key);    
        let server_private_key = EphemeralSecret::random_from_rng(StdRng::from_entropy());
        
        let body = PublicKey::from(&server_private_key)
        .as_bytes()
        .into_iter()
        .fold(String::new(), |acc, x| {acc + &x.to_string()});
        
        state.write().await.add_client(addr, server_private_key.diffie_hellman(&client_public_key).as_bytes());
    
        Ok(Response::new(body))
    }
    else {
        Ok(Response::new("Ok".to_string()))
    }
    
}