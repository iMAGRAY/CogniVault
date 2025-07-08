use anyhow::Result;
use std::path::Path;
use crate::backend::MemoryBackend;

/// Types of plugins supported.
#[derive(Debug, Clone, Copy)]
pub enum PluginKind { Cdylib, Wasi }

/// Loader responsible for bringing signed plugins into the process.
pub struct PluginLoader;

impl PluginLoader {
    /// Load a plugin backend according to its kind.
    pub fn load(path: &Path, kind: PluginKind) -> Result<Box<dyn MemoryBackend>> {
        match kind {
            PluginKind::Cdylib => Self::load_cdylib(path),
            PluginKind::Wasi => Self::load_wasi(path),
        }
    }

    #[cfg(feature = "plugin_cdylib")]
    fn load_cdylib(path: &Path) -> Result<Box<dyn MemoryBackend>> {
        #[cfg(feature="plugin_verify")]
        {
            // example: embedded public key constant; in real system from config
            const PUBKEY: [u8;32] = *include_bytes!("../public_key.bin");
            super::signature::verify(path, &PUBKEY)?;
        }
        unsafe {
            use libloading::{Library, Symbol};
            // Safety: we trust plugin signature verification done elsewhere.
            let lib = Library::new(path)?;
            // Expected symbol: extern "C" fn create_backend() -> *mut dyn MemoryBackend
            type Constructor = unsafe fn() -> *mut dyn MemoryBackend;
            let ctor: Symbol<Constructor> = lib.get(b"create_backend")?;
            let boxed = Box::from_raw(ctor());
            // we must not drop `lib`, otherwise symbols will unload. Leak it.
            std::mem::forget(lib);
            Ok(boxed)
        }
    }
    #[cfg(not(feature = "plugin_cdylib"))]
    fn load_cdylib(_path: &Path) -> Result<Box<dyn MemoryBackend>> {
        Err(anyhow::anyhow!("cdylib plugin support not compiled in"))
    }

    #[cfg(feature = "plugin_wasi")]
    fn load_wasi(path: &Path) -> Result<Box<dyn MemoryBackend>> {
        #[cfg(feature="plugin_verify")]
        {
            const PUBKEY: [u8;32] = *include_bytes!("../public_key.bin");
            super::signature::verify(path, &PUBKEY)?;
        }
        // instantiate engine (simplified)
        use wasmtime::{Engine, Module, Store, Config, Caller, Linker};
        let mut cfg = Config::new();
        cfg.wasm_multi_memory(true);
        let engine = Engine::new(&cfg)?;
        let module = Module::from_file(&engine, path)?;
        let mut store = Store::new(&engine, ());
        let mut linker = Linker::new(&engine);
        // No hostcalls; expecting plugin to call exported functions we ignore.
        let instance = linker.instantiate(&mut store, &module)?;
        let func = instance.get_func(&mut store, "create_backend").ok_or_else(|| anyhow::anyhow!("missing create_backend"))?;
        let typed = func.typed::<(), i32>(&store)?;
        let addr = typed.call(&mut store, ())? as usize;
        // Safety: plugin returns heap pointer to boxed Backend
        let boxed: Box<dyn MemoryBackend> = unsafe { Box::from_raw(addr as *mut dyn MemoryBackend) };
        Ok(boxed)
    }
    #[cfg(not(feature = "plugin_wasi"))]
    fn load_wasi(_path: &Path) -> Result<Box<dyn MemoryBackend>> {
        Err(anyhow::anyhow!("wasi plugin support not compiled in"))
    }
} 