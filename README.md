# CogniVault

High-performance, pluggable **Memory Hub** for embeddings, blobs and vector search – written in 100 % Rust.

![CI](https://img.shields.io/badge/build-passing-brightgreen) ![license](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue)

---

## ✨ Features

* **Runtime-agnostic** – choose `async-std` (default) or `tokio`.
* **Pluggable storage** – RAM cache, encrypted Sled, filesystem objects.
* **Vector search** – SIMD-accelerated HNSW or scalar fallback.
* **Signed plugins** – load extra back-ends via `cdylib` or WASI after Ed25519 verification.
* **Observability** – Prometheus metrics, SLO Guard, cancellation & rlimit.
* **Integrity** – append-only Merkle log, snapshot hooks (PAR2 WIP).

## 📦 Quick start

```shell
# RAM-only hub (default)
cargo add cognivault
```

```rust
use cognivault::*;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    // Hub with ShortMem (in-RAM)
    let mut hub = MemoryHub::new();
    hub.register_backend(Box::new(ShortMem::default()));

    hub.write("foo".into(), b"bar".to_vec()).await?;
    assert_eq!(hub.read("foo".into()).await?.unwrap(), b"bar".to_vec());
    Ok(())
}
```

## 🛠️ Feature matrix

| Feature            | Flag                       | Default |
|--------------------|----------------------------|---------|
| `tokio` runtime    | `runtime_tokio`            | ❌      |
| Encrypted Sled     | `longmem_sled longmem_encrypt` | ❌ |
| Filesystem store   | `detailmem_fs`             | ❌      |
| Merkle log         | `merkle_log`               | ❌      |
| HNSW SIMD ANN      | `ann_hnsw`                 | ❌      |
| Scalar ANN         | `ann_scalar`               | ❌      |
| Prometheus metrics | `dev_metrics`              | ❌      |
| cdylib plugins     | `plugin_cdylib`            | ❌      |
| WASI plugins       | `plugin_wasi`              | ❌      |
| Ed25519 verify     | `plugin_verify`            | ❌      |

Enable a combo:
```bash
cargo test --no-default-features \
  --features "runtime_tokio ann_hnsw longmem_sled longmem_encrypt dev_metrics"
```

## 🖇️ Plugin API (cdylib)

Expected C symbol:
```c
// inside your Rust cdylib
#[no_mangle]
pub extern "C" fn create_backend() -> *mut dyn MemoryBackend { /* ... */ }
```
The shared library and its `.sig` must pass Ed25519 verification (optional).

## 🧱 Repository layout

```
src/
  backend.rs      – trait & alias
  hub.rs          – fan-out / merge core
  shortmem.rs     – RAM backend
  longmem.rs      – Sled + AES-GCM-SIV
  detailmem.rs    – filesystem objects
  ann.rs          – ANN engines (HNSW / scalar)
  plugin.rs       – loader for cdylib / WASI
  cancellation.rs – cancel tokens
  limit_guard.rs  – rlimit / JobObject
  sloguard.rs     – concurrency throttle
  observability.rs– Prometheus export
  merkle.rs       – append-only Merkle log
```

## 🔒 License

Dual-licensed under MIT or Apache-2.0, at your choice. 