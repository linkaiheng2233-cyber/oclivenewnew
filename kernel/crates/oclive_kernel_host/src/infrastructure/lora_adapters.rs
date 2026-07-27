//! Managed storage and validation for llama.cpp LoRA GGUF adapters.

use crate::error::{AppError, Result};
use chrono::Utc;
use oclive_kernel_types::models::{
    ImportLocalLoraAdapterRequest, LocalLoraAdapterDto, LoraContentRating,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;
use zip::ZipArchive;

pub const ADAPTERS_DIR_NAME: &str = "adapters";
pub const ADAPTER_MANIFEST_FILE: &str = "adapter.json";
pub const ADAPTER_GGUF_FILE: &str = "adapter.gguf";
pub const ADAPTER_FORMAT: &str = "llama.cpp-lora-gguf";
const ADAPTER_SCHEMA_VERSION: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_ADAPTER_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_GGUF_METADATA_ENTRIES: u64 = 100_000;
const MAX_GGUF_STRING_BYTES: u64 = 1024 * 1024;
const MAX_GGUF_ARRAY_ITEMS: u64 = 2_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdapterManifest {
    schema_version: u32,
    id: String,
    name: String,
    version: String,
    format: String,
    adapter_file: String,
    adapter_sha256: String,
    #[serde(default)]
    base_model: Option<String>,
    #[serde(default)]
    architecture: Option<String>,
    #[serde(default)]
    content_rating: LoraContentRating,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    installed_at: String,
}

/// Resolved adapter metadata plus its verified local GGUF path.
pub struct ResolvedLocalLoraAdapter {
    pub dto: LocalLoraAdapterDto,
    pub gguf_path: PathBuf,
}

#[derive(Debug, Default)]
struct GgufAdapterMetadata {
    general_type: Option<String>,
    adapter_type: Option<String>,
    architecture: Option<String>,
}

struct CleanupDir(Option<PathBuf>);

impl Drop for CleanupDir {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

#[must_use]
pub fn adapters_root(models_dir: &Path) -> PathBuf {
    models_dir.join(ADAPTERS_DIR_NAME)
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::InvalidParameter(message.into())
}

fn ensure_managed_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(AppError::IoError)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(invalid(format!(
            "managed adapter path '{}' is not a real directory",
            path.display()
        )));
    }
    Ok(())
}

fn ensure_managed_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(AppError::IoError)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(invalid(format!(
            "managed adapter path '{}' is not a regular file",
            path.display()
        )));
    }
    Ok(())
}

fn validate_adapter_id(id: &str) -> Result<&str> {
    let id = id.trim();
    let valid = !id.is_empty()
        && id.len() <= 96
        && !id.starts_with('.')
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if valid {
        Ok(id)
    } else {
        Err(invalid(
            "adapter id must contain only ASCII letters, digits, '.', '_' or '-'",
        ))
    }
}

fn validate_package_path(path: &str) -> Result<String> {
    let normalized = path.replace('\\', "/");
    let candidate = Path::new(&normalized);
    let valid = !normalized.is_empty()
        && !candidate.is_absolute()
        && candidate
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        && candidate
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"));
    if valid {
        Ok(normalized)
    } else {
        Err(invalid(
            "adapterFile must be a relative GGUF path without parent traversal",
        ))
    }
}

