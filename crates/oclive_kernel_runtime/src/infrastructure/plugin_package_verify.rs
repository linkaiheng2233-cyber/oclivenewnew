//! 离线插件包 Ed25519 验签（相对索引条目的 `public_keys`）。

use crate::error::{AppError, Result};
use crate::models::plugin_market_index::PluginIndexEntry;
use base64::{engine::general_purpose::STANDARD as B64_STANDARD, Engine};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginPackageSignatureFile {
    pub plugin_id: String,
    pub pubkey_id: String,
    pub algorithm: String,
    pub signature: String,
    #[serde(default)]
    pub signed_at: Option<String>,
    #[serde(default)]
    pub covers: Option<String>,
}

fn parse_ed25519_pubkey_base64(s: &str) -> Result<VerifyingKey> {
    let bytes = B64_STANDARD
        .decode(s.trim())
        .map_err(|e| AppError::InvalidParameter(format!("invalid base64 public_key: {}", e)))?;
    let arr: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| AppError::InvalidParameter("ed25519 public_key must be 32 bytes".into()))?;
    VerifyingKey::from_bytes(&arr)
        .map_err(|e| AppError::InvalidParameter(format!("invalid ed25519 public_key: {}", e)))
}

fn verify_plugin_package_signature(
    index_entry: &PluginIndexEntry,
    sig: &PluginPackageSignatureFile,
    archive_bytes: &[u8],
) -> Result<()> {
    if sig.plugin_id.trim() != index_entry.id.trim() {
        return Err(AppError::InvalidParameter(format!(
            "[PLUGIN_SIGNATURE_ID_MISMATCH] signature plugin_id mismatch: sig={} index={}",
            sig.plugin_id, index_entry.id
        )));
    }
    if sig.algorithm.trim().to_lowercase() != "ed25519" {
        return Err(AppError::InvalidParameter(format!(
            "[PLUGIN_SIGNATURE_ALGO_UNSUPPORTED] unsupported signature algorithm: {}",
            sig.algorithm
        )));
    }
    let pk = index_entry
        .public_keys
        .iter()
        .find(|k| k.pubkey_id.trim() == sig.pubkey_id.trim())
        .ok_or_else(|| {
            AppError::InvalidParameter(format!(
                "[PLUGIN_PUBKEY_NOT_FOUND] pubkey_id not found in index: {}",
                sig.pubkey_id
            ))
        })?;
    if matches!(pk.status.as_deref(), Some("revoked")) {
        return Err(AppError::InvalidParameter(format!(
            "[PLUGIN_PUBKEY_REVOKED] public key revoked: {}",
            pk.pubkey_id
        )));
    }
    let vk = parse_ed25519_pubkey_base64(&pk.public_key)?;
    let sig_bytes = B64_STANDARD.decode(sig.signature.trim()).map_err(|e| {
        AppError::InvalidParameter(format!(
            "[PLUGIN_SIGNATURE_BASE64_INVALID] invalid base64 signature: {}",
            e
        ))
    })?;
    let sig_arr: [u8; 64] = sig_bytes.as_slice().try_into().map_err(|_| {
        AppError::InvalidParameter(
            "[PLUGIN_SIGNATURE_SIZE_INVALID] ed25519 signature must be 64 bytes".into(),
        )
    })?;
    let signature = Signature::from_bytes(&sig_arr);
    vk.verify_strict(archive_bytes, &signature).map_err(|e| {
        AppError::InvalidParameter(format!(
            "[PLUGIN_SIGNATURE_VERIFY_FAILED] signature verify failed: {}",
            e
        ))
    })?;
    Ok(())
}

pub fn verify_plugin_package_signature_text(
    index_entry: &PluginIndexEntry,
    sig_text: &str,
    archive_bytes: &[u8],
) -> Result<()> {
    let sig: PluginPackageSignatureFile =
        serde_json::from_str(sig_text).map_err(AppError::SerializationError)?;
    verify_plugin_package_signature(index_entry, &sig, archive_bytes)
}
