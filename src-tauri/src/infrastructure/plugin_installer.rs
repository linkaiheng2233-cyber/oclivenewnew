use crate::error::AppError;
use crate::infrastructure::directory_plugins::{parse_manifest_version, OclivePluginManifest};
use crate::infrastructure::plugin_state::PluginStateStore;
use crate::state::AppState;
use base64::engine::general_purpose::STANDARD as B64_STANDARD;
use base64::Engine;
use ed25519_dalek::{Signature, VerifyingKey};
use oclive_validation::validate_plugin_market_index_v1;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub use oclive_kernel_runtime::infrastructure::plugin_archive::{
    extract_oclive_plugin_archive, peek_plugin_id_from_archive_bytes,
};

pub type PluginInstallMeta = crate::models::dto::PluginInstallMetaDto;

// NOTE: 权限 token 映射与种子逻辑已迁移到 API 层：
// - 市场安装：只写入用户 consent 的权限子集
// - 开发者模式侧载：由 extract_plugin_zip 等命令按 manifest 种子写入

pub fn update_install_meta_permissions(
    state: &AppState,
    plugin_id: &str,
    declared_permissions: Vec<String>,
    granted_permissions: Vec<String>,
) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Ok(());
    }
    let root = plugins_dir(state).join(pid);
    let Some(mut meta) = read_install_meta(&root) else {
        return Ok(());
    };
    let mut declared = declared_permissions
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    declared.sort();
    declared.dedup();
    let mut granted = granted_permissions
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    granted.sort();
    granted.dedup();
    meta.declared_permissions = declared;
    meta.granted_permissions = granted;
    write_install_meta(&root, &meta)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublisherPublicKey {
    pub pubkey_id: String,
    /// base64 编码的 Ed25519 public key（32 bytes）
    pub public_key: String,
    /// active|revoked|rotated（由索引侧约定）
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub rotated_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexVersionEntry {
    pub version: String,
    #[serde(default)]
    pub download_url: Option<String>,
    #[serde(default)]
    pub signature_url: Option<String>,
    /// git tag；省略时默认使用 `version`
    #[serde(default)]
    pub git_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexEntry {
    /// 条目类型：`plugin`（默认）| `module`（无代码，依赖+配置预设）| `profile`（保留）
    #[serde(rename = "type", default = "default_index_entry_type")]
    pub entry_type: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub version: String,
    /// 仅 `type=plugin` 必填；`module`/`profile` 可为空字符串。
    pub git: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// 发布者 id（官方索引登记公钥主体）
    #[serde(default)]
    pub publisher: Option<String>,
    /// 发布者公钥环（用于验签）
    #[serde(default)]
    pub public_keys: Vec<PublisherPublicKey>,
    /// 多版本索引（用于回滚/离线包下载）
    #[serde(default)]
    pub versions: Vec<PluginIndexVersionEntry>,

    /// `type=module` 时可选：模块声明（无代码）。
    #[serde(default)]
    pub module: Option<PluginIndexModuleSpec>,

    /// `type=profile` 时可选：profile 声明（无代码）。
    #[serde(default)]
    pub profile: Option<PluginIndexProfileSpec>,
}

fn default_index_entry_type() -> String {
    "plugin".to_string()
}

/// `type=module` 的声明体（无代码）；用于“像 meta package 一样”应用一组依赖与配置预设。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexModuleSpec {
    /// 该模块依赖的插件清单（这些才是有代码的内容）。
    #[serde(default)]
    pub plugins: Vec<PluginIndexModulePluginSpec>,
    /// 可选：后端模块预设（写入会话级后端覆盖）。
    #[serde(default)]
    pub backends: Option<crate::models::plugin_backends::PluginBackendsOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexModulePluginSpec {
    pub id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

/// `type=profile` 的声明体（无代码）。v1 允许“市场分发 profile”，便于一键部署环境。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexProfileSpec {
    /// profile 依赖的插件清单（与 module 相同语义：这些才是有代码的内容）。
    #[serde(default)]
    pub plugins: Vec<PluginIndexModulePluginSpec>,
    /// 可选：会话级后端覆盖（同 `module.backends`）。
    #[serde(default)]
    pub backends: Option<crate::models::plugin_backends::PluginBackendsOverride>,
    /// 可选：预声明权限 token（提示用途；真正授权仍以安装时对依赖插件的 consent 为准）。
    #[serde(default)]
    pub predeclared_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginIndexFile {
    #[serde(default, alias = "generated_at")]
    pub generated_at: Option<String>,
    #[serde(default)]
    pub plugins: Vec<PluginIndexEntry>,
}

pub const DEFAULT_PLUGIN_INDEX_URL: &str =
    "https://raw.githubusercontent.com/linkaiheng2233-cyber/awesome-oclive-plugins/main/plugins.json";

fn plugins_dir(state: &AppState) -> PathBuf {
    state.directory_plugins.app_data_dir().join("plugins")
}

fn cache_path(state: &AppState) -> PathBuf {
    state
        .directory_plugins
        .app_data_dir()
        .join("plugin_index_cache.json")
}

fn cache_path_for_source(state: &AppState, source_url: &str) -> PathBuf {
    // 多源缓存：按 URL sha256 分文件，避免非法文件名与过长路径
    let mut hasher = Sha256::new();
    hasher.update(source_url.trim().as_bytes());
    let digest = hasher
        .finalize()
        .as_slice()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    state
        .directory_plugins
        .app_data_dir()
        .join(format!("plugin_index_cache_{}.json", digest))
}

fn plugin_state_store_path(state: &AppState) -> PathBuf {
    state
        .directory_plugins
        .app_data_dir()
        .join("plugin_state.json")
}

pub fn load_cached_index(state: &AppState) -> Result<PluginIndexFile, AppError> {
    let p = cache_path(state);
    if !p.exists() {
        return Ok(PluginIndexFile {
            generated_at: None,
            plugins: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&p)?;
    serde_json::from_str(&raw)
        .map_err(|e| AppError::Unknown(format!("parse plugin index cache failed: {}", e)))
}

pub fn load_cached_index_for_source(
    state: &AppState,
    source_url: &str,
) -> Result<PluginIndexFile, AppError> {
    let url = source_url.trim();
    if url.is_empty() {
        return load_cached_index(state);
    }
    let p = cache_path_for_source(state, url);
    if !p.exists() {
        return Ok(PluginIndexFile {
            generated_at: None,
            plugins: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&p)?;
    serde_json::from_str(&raw)
        .map_err(|e| AppError::Unknown(format!("parse plugin index cache failed: {}", e)))
}

fn parse_ed25519_pubkey_base64(s: &str) -> Result<VerifyingKey, AppError> {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginPackageSignatureFile {
    pub plugin_id: String,
    pub pubkey_id: String,
    pub algorithm: String, // "ed25519"
    pub signature: String, // base64
    #[serde(default)]
    pub signed_at: Option<String>,
    #[serde(default)]
    pub covers: Option<String>,
}

fn verify_plugin_package_signature(
    index_entry: &PluginIndexEntry,
    sig: &PluginPackageSignatureFile,
    archive_bytes: &[u8],
) -> Result<(), AppError> {
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
) -> Result<(), AppError> {
    let sig: PluginPackageSignatureFile = serde_json::from_str(sig_text)
        .map_err(|e| AppError::Unknown(format!("parse signature.json failed: {}", e)))?;
    verify_plugin_package_signature(index_entry, &sig, archive_bytes)
}

pub fn install_plugin_from_archive_bytes_overwrite(
    state: &AppState,
    bytes: &[u8],
    overwrite: bool,
) -> Result<String, AppError> {
    let tmp = plugins_install_temp_dir(state)?;
    extract_oclive_plugin_archive(bytes, tmp.path())?;
    let manifest = OclivePluginManifest::load_from_dir(tmp.path())
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_dir(state).join(pid.as_str());
    if final_dir.exists() {
        if !overwrite {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        let _ = fs::remove_dir_all(&final_dir);
    }
    fs::create_dir_all(plugins_dir(state))?;
    fs::rename(tmp.path(), &final_dir)?;
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "archive".to_string(),
            git_url: None,
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    std::mem::forget(tmp);
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
}

pub fn sync_plugin_index_online(
    state: &AppState,
    index_url: Option<&str>,
) -> Result<PluginIndexFile, AppError> {
    let env_url = std::env::var("OCLIVE_PLUGIN_INDEX_URL").ok();
    let url = index_url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| env_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
        .unwrap_or(DEFAULT_PLUGIN_INDEX_URL);
    sync_plugin_index_online_at(state, url, &cache_path(state))
}

pub fn sync_plugin_index_online_for_source(
    state: &AppState,
    source_url: &str,
) -> Result<PluginIndexFile, AppError> {
    let url = source_url.trim();
    if url.is_empty() {
        return sync_plugin_index_online(state, None);
    }
    let cache = cache_path_for_source(state, url);
    sync_plugin_index_online_at(state, url, &cache)
}

fn sync_plugin_index_online_at(
    _state: &AppState,
    url: &str,
    cache: &Path,
) -> Result<PluginIndexFile, AppError> {
    let cli = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Unknown(format!("index http client failed: {}", e)))?;
    let resp = cli
        .get(url)
        .send()
        .map_err(|e| AppError::Unknown(format!("sync plugin index failed: {}", e)))?;
    if !resp.status().is_success() {
        return Err(AppError::Unknown(format!(
            "sync plugin index status={} url={}",
            resp.status(),
            url
        )));
    }
    let text = resp
        .text()
        .map_err(|e| AppError::Unknown(format!("read plugin index response failed: {}", e)))?;
    // Validate index contract (esp. no-code module/profile constraints) before persisting cache.
    validate_plugin_market_index_v1(&text)
        .map_err(|e| AppError::Unknown(format!("plugins.json validate failed: {}", e)))?;
    let mut parsed: PluginIndexFile = serde_json::from_str(&text)
        .map_err(|e| AppError::Unknown(format!("parse plugins.json failed: {}", e)))?;
    parsed.plugins.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(parent) = cache.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(
        cache,
        serde_json::to_string_pretty(&parsed)
            .map_err(|e| AppError::Unknown(format!("encode index cache failed: {}", e)))?,
    )?;
    Ok(parsed)
}

fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<(), AppError> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd
        .output()
        .map_err(|e| AppError::Unknown(format!("git command failed: {}", e)))?;
    if !out.status.success() {
        return Err(AppError::Unknown(format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(())
}

fn plugins_install_temp_dir(state: &AppState) -> Result<TempDir, AppError> {
    let root = state.directory_plugins.app_data_dir().join("tmp");
    let _ = fs::create_dir_all(&root);
    TempDir::new_in(root).map_err(AppError::IoError)
}

fn write_install_meta(root: &Path, meta: &PluginInstallMeta) -> Result<(), AppError> {
    let p = root.join(".oclive_install.json");
    let raw = serde_json::to_string_pretty(meta).map_err(AppError::from)?;
    fs::write(p, raw)?;
    Ok(())
}

pub fn read_install_meta(root: &Path) -> Option<PluginInstallMeta> {
    oclive_kernel_runtime::infrastructure::directory_plugins::read_plugin_install_meta(root)
}

pub fn install_plugin_from_archive_bytes(
    state: &AppState,
    bytes: &[u8],
) -> Result<String, AppError> {
    let tmp = plugins_install_temp_dir(state)?;
    extract_oclive_plugin_archive(bytes, tmp.path())?;
    let manifest = OclivePluginManifest::load_from_dir(tmp.path())
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_dir(state).join(pid.as_str());
    if final_dir.exists() {
        return Err(AppError::InvalidParameter(format!(
            "target plugin id already exists: {}",
            final_dir.display()
        )));
    }
    fs::create_dir_all(plugins_dir(state))?;
    fs::rename(tmp.path(), &final_dir)?;
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "archive".to_string(),
            git_url: None,
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    // rename 后 tmp 不再拥有目录；阻止 drop 尝试清理不存在路径
    std::mem::forget(tmp);
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    // 注意：权限授予必须来自“用户同意”（市场安装）或“开发者模式侧载”流程；
    // 这里不再自动授予 manifest 种子权限，避免绕开索引声明与用户授权。
    Ok(pid)
}

pub fn install_plugin_from_download_urls(
    state: &AppState,
    index_entry: &PluginIndexEntry,
    download_url: &str,
    signature_url: &str,
) -> Result<String, AppError> {
    let cli = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| AppError::Unknown(format!("download http client failed: {}", e)))?;
    let archive_bytes = cli
        .get(download_url)
        .send()
        .map_err(|e| AppError::Unknown(format!("download plugin failed: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Unknown(format!("download plugin status failed: {}", e)))?
        .bytes()
        .map_err(|e| AppError::Unknown(format!("read plugin bytes failed: {}", e)))?
        .to_vec();
    let sig_text = cli
        .get(signature_url)
        .send()
        .map_err(|e| AppError::Unknown(format!("download signature failed: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::Unknown(format!("download signature status failed: {}", e)))?
        .text()
        .map_err(|e| AppError::Unknown(format!("read signature text failed: {}", e)))?;
    let sig: PluginPackageSignatureFile = serde_json::from_str(&sig_text)
        .map_err(|e| AppError::Unknown(format!("parse signature.json failed: {}", e)))?;
    verify_plugin_package_signature(index_entry, &sig, &archive_bytes)?;
    install_plugin_from_archive_bytes(state, &archive_bytes)
}

pub fn install_plugin_from_git_tag(
    state: &AppState,
    git_url: &str,
    tag: &str,
    deps: Option<&HashMap<String, String>>,
) -> Result<String, AppError> {
    if let Some(deps_map) = deps {
        let miss = missing_dependencies(state, deps_map)?;
        if !miss.is_empty() {
            return Err(AppError::InvalidParameter(format!(
                "[MISSING_DEPENDENCIES] {}",
                miss.join(" | ")
            )));
        }
    }
    let url = git_url.trim();
    let tag = tag.trim();
    if url.is_empty() {
        return Err(AppError::InvalidParameter("git_url required".into()));
    }
    if tag.is_empty() {
        return Err(AppError::InvalidParameter("git tag required".into()));
    }
    let mut target = plugins_dir(state);
    fs::create_dir_all(&target)?;
    let name = url
        .split('/')
        .next_back()
        .unwrap_or("plugin")
        .trim_end_matches(".git")
        .trim();
    if name.is_empty() {
        return Err(AppError::InvalidParameter("invalid git_url".into()));
    }
    target = target.join(name);
    if target.exists() {
        return Err(AppError::InvalidParameter(format!(
            "plugin dir already exists: {}",
            target.display()
        )));
    }
    run_git(
        &[
            "clone",
            "--depth",
            "1",
            "--branch",
            tag,
            url,
            target.to_string_lossy().as_ref(),
        ],
        None,
    )?;
    let manifest = OclivePluginManifest::load_from_dir(&target)
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_dir(state).join(pid.as_str());
    if final_dir != target {
        if final_dir.exists() {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        fs::rename(&target, &final_dir)?;
    }
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "git_tag".to_string(),
            git_url: Some(url.to_string()),
            pinned_tag: Some(tag.to_string()),
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    // 不在 installer 层自动授予权限（见 install_plugin_from_archive_bytes 注释）
    Ok(pid)
}

fn installed_version_map(state: &AppState) -> HashMap<String, semver::Version> {
    let mut out = HashMap::new();
    let roots = state.directory_plugins.plugin_roots.read();
    for (pid, root) in roots.iter() {
        if let Ok(manifest) = OclivePluginManifest::load_from_dir(root) {
            if let Some(v) = parse_manifest_version(&manifest.version) {
                out.insert(pid.clone(), v);
            }
        }
    }
    out
}

pub fn missing_dependencies(
    state: &AppState,
    deps: &HashMap<String, String>,
) -> Result<Vec<String>, AppError> {
    let versions = installed_version_map(state);
    let mut missing = Vec::new();
    for (dep_id, req_s) in deps {
        let dep = dep_id.trim();
        if dep.is_empty() {
            continue;
        }
        let req = VersionReq::parse(req_s.trim()).map_err(|e| {
            AppError::InvalidParameter(format!(
                "dependency range invalid dep={} req={} err={}",
                dep, req_s, e
            ))
        })?;
        match versions.get(dep) {
            None => missing.push(format!("{} ({})", dep, req_s)),
            Some(v) => {
                if !req.matches(v) {
                    missing.push(format!("{} (need {}, local {})", dep, req_s, v));
                }
            }
        }
    }
    Ok(missing)
}

pub fn install_plugin(
    state: &AppState,
    git_url: &str,
    deps: Option<&HashMap<String, String>>,
) -> Result<String, AppError> {
    if let Some(deps_map) = deps {
        let miss = missing_dependencies(state, deps_map)?;
        if !miss.is_empty() {
            return Err(AppError::InvalidParameter(format!(
                "[MISSING_DEPENDENCIES] {}",
                miss.join(" | ")
            )));
        }
    }
    let url = git_url.trim();
    if url.is_empty() {
        return Err(AppError::InvalidParameter("git_url required".into()));
    }
    let mut target = plugins_dir(state);
    fs::create_dir_all(&target)?;
    let name = url
        .split('/')
        .next_back()
        .unwrap_or("plugin")
        .trim_end_matches(".git")
        .trim();
    if name.is_empty() {
        return Err(AppError::InvalidParameter("invalid git_url".into()));
    }
    target = target.join(name);
    if target.exists() {
        return Err(AppError::InvalidParameter(format!(
            "plugin dir already exists: {}",
            target.display()
        )));
    }
    run_git(
        &[
            "clone",
            "--depth",
            "1",
            url,
            target.to_string_lossy().as_ref(),
        ],
        None,
    )?;
    let manifest = OclivePluginManifest::load_from_dir(&target)
        .map_err(|e| AppError::Unknown(format!("manifest validation failed: {}", e)))?;
    let pid = manifest.id.trim().to_string();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("manifest.id required".into()));
    }
    let final_dir = plugins_dir(state).join(pid.as_str());
    if final_dir != target {
        if final_dir.exists() {
            return Err(AppError::InvalidParameter(format!(
                "target plugin id already exists: {}",
                final_dir.display()
            )));
        }
        fs::rename(&target, &final_dir)?;
    }
    let _ = write_install_meta(
        &final_dir,
        &PluginInstallMeta {
            install_method: "git".to_string(),
            git_url: Some(url.to_string()),
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    );
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    // 不在 installer 层自动授予权限（见 install_plugin_from_archive_bytes 注释）
    Ok(pid)
}

pub fn update_plugin(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()));
    }
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots
            .get(pid)
            .cloned()
            .ok_or_else(|| AppError::InvalidParameter(format!("plugin not found: {}", pid)))?
    };
    if let Some(meta) = read_install_meta(&root) {
        if let Some(tag) = meta
            .pinned_tag
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return Err(AppError::InvalidParameter(format!(
                "[PLUGIN_PINNED_VERSION] plugin is pinned to tag {}; update via market version install",
                tag
            )));
        }
    }
    run_git(&["pull", "--ff-only"], Some(&root))?;
    let _ = OclivePluginManifest::load_from_dir(&root)
        .map_err(|e| AppError::Unknown(format!("manifest validation failed after pull: {}", e)))?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}

fn remove_plugin_from_state_store(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Ok(());
    }
    let p = plugin_state_store_path(state);
    let mut store = PluginStateStore::load(&p);
    if let Some(g) = store.global.as_mut() {
        g.slots.disabled_plugins.retain(|x| x.trim() != pid);
        g.slots
            .slot_order
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        g.slots
            .disabled_slot_contributions
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        g.slots.slot_appearance.remove(pid);
        if g.shell_plugin_id.trim() == pid {
            g.shell_plugin_id.clear();
        }
    }
    for role in store.roles.values_mut() {
        role.slots.disabled_plugins.retain(|x| x.trim() != pid);
        role.slots
            .slot_order
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        role.slots
            .disabled_slot_contributions
            .values_mut()
            .for_each(|v| v.retain(|x| x.trim() != pid));
        role.slots.slot_appearance.remove(pid);
        if role.shell_plugin_id.trim() == pid {
            role.shell_plugin_id.clear();
        }
    }
    store
        .save(&p)
        .map_err(|e| AppError::Unknown(format!("save plugin_state failed: {}", e)))?;
    let _ = state.directory_plugins.reload_plugin_state();
    Ok(())
}

pub fn uninstall_plugin(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
    let pid = plugin_id.trim();
    if pid.is_empty() {
        return Err(AppError::InvalidParameter("plugin_id required".into()));
    }
    let root = {
        let roots = state.directory_plugins.plugin_roots.read();
        roots
            .get(pid)
            .cloned()
            .ok_or_else(|| AppError::InvalidParameter(format!("plugin not found: {}", pid)))?
    };
    state.directory_plugins.clear_plugin_process(pid);
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    remove_plugin_from_state_store(state, pid)?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}
