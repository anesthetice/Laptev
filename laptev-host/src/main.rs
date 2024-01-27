use std::{collections::HashMap, net::SocketAddr, sync::{Arc, RwLock}, fmt::Debug};
use aes_gcm_siv::Aes256GcmSiv;
use axum::Router;
use config::Config;
use tokio;


mod error;
mod config;
mod web;


#[tokio::main]
async fn main() {
    let shared_state: SharedState = Arc::new(RwLock::new(AppState::new().await));
}

type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub config: Config,
    pub db: HashMap<SocketAddr, Aes256GcmSiv>,
}

impl AppState {
    pub async fn new() -> Self {
        AppState {
            config: Config::new().await,
            db: HashMap::new(),
        }
    }
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\n{}", self.config, self.db.len())
    }
}