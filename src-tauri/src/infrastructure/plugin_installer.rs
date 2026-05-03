use crate::error::AppError;
use crate::state::AppState;
pub use oclive_kernel_runtime::infrastructure::plugin_archive::{
    extract_oclive_plugin_archive, peek_plugin_id_from_archive_bytes,
};
pub use oclive_kernel_runtime::infrastructure::plugin_index_sync::{
    load_plugin_index_cache, plugin_index_cache_path_for_source, plugin_index_default_cache_path,
    resolve_plugin_index_url, sync_plugin_index_from_url, DEFAULT_PLUGIN_INDEX_URL,
};
use oclive_kernel_runtime::infrastructure::plugin_install::{
    install_plugin_from_archive_bytes_at, install_plugin_from_archive_bytes_overwrite_at,
    install_plugin_from_download_urls_at, install_plugin_from_git_head_at,
    install_plugin_from_git_tag_at, installed_plugin_version_map, missing_plugin_dependencies,
    plugin_state_store_default_path, remove_plugin_from_plugin_state_file_async,
    update_git_plugin_at,
    update_install_meta_permissions_at,
};
pub use oclive_kernel_runtime::infrastructure::plugin_package_verify::verify_plugin_package_signature_text;
pub use oclive_kernel_runtime::models::plugin_market_index::{
    PluginIndexEntry, PluginIndexFile, PluginIndexModulePluginSpec, PluginIndexModuleSpec,
    PluginIndexProfileSpec, PluginIndexVersionEntry, PublisherPublicKey,
};

pub type PluginInstallMeta = crate::models::dto::PluginInstallMetaDto;

fn plugins_dir(state: &AppState) -> std::path::PathBuf {
    state.directory_plugins.app_data_dir().join("plugins")
}

fn plugin_state_store_path(state: &AppState) -> std::path::PathBuf {
    plugin_state_store_default_path(state.directory_plugins.app_data_dir())
}

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
    update_install_meta_permissions_at(&root, declared_permissions, granted_permissions)
}

pub fn load_cached_index(state: &AppState) -> Result<PluginIndexFile, AppError> {
    let p = plugin_index_default_cache_path(state.directory_plugins.app_data_dir());
    load_plugin_index_cache(&p)
}

pub fn load_cached_index_for_source(
    state: &AppState,
    source_url: &str,
) -> Result<PluginIndexFile, AppError> {
    let url = source_url.trim();
    if url.is_empty() {
        return load_cached_index(state);
    }
    let p = plugin_index_cache_path_for_source(state.directory_plugins.app_data_dir(), url);
    load_plugin_index_cache(&p)
}

pub fn install_plugin_from_archive_bytes_overwrite(
    state: &AppState,
    bytes: &[u8],
    overwrite: bool,
) -> Result<String, AppError> {
    let pid = install_plugin_from_archive_bytes_overwrite_at(
        &plugins_dir(state),
        state.directory_plugins.app_data_dir(),
        bytes,
        overwrite,
        &PluginInstallMeta {
            install_method: "archive".to_string(),
            git_url: None,
            pinned_tag: None,
            declared_permissions: Vec::new(),
            granted_permissions: Vec::new(),
        },
    )?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
}

pub async fn sync_plugin_index_online(
    state: &AppState,
    index_url: Option<&str>,
) -> Result<PluginIndexFile, AppError> {
    let url = resolve_plugin_index_url(index_url);
    let cache = plugin_index_default_cache_path(state.directory_plugins.app_data_dir());
    sync_plugin_index_from_url(&url, &cache).await
}

pub async fn sync_plugin_index_online_for_source(
    state: &AppState,
    source_url: &str,
) -> Result<PluginIndexFile, AppError> {
    let url = source_url.trim();
    if url.is_empty() {
        return sync_plugin_index_online(state, None).await;
    }
    let cache = plugin_index_cache_path_for_source(state.directory_plugins.app_data_dir(), url);
    sync_plugin_index_from_url(url, &cache).await
}

pub fn read_install_meta(root: &std::path::Path) -> Option<PluginInstallMeta> {
    oclive_kernel_runtime::infrastructure::directory_plugins::read_plugin_install_meta(root)
}

pub fn install_plugin_from_archive_bytes(
    state: &AppState,
    bytes: &[u8],
) -> Result<String, AppError> {
    let pid = install_plugin_from_archive_bytes_at(
        &plugins_dir(state),
        state.directory_plugins.app_data_dir(),
        bytes,
    )?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
}

pub fn install_plugin_from_download_urls(
    state: &AppState,
    index_entry: &PluginIndexEntry,
    download_url: &str,
    signature_url: &str,
) -> Result<String, AppError> {
    let pid = install_plugin_from_download_urls_at(
        &plugins_dir(state),
        state.directory_plugins.app_data_dir(),
        index_entry,
        download_url,
        signature_url,
    )?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
}

pub fn install_plugin_from_git_tag(
    state: &AppState,
    git_url: &str,
    tag: &str,
    deps: Option<&std::collections::HashMap<String, String>>,
) -> Result<String, AppError> {
    let versions = {
        let roots = state.directory_plugins.plugin_roots.read();
        installed_plugin_version_map(&roots)
    };
    let pid = install_plugin_from_git_tag_at(&plugins_dir(state), git_url, tag, &versions, deps)?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(pid)
}

pub fn missing_dependencies(
    state: &AppState,
    deps: &std::collections::HashMap<String, String>,
) -> Result<Vec<String>, AppError> {
    let roots = state.directory_plugins.plugin_roots.read();
    let versions = installed_plugin_version_map(&roots);
    missing_plugin_dependencies(&versions, deps)
}

pub fn install_plugin(
    state: &AppState,
    git_url: &str,
    deps: Option<&std::collections::HashMap<String, String>>,
) -> Result<String, AppError> {
    let versions = {
        let roots = state.directory_plugins.plugin_roots.read();
        installed_plugin_version_map(&roots)
    };
    let pid = install_plugin_from_git_head_at(&plugins_dir(state), git_url, &versions, deps)?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
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
    update_git_plugin_at(&root)?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}

pub async fn uninstall_plugin(state: &AppState, plugin_id: &str) -> Result<(), AppError> {
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
    let root_for_rm = root.clone();
    tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        if root_for_rm.exists() {
            std::fs::remove_dir_all(&root_for_rm)?;
        }
        Ok(())
    })
    .await
    .map_err(|e| {
        AppError::InvalidParameter(format!("[PLUGIN_UNINSTALL] remove_dir join: {}", e))
    })??;

    remove_plugin_from_plugin_state_file_async(&plugin_state_store_path(state), pid).await?;
    state
        .directory_plugins
        .reload_plugin_state_async()
        .await
        .map_err(|e| AppError::InvalidParameter(format!("[PLUGIN_STATE_RELOAD] {}", e)))?;
    state
        .directory_plugins
        .rescan_plugin_roots(state.storage.roles_dir());
    Ok(())
}
