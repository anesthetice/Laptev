use axum::{Router, routing::post};

/* 
1. the client posts their public key and receives the server's public key as a response
2. the client and the server each create a symetric cipher with the shared secret
3. the client sends the encrypted password to the server 
*/


/*
pub fn routes_auth<S>(state: S) -> Router 
where S: Clone + Send + Sync
{
    Router::new()
        //.route("/auth/crypt", post(auth()))
        .with_state(state)
}

pub async fn auth() {

}
*/

pub async fn crypt() {
    

}