fn validate_manifest(mut manifest: AdapterManifest) -> Result<AdapterManifest> {
    if manifest.schema_version != ADAPTER_SCHEMA_VERSION {
        return Err(invalid(format!(
            "unsupported .ocadapter schemaVersion {}; expected {ADAPTER_SCHEMA_VERSION}",
            manifest.schema_version
        )));
    }
    manifest.id = validate_adapter_id(&manifest.id)?.to_string();
    manifest.name = manifest.name.trim().to_string();
    manifest.version = manifest.version.trim().to_string();
    manifest.format = manifest.format.trim().to_string();
    manifest.adapter_file = validate_package_path(&manifest.adapter_file)?;
    manifest.adapter_sha256 = manifest.adapter_sha256.trim().to_ascii_lowercase();
    if manifest.name.is_empty() || manifest.name.chars().count() > 160 {
        return Err(invalid("adapter name must contain 1 to 160 characters"));
    }
    if manifest.version.is_empty() || manifest.version.chars().count() > 64 {
        return Err(invalid("adapter version must contain 1 to 64 characters"));
    }
    if manifest.format != ADAPTER_FORMAT {
        return Err(invalid(format!(
            "unsupported adapter format '{}'; expected {ADAPTER_FORMAT}",
            manifest.format
        )));
    }
    if manifest.adapter_sha256.len() != 64
        || !manifest
            .adapter_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(invalid("adapterSha256 must be a 64-character SHA-256"));
    }
    Ok(manifest)
}

fn normalize_optional(value: Option<String>, max_chars: usize) -> Option<String> {
    value.and_then(|value| {
        let trimmed: String = value.trim().chars().take(max_chars).collect();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn file_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(AppError::IoError)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(AppError::IoError)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn copy_with_sha256(
    reader: &mut impl Read,
    destination: &Path,
    max_bytes: u64,
) -> Result<(u64, String)> {
    let mut output = File::create(destination).map_err(AppError::IoError)?;
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(AppError::IoError)?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or_else(|| invalid("adapter size overflow"))?;
        if total > max_bytes {
            return Err(invalid("adapter exceeds the 16 GiB managed import limit"));
        }
        output
            .write_all(&buffer[..read])
            .map_err(AppError::IoError)?;
        hasher.update(&buffer[..read]);
    }
    output.sync_all().map_err(AppError::IoError)?;
    Ok((total, format!("{:x}", hasher.finalize())))
}

fn read_u32(reader: &mut impl Read) -> Result<u32> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes).map_err(AppError::IoError)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> Result<u64> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes).map_err(AppError::IoError)?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_gguf_string(reader: &mut (impl Read + Seek), file_len: u64) -> Result<String> {
    let len = read_u64(reader)?;
    if len > MAX_GGUF_STRING_BYTES {
        return Err(invalid("GGUF metadata string exceeds the safe limit"));
    }
    ensure_remaining(reader, file_len, len)?;
    let mut bytes = vec![0_u8; len as usize];
    reader.read_exact(&mut bytes).map_err(AppError::IoError)?;
    String::from_utf8(bytes).map_err(|_| invalid("GGUF metadata contains invalid UTF-8"))
}

fn ensure_remaining(reader: &mut impl Seek, file_len: u64, bytes: u64) -> Result<()> {
    let position = reader.stream_position().map_err(AppError::IoError)?;
    if position.checked_add(bytes).is_none_or(|end| end > file_len) {
        return Err(invalid("GGUF metadata is truncated"));
    }
    Ok(())
}

fn seek_forward(reader: &mut impl Seek, file_len: u64, bytes: u64) -> Result<()> {
    ensure_remaining(reader, file_len, bytes)?;
    let offset = i64::try_from(bytes).map_err(|_| invalid("GGUF metadata offset overflow"))?;
    reader
        .seek(SeekFrom::Current(offset))
        .map_err(AppError::IoError)?;
    Ok(())
}

