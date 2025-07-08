use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use futures::future::Future;

/// Lightweight runtime-agnostic cancellation token.
#[derive(Clone, Debug)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Create a new uncancelled token.
    pub fn new() -> Self { Self { cancelled: Arc::new(AtomicBool::new(false)) } }

    /// Trigger cancellation. Idempotent.
    pub fn cancel(&self) { self.cancelled.store(true, Ordering::SeqCst); }

    /// Check synchronously whether the token is cancelled.
    pub fn is_cancelled(&self) -> bool { self.cancelled.load(Ordering::SeqCst) }

    /// Return a future that resolves when the token is cancelled.
    pub fn cancelled(&self) -> impl Future<Output = ()> + '_ {
        futures::future::poll_fn(move |cx| {
            if self.is_cancelled() {
                return std::task::Poll::Ready(());
            }
            // Re-register waker until cancel flag toggles.
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        })
    }
} 