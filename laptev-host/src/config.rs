use rand::{SeedableRng, RngCore};
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub password: Vec<u8>,
}

impl Config {
    /// uses Self::load(), Self::generate(), and Self::save() to guarantee a valid configuration is obtained
    pub async fn new() -> Self {
        match Self::load().await {
            Ok(config) => config,
            Err(_) => {
                let config = Self::generate();
                config.save().await.unwrap();
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
            .write_all(serde_json::to_vec_pretty(&self)?.as_ref())
            .await?;
        
        Ok(())
    }

    async fn load() -> anyhow::Result<Self> {
        let mut buffer: Vec<u8> = Vec::with_capacity(4096);
        tokio::fs::OpenOptions::new()
            .create(false)
            .read(true)
            .open("laptev.config")
            .await?
            .read_to_end(&mut buffer)
            .await?;
        println!("{:?}", buffer.len());
        Ok(serde_json::from_slice(&buffer)?)
    }

    fn generate() -> Self {
        let mut password: Vec<u8> = vec![0; 256];
        rand::rngs::StdRng::from_entropy().fill_bytes(&mut password);
        Self { port: 12675, password }
    }
}