fn skip_gguf_value(
    reader: &mut (impl Read + Seek),
    file_len: u64,
    value_type: u32,
    depth: usize,
) -> Result<()> {
    if depth > 2 {
        return Err(invalid("GGUF metadata array nesting is unsupported"));
    }
    match value_type {
        0 | 1 | 7 => seek_forward(reader, file_len, 1),
        2 | 3 => seek_forward(reader, file_len, 2),
        4..=6 => seek_forward(reader, file_len, 4),
        8 => {
            let _ = read_gguf_string(reader, file_len)?;
            Ok(())
        }
        9 => {
            let element_type = read_u32(reader)?;
            let count = read_u64(reader)?;
            if count > MAX_GGUF_ARRAY_ITEMS {
                return Err(invalid("GGUF metadata array exceeds the safe limit"));
            }
            let fixed_size = match element_type {
                0 | 1 | 7 => Some(1_u64),
                2 | 3 => Some(2_u64),
                4..=6 => Some(4_u64),
                10..=12 => Some(8_u64),
                _ => None,
            };
            if let Some(item_size) = fixed_size {
                return seek_forward(
                    reader,
                    file_len,
                    count
                        .checked_mul(item_size)
                        .ok_or_else(|| invalid("GGUF metadata array size overflow"))?,
                );
            }
            for _ in 0..count {
                skip_gguf_value(reader, file_len, element_type, depth + 1)?;
            }
            Ok(())
        }
        10..=12 => seek_forward(reader, file_len, 8),
        _ => Err(invalid(format!(
            "unsupported GGUF metadata value type {value_type}"
        ))),
    }
}

fn inspect_gguf_metadata(path: &Path) -> Result<GgufAdapterMetadata> {
    let mut file = File::open(path).map_err(AppError::IoError)?;
    let file_len = file.metadata().map_err(AppError::IoError)?.len();
    if file_len > MAX_ADAPTER_BYTES {
        return Err(invalid("adapter exceeds the 16 GiB managed import limit"));
    }
    let mut magic = [0_u8; 4];
    file.read_exact(&mut magic).map_err(AppError::IoError)?;
    if magic != *b"GGUF" {
        return Err(invalid("selected file is not a GGUF file"));
    }
    let version = read_u32(&mut file)?;
    if !(2..=3).contains(&version) {
        return Err(invalid(format!(
            "unsupported GGUF version {version}; expected version 2 or 3"
        )));
    }
    let _tensor_count = read_u64(&mut file)?;
    let metadata_count = read_u64(&mut file)?;
    if metadata_count > MAX_GGUF_METADATA_ENTRIES {
        return Err(invalid("GGUF metadata entry count exceeds the safe limit"));
    }

    let mut metadata = GgufAdapterMetadata::default();
    for _ in 0..metadata_count {
        let key = read_gguf_string(&mut file, file_len)?;
        let value_type = read_u32(&mut file)?;
        let capture = matches!(
            key.as_str(),
            "general.type" | "adapter.type" | "general.architecture"
        );
        if capture {
            if value_type != 8 {
                return Err(invalid(format!("GGUF metadata '{key}' must be a string")));
            }
            let value = read_gguf_string(&mut file, file_len)?;
            match key.as_str() {
                "general.type" => metadata.general_type = Some(value),
                "adapter.type" => metadata.adapter_type = Some(value),
                "general.architecture" => metadata.architecture = Some(value),
                _ => {}
            }
        } else {
            skip_gguf_value(&mut file, file_len, value_type, 0)?;
        }
    }

    Ok(metadata)
}

fn inspect_gguf_adapter(path: &Path) -> Result<GgufAdapterMetadata> {
    let metadata = inspect_gguf_metadata(path)?;
    if metadata.general_type.as_deref() != Some("adapter")
        || metadata.adapter_type.as_deref() != Some("lora")
    {
        return Err(invalid(
            "GGUF is not a llama.cpp LoRA adapter (expected general.type=adapter and adapter.type=lora)",
        ));
    }
    Ok(metadata)
}

/// Read the `general.architecture` value from a GGUF base model.
///
/// # Errors
///
/// Returns an error when the file is not valid GGUF metadata or is itself an adapter.
pub fn gguf_base_model_architecture(path: &Path) -> Result<Option<String>> {
    let metadata = inspect_gguf_metadata(path)?;
    if metadata.general_type.as_deref() == Some("adapter") {
        return Err(invalid(
            "selected LoRA base model is itself an adapter GGUF",
        ));
    }
    Ok(metadata.architecture)
}

