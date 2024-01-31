use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, str::FromStr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub entries: HashMap<IpAddr, Vec<u8>>,
}

impl Config {
    pub async fn new() -> Self {
        match Self::load().await {
            Ok(config) => {
                tracing::info!("configuration loaded from laptev.config");
                config
            }
            Err(_) => {
                tracing::info!("failed to load configuration");
                let mut config = Self::default();
                config
                    .entries
                    .insert(IpAddr::from_str("127.0.0.1").unwrap(), vec![12, 24]);
                if config.save().await.is_err() {
                    tracing::warn!("failed to save generated config")
                }
                config
            }
        }
    }

    async fn save(&self) -> anyhow::Result<()> {
        let _ = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("laptev.config")
            .await?
            .write_all(&serde_json::to_vec_pretty(&self)?)
            .await?;

        Ok(())
    }

    pub async fn load() -> anyhow::Result<Self> {
        let mut buffer: Vec<u8> = Vec::with_capacity(1024);
        tokio::fs::OpenOptions::new()
            .create(false)
            .read(true)
            .open("laptev.config")
            .await?
            .read_to_end(&mut buffer)
            .await?;
        Ok(serde_json::from_slice(&buffer)?)
    }
}
