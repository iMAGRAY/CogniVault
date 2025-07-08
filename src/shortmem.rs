use crate::backend::{HubResult, MemoryBackend};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// Simple in-memory LRU-less backend intended mainly for caching & testing.
#[derive(Default)]
pub struct ShortMem {
    inner: Arc<DashMap<String, Vec<u8>>>,
}

#[async_trait]
impl MemoryBackend for ShortMem {
    async fn write(&self, key: String, value: Vec<u8>) -> HubResult<()> {
        self.inner.insert(key, value);
        Ok(())
    }

    async fn read(&self, key: String) -> HubResult<Option<Vec<u8>>> {
        Ok(self.inner.get(&key).map(|v| v.value().clone()))
    }
} 