fn manifest_to_dto(
    manifest: &AdapterManifest,
    adapter_path: &Path,
    active_id: Option<&str>,
) -> Result<LocalLoraAdapterDto> {
    let size_bytes = adapter_path.metadata().map_err(AppError::IoError)?.len();
    Ok(LocalLoraAdapterDto {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        format: manifest.format.clone(),
        content_rating: manifest.content_rating,
        file_name: ADAPTER_GGUF_FILE.to_string(),
        size_bytes,
        sha256: manifest.adapter_sha256.clone(),
        base_model: manifest.base_model.clone(),
        architecture: manifest.architecture.clone(),
        description: manifest.description.clone(),
        license: manifest.license.clone(),
        source: manifest.source.clone(),
        installed_at: manifest.installed_at.clone(),
        active: active_id.is_some_and(|active| active == manifest.id),
    })
}

fn read_installed_manifest(adapter_dir: &Path) -> Result<AdapterManifest> {
    ensure_managed_directory(adapter_dir)?;
    let manifest_path = adapter_dir.join(ADAPTER_MANIFEST_FILE);
    ensure_managed_file(&manifest_path)?;
    let bytes = fs::read(manifest_path).map_err(AppError::IoError)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(invalid("installed adapter manifest exceeds the safe limit"));
    }
    let manifest: AdapterManifest = serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("invalid installed adapter manifest: {error}")))?;
    validate_manifest(manifest)
}

fn write_manifest(staging: &Path, manifest: &AdapterManifest) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| invalid(format!("serialize adapter manifest: {error}")))?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(invalid("adapter manifest exceeds the safe limit"));
    }
    let path = staging.join(ADAPTER_MANIFEST_FILE);
    let mut file = File::create(path).map_err(AppError::IoError)?;
    file.write_all(&bytes).map_err(AppError::IoError)?;
    file.sync_all().map_err(AppError::IoError)
}

fn commit_staging(
    staging_guard: &mut CleanupDir,
    root: &Path,
    id: &str,
    replace_existing: bool,
) -> Result<PathBuf> {
    let staging = staging_guard
        .0
        .as_ref()
        .ok_or_else(|| invalid("adapter staging directory is unavailable"))?;
    let destination = root.join(id);
    if destination.exists() && !replace_existing {
        return Err(invalid(format!(
            "adapter '{id}' is already installed; enable replaceExisting to replace it"
        )));
    }

    let backup = root.join(format!(".backup-{id}-{}", Uuid::new_v4()));
    if destination.exists() {
        ensure_managed_directory(&destination)?;
        fs::rename(&destination, &backup).map_err(AppError::IoError)?;
    }
    if let Err(error) = fs::rename(staging, &destination) {
        if backup.exists() {
            let _ = fs::rename(&backup, &destination);
        }
        return Err(AppError::IoError(error));
    }
    staging_guard.0 = None;
    if backup.exists() {
        fs::remove_dir_all(backup).map_err(AppError::IoError)?;
    }
    Ok(destination)
}

fn package_manifest(archive: &mut ZipArchive<File>) -> Result<AdapterManifest> {
    let mut manifest_indices = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|error| invalid(format!("read .ocadapter entry: {error}")))?;
        if entry.name().replace('\\', "/") == ADAPTER_MANIFEST_FILE {
            manifest_indices.push(index);
        }
    }
    if manifest_indices.len() != 1 {
        return Err(invalid(
            ".ocadapter must contain exactly one root adapter.json",
        ));
    }
    let entry = archive
        .by_index(manifest_indices[0])
        .map_err(|error| invalid(format!("read adapter.json: {error}")))?;
    if entry.size() > MAX_MANIFEST_BYTES {
        return Err(invalid("adapter.json exceeds the 1 MiB safe limit"));
    }
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(AppError::IoError)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(invalid("adapter.json exceeds the 1 MiB safe limit"));
    }
    let manifest: AdapterManifest = serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("invalid adapter.json: {error}")))?;
    validate_manifest(manifest)
}

