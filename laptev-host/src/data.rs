use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
    collections::HashMap,
    fmt::Debug,
};
use aes_gcm_siv::Aes256GcmSiv;

use crate::config::Config;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub config: Config,
    pub db: HashMap<SocketAddr, ClientData>,
}

impl AppState {
    pub async fn new() -> Self {
        AppState {
            config: Config::new().await,
            db: HashMap::new(),
        }
    }
    /// removes a client if it has expired
    /// OPTIONAL TODO: reloads the config if it has changed
    pub fn update(&mut self) {
        let current_time = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

        self.db.retain(|key, value| {
            value.timestamp + self.config.expiration > current_time
        })
    }
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let client_data = self.db.iter().map(|(addr, data)| {
            format!("address = {}\ntimestamp = {}\n", addr.to_string(), data.timestamp)
        }).collect::<Vec<String>>().join("\n");
        write!(f, "[Config]\n{:?}\n[Clients]\n{}", self.config, client_data)
    }
}

pub struct ClientData {
    pub timestamp: u64,
    pub cipher: Aes256GcmSiv,
}

impl Debug for ClientData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client :\ncreation timestamp = {}", self.timestamp)
    }
}