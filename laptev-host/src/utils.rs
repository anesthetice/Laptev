use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::time::{Duration, SystemTime};

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

pub fn get_rng() -> StdRng {
    rand::rngs::StdRng::from_entropy()
}

pub fn rng_fill_bytes(bytes: &mut [u8]) {
    get_rng().fill_bytes(bytes);
}

pub async fn clean_older_than(seconds: u64) {
    let current_timestamp = get_timestamp();
    if let Ok(mut read_dir) = tokio::fs::read_dir("./data").await {
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let entry = entry.path();
            if entry.extension().is_none() {
                continue;
            }
            if entry.extension().unwrap().to_str().is_none() {
                continue;
            }
            if entry.file_stem().is_none() {
                continue;
            }
            if entry.file_stem().unwrap().to_str().is_none() {
                continue;
            }
            if let Ok(timestamp) = entry.file_stem().unwrap().to_str().unwrap().parse::<u64>() {
                if current_timestamp - timestamp < seconds {
                    continue;
                }
                match tokio::fs::remove_file(entry).await {
                    Ok(()) => tracing::info!("removed a file older than {}", seconds),
                    Err(error) => tracing::warn!("failed to remove an old file\n{}", error),
                }
            }
        }
    }
}