fn raw_manifest(
    request: &ImportLocalLoraAdapterRequest,
    source: &Path,
    sha256: String,
    metadata: &GgufAdapterMetadata,
) -> Result<AdapterManifest> {
    let fallback_name = source
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Local LoRA");
    let name = request
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback_name);
    let id = format!("local.lora.{}", &sha256[..16]);
    validate_manifest(AdapterManifest {
        schema_version: ADAPTER_SCHEMA_VERSION,
        id,
        name: name.to_string(),
        version: "0.0.0-local".to_string(),
        format: ADAPTER_FORMAT.to_string(),
        adapter_file: ADAPTER_GGUF_FILE.to_string(),
        adapter_sha256: sha256,
        base_model: normalize_optional(request.base_model.clone(), 256),
        architecture: metadata.architecture.clone(),
        content_rating: request.content_rating,
        description: None,
        license: None,
        source: None,
        installed_at: Utc::now().to_rfc3339(),
    })
}

/// Import a raw LoRA GGUF or `.ocadapter` into `<models>/adapters/<id>`.
///
/// # Errors
///
/// Returns an error for unsafe packages, invalid GGUF metadata, checksum
/// mismatches, I/O failures, or an existing id without explicit replacement.
pub fn import_local_lora_adapter(
    models_dir: &Path,
    request: &ImportLocalLoraAdapterRequest,
    protected_active_id: Option<&str>,
) -> Result<LocalLoraAdapterDto> {
    let source = PathBuf::from(request.source_path.trim());
    if !source.is_file() {
        return Err(invalid("LoRA adapter source file does not exist"));
    }
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    if !extension.eq_ignore_ascii_case("gguf") && !extension.eq_ignore_ascii_case("ocadapter") {
        return Err(invalid(
            "LoRA import accepts only llama.cpp adapter .gguf or .ocadapter files",
        ));
    }

    let root = adapters_root(models_dir);
    fs::create_dir_all(&root).map_err(AppError::IoError)?;
    ensure_managed_directory(&root)?;
    let staging = root.join(format!(".import-{}", Uuid::new_v4()));
    fs::create_dir(&staging).map_err(AppError::IoError)?;
    let mut staging_guard = CleanupDir(Some(staging.clone()));
    let target = staging.join(ADAPTER_GGUF_FILE);

    let manifest = if extension.eq_ignore_ascii_case("gguf") {
        let mut input = File::open(&source).map_err(AppError::IoError)?;
        let (_, sha256) = copy_with_sha256(&mut input, &target, MAX_ADAPTER_BYTES)?;
        let metadata = inspect_gguf_adapter(&target)?;
        raw_manifest(request, &source, sha256, &metadata)?
    } else {
        let package = File::open(&source).map_err(AppError::IoError)?;
        let mut archive = ZipArchive::new(package)
            .map_err(|error| invalid(format!("invalid .ocadapter ZIP: {error}")))?;
        let mut manifest = package_manifest(&mut archive)?;
        let adapter_file = manifest.adapter_file.clone();
        let mut adapter_entries = 0_usize;
        for index in 0..archive.len() {
            let entry = archive
                .by_index(index)
                .map_err(|error| invalid(format!("read .ocadapter entry: {error}")))?;
            if entry.name().replace('\\', "/") == adapter_file {
                adapter_entries += 1;
            }
        }
        if adapter_entries != 1 {
            return Err(invalid(format!(
                ".ocadapter must contain exactly one '{adapter_file}' entry"
            )));
        }
        let mut entry = archive.by_name(&adapter_file).map_err(|_| {
            invalid(format!(
                "adapterFile '{adapter_file}' is missing from package"
            ))
        })?;
        if entry.is_dir() || entry.size() > MAX_ADAPTER_BYTES {
            return Err(invalid(
                "packaged adapter is not a valid file or is too large",
            ));
        }
        let (_, sha256) = copy_with_sha256(&mut entry, &target, MAX_ADAPTER_BYTES)?;
        if sha256 != manifest.adapter_sha256 {
            return Err(invalid(
                "packaged adapter SHA-256 does not match adapter.json",
            ));
        }
        let metadata = inspect_gguf_adapter(&target)?;
        if let (Some(expected), Some(actual)) = (
            manifest.architecture.as_deref(),
            metadata.architecture.as_deref(),
        ) {
            if !expected.eq_ignore_ascii_case(actual) {
                return Err(invalid(format!(
                    "adapter architecture mismatch: manifest '{expected}', GGUF '{actual}'"
                )));
            }
        }
        manifest.adapter_file = ADAPTER_GGUF_FILE.to_string();
        manifest.architecture = manifest.architecture.or(metadata.architecture);
        manifest.installed_at = Utc::now().to_rfc3339();
        manifest
    };

    if protected_active_id.is_some_and(|active_id| active_id.trim() == manifest.id) {
        return Err(invalid(
            "deactivate the LoRA adapter before replacing its managed files",
        ));
    }
    write_manifest(&staging, &manifest)?;
    let destination = commit_staging(
        &mut staging_guard,
        &root,
        &manifest.id,
        request.replace_existing,
    )?;
    manifest_to_dto(&manifest, &destination.join(ADAPTER_GGUF_FILE), None)
}

