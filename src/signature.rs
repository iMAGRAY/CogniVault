#[cfg(feature = "plugin_verify")]
use ed25519_dalek::{Signature, Verifier, PublicKey};
use std::fs;

/// Verify detached signature file `<artifact>.sig` against given public key bytes.
#[cfg(feature = "plugin_verify")]
pub fn verify(artifact_path: &std::path::Path, public_key_bytes: &[u8;32]) -> anyhow::Result<()> {
    let sig_path = artifact_path.with_extension("sig");
    let sig_bytes = fs::read(&sig_path)
        .map_err(|e| anyhow::anyhow!("Signature file missing: {}", e))?;
    let signature = Signature::from_bytes(&sig_bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid signature length"))?);
    let pk = PublicKey::from_bytes(public_key_bytes)?;
    let data = fs::read(artifact_path)?;
    pk.verify_strict(&data, &signature).map_err(|e| anyhow::anyhow!("Signature check failed: {e}"))
}

#[cfg(not(feature = "plugin_verify"))]
pub fn verify(_artifact_path: &std::path::Path, _pk: &[u8;32]) -> anyhow::Result<()> { Ok(()) } 