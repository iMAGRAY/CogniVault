use crate::backend::{HubResult, MemoryBackend};
use futures::future::join_all;
use std::sync::Arc;
#[cfg(feature = "dev_metrics")] use metrics::{counter, histogram};

/// Central router coordinating access to multiple memory back-ends.
///
/// All operations are executed against every registered backend in parallel.
/// For `read` the current strategy is **first backend that returns a value wins**.
/// This is a placeholder for more advanced loser-tree merge.
pub struct MemoryHub {
    backends: Vec<Arc<dyn MemoryBackend>>,
}

impl MemoryHub {
    /// Create an empty hub. Register at least one backend before use.
    pub fn new() -> Self {
        Self { backends: Vec::new() }
    }

    /// Register a backend implementation. Can be called at runtime during init.
    pub fn register_backend(&mut self, backend: Box<dyn MemoryBackend>) {
        self.backends.push(backend.into());
    }

    /// Store the value in **all** back-ends.
    pub async fn write(&self, key: String, value: Vec<u8>) -> HubResult<()> {
        let mut futures_vec = Vec::with_capacity(self.backends.len());
        for be in &self.backends {
            let k = key.clone();
            let v = value.clone();
            let be = Arc::clone(be);
            futures_vec.push(async move { be.write(k, v).await });
        }

        // Wait for all backends to finish.
        // Collect first error (if any).
        let results = join_all(futures_vec).await;
        #[cfg(feature = "dev_metrics")] counter!("memory_hub.write.total", 1);
        for res in results {
            res?;
        }
        #[cfg(feature = "dev_metrics")] histogram!("memory_hub.write.latency_ms", 0.0); // placeholder
        Ok(())
    }

    /// Retrieve value by key from back-ends concurrently.
    /// Returns the first successful hit or `None` if nobody has the key.
    pub async fn read(&self, key: String) -> HubResult<Option<Vec<u8>>> {
        #[cfg(feature = "dev_metrics")] counter!("memory_hub.read.total", 1);
        let mut futures_vec = Vec::with_capacity(self.backends.len());
        for be in &self.backends {
            let k = key.clone();
            let be = Arc::clone(be);
            futures_vec.push(async move { be.read(k).await });
        }

        // Drive all futures concurrently.
        let results = join_all(futures_vec).await;
        // First Some wins.
        for res in results {
            match res? {
                Some(v) => return Ok(Some(v)),
                None => continue,
            }
        }
        Ok(None)
    }
} 