/// Resolve and fully revalidate an installed adapter.
///
/// # Errors
///
/// Returns an error when the id, manifest, GGUF metadata, or checksum is invalid.
pub fn resolve_local_lora_adapter(
    models_dir: &Path,
    adapter_id: &str,
    active_id: Option<&str>,
) -> Result<ResolvedLocalLoraAdapter> {
    let id = validate_adapter_id(adapter_id)?;
    let root = adapters_root(models_dir);
    if !root.is_dir() {
        return Err(invalid(format!("LoRA adapter '{id}' is not installed")));
    }
    ensure_managed_directory(&root)?;
    let adapter_dir = root.join(id);
    if !adapter_dir.is_dir() {
        return Err(invalid(format!("LoRA adapter '{id}' is not installed")));
    }
    ensure_managed_directory(&adapter_dir)?;
    let manifest = read_installed_manifest(&adapter_dir)?;
    if manifest.id != id {
        return Err(invalid(
            "installed adapter directory id does not match manifest id",
        ));
    }
    let gguf_path = adapter_dir.join(ADAPTER_GGUF_FILE);
    ensure_managed_file(&gguf_path)?;
    let _ = inspect_gguf_adapter(&gguf_path)?;
    let sha256 = file_sha256(&gguf_path)?;
    if sha256 != manifest.adapter_sha256 {
        return Err(invalid(format!(
            "installed LoRA adapter '{id}' failed checksum verification"
        )));
    }
    let dto = manifest_to_dto(&manifest, &gguf_path, active_id)?;
    Ok(ResolvedLocalLoraAdapter { dto, gguf_path })
}

