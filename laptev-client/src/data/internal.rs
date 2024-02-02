use aes_gcm_siv::Aes256GcmSiv;
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedCipher(Arc<Aes256GcmSiv>);

impl SharedCipher {
    pub fn new(cipher: Aes256GcmSiv) -> Self {
        Self(Arc::new(cipher))
    }
}

impl core::ops::Deref for SharedCipher {
    type Target = Arc<Aes256GcmSiv>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Debug for SharedCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedCipher")
    }
}