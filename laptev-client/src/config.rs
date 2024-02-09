use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::IpAddr,
    str::FromStr,
};
use time::UtcOffset;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    // the default address that will be displayed on launch
    pub default_address: String,
    // the maximum amount of entries displayed when synced with the host
    pub size: usize,
    // how many entries the host should skip when syncing
    pub skip: usize,
    // your local time offset, will default to UTC (meaning 0)
    pub local_offset: UtcOffset,
    // hosts and their associated passwords
    pub entries: HashMap<IpAddr, Vec<u8>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut entries = HashMap::new();
        entries.insert(IpAddr::from_str("127.0.0.1").unwrap(), Vec::new());
        Self {
            default_address: String::from("127.0.0.1:12675"),
            size: 25,
            skip: 0,
            local_offset: UtcOffset::from_whole_seconds(0).unwrap(),
            entries,
        }
    }
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
