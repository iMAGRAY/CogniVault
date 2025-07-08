use crate::backend::{MemoryBackend, HubResult};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;

/// File-system based backend for storing large objects & vectors on demand.
/// Each key maps to a file `<root>/<key>.bin`. Integrity can be checked by
/// computing SHA-256 over content; Merkle-log/PAR2 snapshot reserved for future.
#[derive(Clone)]
pub struct DetailMem {
    root: PathBuf,
}

impl DetailMem {
    pub fn open<P: AsRef<Path>>(root: P) -> HubResult<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn file_path(&self, key: &str) -> PathBuf {
        let mut path = self.root.clone();
        path.push(format!("{}.bin", key));
        path
    }
}

#[async_trait]
impl MemoryBackend for DetailMem {
    async fn write(&self, key: String, value: Vec<u8>) -> HubResult<()> {
        let path = self.file_path(&key);
        // ensure parent dir exists (race-free since create_dir_all ok on exist)
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        // Write atomically: write to tmp then rename
        let tmp_path = path.with_extension("bin.tmp");
        {
            let mut f = File::create(&tmp_path)?;
            f.write_all(&value)?;
            f.sync_all()?;
        }
        fs::rename(&tmp_path, &path)?;

        // compute checksum for Merkle root (placeholder)
        #[cfg(feature="merkle_log")]
        {
            use crate::merkle::MerkleLog;
            let mut log = MerkleLog::open(self.root.join("merkle.log"))?;
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&value);
            log.append(hasher.finalize().into())?;
        }
        Ok(())
    }

    async fn read(&self, key: String) -> HubResult<Option<Vec<u8>>> {
        let path = self.file_path(&key);
        match fs::read(path) {
            Ok(buf) => Ok(Some(buf)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
} 