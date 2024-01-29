use aes_gcm_siv::{Aes256GcmSiv, KeyInit};
use std::{
    net::IpAddr,
    sync::Arc,
    collections::HashMap,
    fmt::Debug,
};
use tokio::sync::RwLock;

use crate::{config::Config, utils::get_timestamp};

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub config: Config,
    pub db: HashMap<IpAddr, ClientData>,
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
        let current_time = get_timestamp();

        self.db.retain(|_, value| {
            value.timestamp + self.config.expiration > current_time
        })
    }

    pub fn add_client(&mut self, addr: IpAddr, key: &[u8; 32]) {
        self.update();
        self.db.insert(addr, ClientData::new(key));
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
    authenticated: bool,
}

impl ClientData {
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            timestamp: get_timestamp(),
            // unwrap because our key is guaranteed to be 32 bytes long
            cipher: Aes256GcmSiv::new_from_slice(key).unwrap(),
            authenticated: false,
        }
    }
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
    pub fn authenticate(&mut self) {
        self.authenticated = true;
    }
}

impl Debug for ClientData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client :\ncreation timestamp = {}", self.timestamp)
    }
}