#[must_use]
pub fn list_local_lora_adapters(
    models_dir: &Path,
    active_id: Option<&str>,
) -> Vec<LocalLoraAdapterDto> {
    let root = adapters_root(models_dir);
    if ensure_managed_directory(&root).is_err() {
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut adapters = entries
        .flatten()
        .filter_map(|entry| {
            let id = entry.file_name().to_string_lossy().into_owned();
            if id.starts_with('.') || !entry.path().is_dir() {
                return None;
            }
            let adapter_dir = entry.path();
            let listed = (|| {
                ensure_managed_directory(&adapter_dir)?;
                let manifest = read_installed_manifest(&adapter_dir)?;
                if manifest.id != id {
                    return Err(invalid(
                        "installed adapter directory id does not match manifest id",
                    ));
                }
                let gguf_path = adapter_dir.join(ADAPTER_GGUF_FILE);
                ensure_managed_file(&gguf_path)?;
                let metadata = inspect_gguf_adapter(&gguf_path)?;
                if let (Some(expected), Some(actual)) = (
                    manifest.architecture.as_deref(),
                    metadata.architecture.as_deref(),
                ) {
                    if !expected.eq_ignore_ascii_case(actual) {
                        return Err(invalid("installed adapter architecture metadata mismatch"));
                    }
                }
                manifest_to_dto(&manifest, &gguf_path, active_id)
            })();
            match listed {
                Ok(dto) => Some(dto),
                Err(error) => {
                    tracing::warn!(
                        target: "oclive_lora",
                        adapter_id = %id,
                        %error,
                        "ignored invalid installed LoRA adapter"
                    );
                    None
                }
            }
        })
        .collect::<Vec<_>>();
    adapters.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    adapters
}

/// Remove one inactive managed adapter directory.
///
/// # Errors
///
/// Returns an error for an invalid/missing id or filesystem failure.
pub fn delete_local_lora_adapter(models_dir: &Path, adapter_id: &str) -> Result<()> {
    let id = validate_adapter_id(adapter_id)?;
    let root = adapters_root(models_dir);
    if !root.is_dir() {
        return Err(invalid(format!("LoRA adapter '{id}' is not installed")));
    }
    ensure_managed_directory(&root)?;
    let adapter_dir = root.join(id);
    if !adapter_dir.is_dir() {
        return Err(invalid(format!("LoRA adapter '{id}' is not installed")));
    }
    ensure_managed_directory(&adapter_dir)?;
    let manifest = read_installed_manifest(&adapter_dir)?;
    if manifest.id != id {
        return Err(invalid(
            "installed adapter directory id does not match manifest id",
        ));
    }
    fs::remove_dir_all(adapter_dir).map_err(AppError::IoError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::write::SimpleFileOptions;

    fn push_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }

    fn write_minimal_gguf(path: &Path, general_type: &str) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GGUF");
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.extend_from_slice(&3_u64.to_le_bytes());
        for (key, value) in [
            ("general.type", general_type),
            ("adapter.type", "lora"),
            ("general.architecture", "llama"),
        ] {
            push_string(&mut bytes, key);
            bytes.extend_from_slice(&8_u32.to_le_bytes());
            push_string(&mut bytes, value);
        }
        fs::write(path, bytes).expect("write fixture");
    }

    fn request(source: &Path) -> ImportLocalLoraAdapterRequest {
        ImportLocalLoraAdapterRequest {
            source_path: source.to_string_lossy().into_owned(),
            name: Some("Fixture adapter".to_string()),
            base_model: Some("fixture-base.gguf".to_string()),
            content_rating: LoraContentRating::General,
            replace_existing: false,
        }
    }

    fn write_package(path: &Path, gguf_path: &Path, adapter_file: &str, sha256: &str) {
        let manifest = AdapterManifest {
            schema_version: ADAPTER_SCHEMA_VERSION,
            id: "fixture.package".to_string(),
            name: "Packaged fixture".to_string(),
            version: "1.0.0".to_string(),
            format: ADAPTER_FORMAT.to_string(),
            adapter_file: adapter_file.to_string(),
            adapter_sha256: sha256.to_string(),
            base_model: Some("fixture-base".to_string()),
            architecture: Some("llama".to_string()),
            content_rating: LoraContentRating::Adult,
            description: Some("fixture".to_string()),
            license: Some("test-only".to_string()),
            source: None,
            installed_at: String::new(),
        };
        let file = File::create(path).expect("create package");
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file(ADAPTER_MANIFEST_FILE, SimpleFileOptions::default())
            .expect("manifest entry");
        writer
            .write_all(&serde_json::to_vec(&manifest).expect("manifest JSON"))
            .expect("write manifest");
        if !adapter_file.contains("..") {
            writer
                .start_file(adapter_file, SimpleFileOptions::default())
                .expect("adapter entry");
            writer
                .write_all(&fs::read(gguf_path).expect("read GGUF"))
                .expect("write GGUF");
        }
        writer.finish().expect("finish package");
    }

    #[test]
    fn raw_gguf_import_is_verified_and_listed() {
        let temp = tempfile::tempdir().expect("temp");
        let source = temp.path().join("fixture.gguf");
        write_minimal_gguf(&source, "adapter");

        let imported = import_local_lora_adapter(temp.path(), &request(&source), None)
            .expect("import adapter");
        assert_eq!(imported.name, "Fixture adapter");
        assert_eq!(imported.architecture.as_deref(), Some("llama"));
        assert!(!imported.active);

        let listed = list_local_lora_adapters(temp.path(), Some(&imported.id));
        assert_eq!(listed.len(), 1);
        assert!(listed[0].active);
        assert_eq!(listed[0].sha256, imported.sha256);
    }

    #[test]
    fn base_model_gguf_is_rejected() {
        let temp = tempfile::tempdir().expect("temp");
        let source = temp.path().join("model.gguf");
        write_minimal_gguf(&source, "model");

        let error = import_local_lora_adapter(temp.path(), &request(&source), None)
            .expect_err("base model must be rejected");
        assert!(error.to_string().contains("not a llama.cpp LoRA adapter"));
    }

    #[test]
    fn replace_requires_explicit_flag_and_delete_is_scoped() {
        let temp = tempfile::tempdir().expect("temp");
        let source = temp.path().join("fixture.gguf");
        write_minimal_gguf(&source, "adapter");
        let first =
            import_local_lora_adapter(temp.path(), &request(&source), None).expect("first import");
        let error = import_local_lora_adapter(temp.path(), &request(&source), None)
            .expect_err("replace must be explicit");
        assert!(error.to_string().contains("already installed"));

        let mut replacement = request(&source);
        replacement.replace_existing = true;
        let error = import_local_lora_adapter(temp.path(), &replacement, Some(&first.id))
            .expect_err("active adapter files must be protected");
        assert!(error.to_string().contains("deactivate"));
        let replaced =
            import_local_lora_adapter(temp.path(), &replacement, None).expect("replace adapter");
        assert_eq!(replaced.id, first.id);

        delete_local_lora_adapter(temp.path(), &first.id).expect("delete adapter");
        assert!(list_local_lora_adapters(temp.path(), None).is_empty());
    }

    #[test]
    fn ocadapter_import_checks_manifest_and_checksum() {
        let temp = tempfile::tempdir().expect("temp");
        let gguf = temp.path().join("fixture.gguf");
        write_minimal_gguf(&gguf, "adapter");
        let sha256 = file_sha256(&gguf).expect("hash");
        let package = temp.path().join("fixture.ocadapter");
        write_package(&package, &gguf, "weights/adapter.gguf", &sha256);

        let imported = import_local_lora_adapter(temp.path(), &request(&package), None)
            .expect("package import");
        assert_eq!(imported.id, "fixture.package");
        assert_eq!(imported.content_rating, LoraContentRating::Adult);
        assert_eq!(imported.sha256, sha256);

        let bad_package = temp.path().join("bad.ocadapter");
        write_package(&bad_package, &gguf, "weights/adapter.gguf", &"0".repeat(64));
        let error = import_local_lora_adapter(temp.path(), &request(&bad_package), None)
            .expect_err("checksum mismatch must fail");
        assert!(error.to_string().contains("SHA-256"));
    }

    #[test]
    fn ocadapter_rejects_parent_traversal() {
        let temp = tempfile::tempdir().expect("temp");
        let gguf = temp.path().join("fixture.gguf");
        write_minimal_gguf(&gguf, "adapter");
        let package = temp.path().join("traversal.ocadapter");
        write_package(
            &package,
            &gguf,
            "../adapter.gguf",
            &file_sha256(&gguf).expect("hash"),
        );

        let error = import_local_lora_adapter(temp.path(), &request(&package), None)
            .expect_err("parent traversal must fail");
        assert!(error.to_string().contains("without parent traversal"));
    }
}
