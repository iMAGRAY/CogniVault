use async_trait::async_trait;
use std::error::Error;

/// Alias for library result type.
pub type HubResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

/// Core abstraction every storage adapter or plugin must implement.
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    /// Persist or update the value associated with the given key.
    async fn write(&self, key: String, value: Vec<u8>) -> HubResult<()>;

    /// Retrieve value by key. `Ok(None)` means key not found.
    async fn read(&self, key: String) -> HubResult<Option<Vec<u8>>>;
} 