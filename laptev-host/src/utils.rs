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
