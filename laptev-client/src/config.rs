use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::IpAddr,
    str::FromStr,
};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub entries: HashMap<IpAddr, Vec<u8>>,
}

impl Config {
    pub fn new() -> Self {
        match Self::load() {
            Ok(config) => {
                tracing::info!("configuration loaded from laptev.config");
                config
            }
            Err(error) => {
                tracing::warn!("failed to load configuration\n{}", error);
                let mut config = Self::default();
                config
                    .entries
                    .insert(IpAddr::from_str("127.0.0.1").unwrap(), vec![0]);
                if let Err(error) = config.save() {
                    tracing::warn!("failed to save generated config\n{}", error);
                }
                config
            }
        }
    }

    fn save(&self) -> anyhow::Result<()> {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("laptev.config")?
            .write_all(&serde_json::to_vec_pretty(&self)?)?;

        Ok(())
    }

    fn load() -> anyhow::Result<Self> {
        let mut buffer: Vec<u8> = Vec::with_capacity(1024);
        std::fs::OpenOptions::new()
            .create(false)
            .read(true)
            .open("laptev.config")?
            .read_to_end(&mut buffer)?;

        Ok(serde_json::from_slice(&buffer)?)
    }
}
