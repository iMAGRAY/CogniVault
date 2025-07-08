use async_lock::Semaphore;
use std::sync::Arc;

/// SLO Guard — ограничивает максимальное число одновременных операций,
/// помогает предотвратить перегрузку памяти/CPU.
#[derive(Clone)]
pub struct SloGuard {
    sem: Arc<Semaphore>,
}

impl SloGuard {
    /// Создать Guard с лимитом `max_concurrent`.
    pub fn new(max_concurrent: usize) -> Self {
        Self { sem: Arc::new(Semaphore::new(max_concurrent)) }
    }

    /// Запустить асинхронную функцию, автоматически занимая слот.
    pub async fn run<F, Fut, T>(&self, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let permit = self.sem.acquire().await;
        let res = f().await;
        drop(permit);
        res
    }
} 