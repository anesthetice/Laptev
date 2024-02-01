use aes_gcm_siv::{aead::Aead, Aes256GcmSiv};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::utils::rng_fill_bytes;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EncryptedMessage {
    nonce: [u8; 12],
    data: Vec<u8>,
}

impl EncryptedMessage {
    pub fn new(unencrypted_data: &[u8], cipher: &Aes256GcmSiv) -> Result<Self> {
        let mut nonce: [u8; 12] = [0; 12];
        rng_fill_bytes(&mut nonce);
        Ok(Self {
            nonce: nonce,
            data: cipher.encrypt(&nonce.into(), unencrypted_data)?,
        })
    }
    pub fn try_from_bytes(data: &[u8]) -> Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
    pub fn try_decrypt(&self, cipher: &Aes256GcmSiv) -> Result<Vec<u8>> {
        let mut data = cipher.decrypt(&self.nonce.into(), self.data.as_ref())?;
        println!("{:?}", data);
        // removes the first 8 bytes that give us the length of the plaintext
        // really not useful to us since we are not using associated data
        // if data.len() >= 8 {data.drain(0..8);}
        Ok(data)
    }
    pub fn into_bytes(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn encrypted_message() {
        use super::EncryptedMessage;
        use aes_gcm_siv::{Aes256GcmSiv, KeyInit};
        use rand::SeedableRng;

        let cipher: Aes256GcmSiv = Aes256GcmSiv::new(&Aes256GcmSiv::generate_key(
            rand::rngs::StdRng::from_entropy(),
        ));
        let initial_data: Vec<u8> = vec![101, 21, 211, 1];
        let encrypted_message = EncryptedMessage::new(&initial_data, &cipher).unwrap();
        assert_eq!(
            initial_data,
            encrypted_message.try_decrypt(&cipher).unwrap()
        )
    }
}
