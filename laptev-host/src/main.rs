use std::{
    sync::{Arc, RwLock},
};
use axum::{extract::FromRef, Router};

mod error;
mod config;
mod web;
mod data;
use data::*;
use rand::SeedableRng;
use x25519_dalek::PublicKey;


#[tokio::main]
async fn main() {
    let shared_state: SharedState = Arc::new(RwLock::new(AppState::new().await));
    use x25519_dalek::EphemeralSecret;
    
    let server_private = EphemeralSecret::random_from_rng(rand::rngs::StdRng::from_entropy());
    let server_public = PublicKey::from(&server_private);
    //PublicKey::from_ref(input)

}

