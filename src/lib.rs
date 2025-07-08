// SPDX-License-Identifier: MIT
//! Memory Hub — core routing layer between clients and memory back-ends.
//!
//! High-level architecture (see docs/architecture.md):
//!  • Clients/API call [`MemoryHub`] methods.
//!  • Hub in parallel fan-outs the request to every registered backend that
//!    implements [`MemoryBackend`] trait.
//!  • On reads hub merges results using simple first-win policy for now
//!    (placeholder for loser-tree merge).
//!
//! By default the crate is runtime-agnostic: choose either `runtime_async_std`
//! (default) or `runtime_tokio` feature at compile time.

mod backend;
mod hub;
mod shortmem;
mod cancellation;
pub use cancellation::CancellationToken;
mod limit_guard;
pub use limit_guard::LimitGuard;
mod policy;
pub use policy::{PolicyEngine, AllowAllPolicy};
mod plugin;
pub use plugin::{PluginLoader, PluginKind};
#[cfg(feature = "longmem_sled")] mod longmem;
#[cfg(feature = "detailmem_fs")] mod detailmem;
#[cfg(feature = "detailmem_fs")] pub use detailmem::DetailMem;
#[cfg(feature = "dev_metrics")] mod observability;
#[cfg(feature = "dev_metrics")] pub use observability as obs;
#[cfg(any(feature = "ann_hnsw", feature = "ann_scalar"))] mod ann;
#[cfg(any(feature = "ann_hnsw", feature = "ann_scalar"))] pub use ann::{AnnEngine, AnnDefault};
#[cfg(feature="plugin_verify")] mod signature;
#[cfg(feature="merkle_log")] pub mod merkle;

pub mod sloguard; pub use sloguard::SloGuard;

pub use backend::MemoryBackend;
#[cfg(feature = "longmem_sled")] pub use longmem::LongMem;
pub use hub::MemoryHub;

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn smoke_write_read() {
        let mut hub = MemoryHub::new();
        hub.register_backend(Box::new(crate::shortmem::ShortMem::default()));

        hub.write("foo".into(), b"bar".to_vec()).await.unwrap();
        let res = hub.read("foo".into()).await.unwrap();
        assert_eq!(res.unwrap(), b"bar".to_vec());
    }
}

#[cfg(all(feature="ann_scalar", test))]
mod ann_tests {
    use super::*;

    #[async_std::test]
    async fn ann_scalar_basic() {
        let mut engine = ann::AnnDefault::new(3);
        engine.add_vector(vec![0.0,0.0,0.0]).unwrap();
        engine.add_vector(vec![1.0,0.0,0.0]).unwrap();
        let res = engine.search(&[0.1,0.0,0.0], 1).unwrap();
        assert_eq!(res[0], 0);
    }
}
