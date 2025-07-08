use sha2::{Sha256, Digest};
use anyhow::Result;
use std::fs::{OpenOptions, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;

/// Append-only Merkle log: каждый блок = sha256(data).
/// Файл хранит последовательность хэшей (32B). Корневой хэш можно
/// вычислить свёрткой сверху вниз (pair-wise).
pub struct MerkleLog {
    file: File,
}

impl MerkleLog {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new().create(true).read(true).append(true).open(path)?;
        Ok(Self { file })
    }

    pub fn append(&mut self, leaf_hash: [u8;32]) -> Result<()> {
        self.file.write_all(&leaf_hash)?;
        self.file.sync_data()?;
        Ok(())
    }

    /// Вычислить Merkle-root, читая все листья.
    pub fn root(&mut self) -> Result<[u8;32]> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut leaves = Vec::<[u8;32]>::new();
        let mut buf = [0u8;32];
        while self.file.read_exact(&mut buf).is_ok() {
            leaves.push(buf);
        }
        if leaves.is_empty() { return Ok([0u8;32]); }
        let mut level = leaves;
        while level.len() > 1 {
            let mut next = Vec::with_capacity((level.len()+1)/2);
            for chunk in level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len()==2 { hasher.update(chunk[1]); }
                let h = hasher.finalize();
                next.push(h.into());
            }
            level = next;
        }
        Ok(level[0])
    }
} 