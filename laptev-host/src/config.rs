use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub password: Vec<u8>,
    pub client_expiration_time: u64,
    pub file_expiration_time: u64,
}

impl Config {
    /// uses Self::load(), Self::generate(), and Self::save() to guarantee a valid configuration is obtained
    pub async fn new() -> Self {
        match Self::load().await {
            Ok(config) => {
                tracing::info!("configuration loaded from laptev.config");
                config
            }
            Err(error) => {
                tracing::warn!("failed to load configuration\n{}", error);
                let config = Self::generate();
                if let Err(error) = config.save().await {
                    tracing::warn!("failed to save generated config\n{}", error);
                }
                config
            }
        }
    }

    async fn save(&self) -> anyhow::Result<()> {
        let serialized_data: String = format!(
            "{{\n  \"port\": {},\n  \"password\": {},\n  \"client_expiration_time\": {},\n  \"file_expiration_time\": {}\n}}",
            serde_json::to_string_pretty(&self.port)?,
            serde_json::to_string(&self.password)?,
            serde_json::to_string_pretty(&self.client_expiration_time)?,
            serde_json::to_string_pretty(&self.file_expiration_time)?,
        );

        tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("laptev.config")
            .await?
            .write_all(serialized_data.as_bytes())
            .await?;

        Ok(())
    }

    async fn load() -> anyhow::Result<Self> {
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

    fn generate() -> Self {
        let mut password: Vec<u8> = vec![0; 128];
        rand::rngs::StdRng::from_entropy().fill_bytes(&mut password);
        Self {
            port: 12675,
            password,
            client_expiration_time: 1800,
            file_expiration_time: 259200,
        }
    }
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "port = {}\npassword = {:?}\n", self.port, self.password)
    }
}
