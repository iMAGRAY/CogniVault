use crate::backend::{HubResult, MemoryBackend};
use async_trait::async_trait;
#[cfg(feature = "longmem_encrypt")] use aes_gcm_siv::{aead::{Aead, KeyInit, OsRng, generic_array::GenericArray}, Aes256GcmSiv};

/// Persistent storage backend backed by sled key-value database.
/// If feature `longmem_encrypt` is enabled, values are encrypted with random nonce
/// using AES-256-GCM-SIV.
pub struct LongMem {
    db: sled::Db,
    #[cfg(feature = "longmem_encrypt")]
    cipher: Aes256GcmSiv,
}

impl LongMem {
    /// Open/create database at `path`. When encryption is enabled, `key` must be Some(32 bytes).
    pub fn open(path: &std::path::Path, key: Option<[u8;32]>) -> HubResult<Self> {
        let db = sled::open(path)?;
        #[cfg(feature = "longmem_encrypt")]
        {
            let key_bytes = key.ok_or_else(|| anyhow::anyhow!("encryption key required"))?;
            let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key_bytes));
            Ok(Self { db, cipher })
        }
        #[cfg(not(feature = "longmem_encrypt"))]
        {
            let _ = key;
            Ok(Self { db })
        }
    }

    #[cfg(feature = "longmem_encrypt")]
    fn encrypt(&self, plaintext: &[u8]) -> HubResult<Vec<u8>> {
        use aes_gcm_siv::aead::rand_core::RngCore;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let nonce_ga = GenericArray::from_slice(&nonce);
        let mut ciphertext = self.cipher.encrypt(nonce_ga, plaintext)?;
        // prepend nonce
        let mut combined = nonce.to_vec();
        combined.append(&mut ciphertext);
        Ok(combined)
    }

    #[cfg(feature = "longmem_encrypt")]
    fn decrypt(&self, data: &[u8]) -> HubResult<Vec<u8>> {
        if data.len() < 12 { return Err(anyhow::anyhow!("ciphertext too short")); }
        let (nonce, ct) = data.split_at(12);
        let nonce_ga = GenericArray::from_slice(nonce);
        Ok(self.cipher.decrypt(nonce_ga, ct)? )
    }
}

#[async_trait]
impl MemoryBackend for LongMem {
    async fn write(&self, key: String, mut value: Vec<u8>) -> HubResult<()> {
        #[cfg(feature = "longmem_encrypt")]
        { value = self.encrypt(&value)?; }
        self.db.insert(key, value)?;
        Ok(())
    }

    async fn read(&self, key: String) -> HubResult<Option<Vec<u8>>> {
        match self.db.get(key)? {
            Some(v) => {
                let mut bytes = v.to_vec();
                #[cfg(feature = "longmem_encrypt")]
                { bytes = self.decrypt(&bytes)?; }
                Ok(Some(bytes))
            }
            None => Ok(None)
        }
    }
} 