//! Filesystem + SQLite aggregation for chat storage management UI.

use super::config::{
    read_role_chat_storage_location, resolve_role_chat_storage_root, resolve_session_dir,
    resolve_storage_root, sanitize_path_segment,
};
use super::mirror::MirrorDocument;
use super::types::{RoleStorageStat, SceneStorageStat};
use crate::error::{AppError, Result};
use crate::infrastructure::db::DbManager;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Default)]
struct SceneAgg {
    session_count: u32,
    last_active: Option<String>,
    file_bytes: u64,
}

/// Global mirror root plus each role's resolved root (`role_pack` may differ from global).
#[must_use]
pub fn enumerate_chat_mirror_roots(
    app_data_dir: &Path,
    roles_dir: &Path,
    storage_root: &Path,
) -> Vec<PathBuf> {
    let mut roots = vec![storage_root.to_path_buf()];
    let global = resolve_storage_root(app_data_dir);
    if global != *storage_root && !roots.iter().any(|r| r == &global) {
        roots.push(global);
    }
    if !roles_dir.is_dir() {
        return roots;
    }
    let Ok(entries) = std::fs::read_dir(roles_dir) else {
        return roots;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let role_id = entry.file_name().to_string_lossy().into_owned();
        let root = resolve_role_chat_storage_root(app_data_dir, roles_dir, &role_id, None);
        if !roots.iter().any(|r| r == &root) {
            roots.push(root);
        }
    }
    roots
}

/// Sum file sizes under `{root}/{role_id}/{scene_id}/` trees into `by_role`.
async fn accumulate_mirror_tree_bytes(
    root: &Path,
    by_role: &mut BTreeMap<String, BTreeMap<String, SceneAgg>>,
) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    let mut roles = fs::read_dir(root).await.map_err(AppError::IoError)?;
    while let Some(role_entry) = roles.next_entry().await.map_err(AppError::IoError)? {
        let role_path = role_entry.path();
        if !role_path.is_dir() {
            continue;
        }
        let role_id = role_entry.file_name().to_string_lossy().into_owned();
        let scenes_map = by_role.entry(role_id).or_default();
        let mut scenes = fs::read_dir(&role_path).await.map_err(AppError::IoError)?;
        while let Some(scene_entry) = scenes.next_entry().await.map_err(AppError::IoError)? {
            let scene_path = scene_entry.path();
            if !scene_path.is_dir() {
                continue;
            }
            let scene_id = scene_entry.file_name().to_string_lossy().into_owned();
            let bytes = dir_size_bytes(&scene_path).await?;
            let agg = scenes_map.entry(scene_id).or_default();
            agg.file_bytes = agg.file_bytes.saturating_add(bytes);
        }
    }
    Ok(())
}

async fn accumulate_role_pack_mirror_trees(
    app_data_dir: &Path,
    roles_dir: &Path,
    by_role: &mut BTreeMap<String, BTreeMap<String, SceneAgg>>,
) -> Result<()> {
    if !roles_dir.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(roles_dir).await.map_err(AppError::IoError)?;
    while let Some(entry) = entries.next_entry().await.map_err(AppError::IoError)? {
        if !entry.file_type().await.map_err(AppError::IoError)?.is_dir() {
            continue;
        }
        let role_id = entry.file_name().to_string_lossy().into_owned();
        if read_role_chat_storage_location(roles_dir, &role_id) != "role_pack" {
            continue;
        }
        let role_root = resolve_role_chat_storage_root(
            app_data_dir,
            roles_dir,
            &role_id,
            Some("role_pack"),
        );
        accumulate_mirror_tree_bytes(&role_root, by_role).await?;
    }
    Ok(())
}

/// Count JSON session files and bytes under mirror roots (file backend).
async fn accumulate_file_sessions_from_root(
    root: &Path,
    by_role: &mut BTreeMap<String, BTreeMap<String, SceneAgg>>,
) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    let mut roles = fs::read_dir(root).await.map_err(AppError::IoError)?;
    while let Some(role_entry) = roles.next_entry().await.map_err(AppError::IoError)? {
        let role_path = role_entry.path();
        if !role_path.is_dir() {
            continue;
        }
        let role_id = role_entry.file_name().to_string_lossy().into_owned();
        let scenes_map = by_role.entry(role_id).or_default();
        let mut scenes = fs::read_dir(&role_path).await.map_err(AppError::IoError)?;
        while let Some(scene_entry) = scenes.next_entry().await.map_err(AppError::IoError)? {
            let scene_path = scene_entry.path();
            if !scene_path.is_dir() {
                continue;
            }
            let scene_id = scene_entry.file_name().to_string_lossy().into_owned();
            let agg = scenes_map.entry(scene_id).or_default();
            let mut files = fs::read_dir(&scene_path).await.map_err(AppError::IoError)?;
            while let Some(file_entry) = files.next_entry().await.map_err(AppError::IoError)? {
                let p = file_entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let bytes = file_entry.metadata().await.map_err(AppError::IoError)?.len();
                agg.file_bytes = agg.file_bytes.saturating_add(bytes);
                agg.session_count = agg.session_count.saturating_add(1);
                if let Ok(raw) = fs::read_to_string(&p).await {
                    if let Ok(doc) = serde_json::from_str::<MirrorDocument>(&raw) {
                        let newer = agg
                            .last_active
                            .as_ref()
                            .map(|cur| doc.updated_at.as_str() > cur.as_str())
                            .unwrap_or(true);
                        if newer {
                            agg.last_active = Some(doc.updated_at);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn build_role_storage_stats(
    by_role: BTreeMap<String, BTreeMap<String, SceneAgg>>,
) -> Vec<RoleStorageStat> {
    let mut out: Vec<RoleStorageStat> = Vec::new();
    for (role_id, scenes_map) in by_role {
        let mut scenes: Vec<SceneStorageStat> = Vec::new();
        let mut role_bytes = 0u64;
        let mut role_last: Option<String> = None;
        for (scene_id, agg) in scenes_map {
            role_bytes = role_bytes.saturating_add(agg.file_bytes);
            if let Some(ref la) = agg.last_active {
                let newer = role_last
                    .as_ref()
                    .map(|cur| la.as_str() > cur.as_str())
                    .unwrap_or(true);
                if newer {
                    role_last = Some(la.clone());
                }
            }
            scenes.push(SceneStorageStat {
                scene_id,
                session_count: agg.session_count,
                total_size_bytes: agg.file_bytes,
                last_active: agg.last_active,
            });
        }
        scenes.sort_by(|a, b| a.scene_id.cmp(&b.scene_id));
        out.push(RoleStorageStat {
            role_id,
            total_size_bytes: role_bytes,
            scene_count: scenes.len() as u32,
            last_active: role_last,
            scenes,
        });
    }
    out.sort_by(|a, b| a.role_id.cmp(&b.role_id));
    out
}

/// Collect per-role / per-scene storage stats (mirror dir sizes + SQLite session metadata).
///
/// # Errors
///
/// IO / DB errors propagate.
pub async fn collect_chat_storage_stats(
    app_data_dir: &Path,
    roles_dir: &Path,
    db: &DbManager,
) -> Result<Vec<RoleStorageStat>> {
    let root = resolve_storage_root(app_data_dir);
    let mut by_role: BTreeMap<String, BTreeMap<String, SceneAgg>> = BTreeMap::new();

    accumulate_mirror_tree_bytes(&root, &mut by_role).await?;
    accumulate_role_pack_mirror_trees(app_data_dir, roles_dir, &mut by_role).await?;

    let rows = db.list_chat_session_scene_stats().await?;
    for (role_id, scene_id, cnt, last_active) in rows {
        let scenes_map = by_role.entry(role_id).or_default();
        let agg = scenes_map.entry(scene_id).or_default();
        agg.session_count = cnt;
        if let Some(la) = last_active {
            let newer = agg
                .last_active
                .as_ref()
                .map(|cur| la.as_str() > cur.as_str())
                .unwrap_or(true);
            if newer {
                agg.last_active = Some(la);
            }
        }
    }

    Ok(build_role_storage_stats(by_role))
}

/// File-backend stats: scan JSON session files under all known mirror roots.
///
/// # Errors
///
/// IO errors propagate.
pub async fn collect_file_chat_storage_stats(
    app_data_dir: &Path,
    roles_dir: &Path,
    storage_root: &Path,
) -> Result<Vec<RoleStorageStat>> {
    let mut by_role: BTreeMap<String, BTreeMap<String, SceneAgg>> = BTreeMap::new();
    for root in enumerate_chat_mirror_roots(app_data_dir, roles_dir, storage_root) {
        accumulate_file_sessions_from_root(&root, &mut by_role).await?;
    }
    Ok(build_role_storage_stats(by_role))
}

/// Sum file sizes under a directory (non-recursive file list per subtree walk).
async fn dir_size_bytes(dir: &Path) -> Result<u64> {
    let mut total = 0u64;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        let mut read = fs::read_dir(&path).await.map_err(AppError::IoError)?;
        while let Some(entry) = read.next_entry().await.map_err(AppError::IoError)? {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(meta) = entry.metadata().await {
                total = total.saturating_add(meta.len());
            }
        }
    }
    Ok(total)
}

/// Delete mirror directory for one role+scene.
///
/// # Errors
///
/// IO / path errors propagate.
pub async fn delete_mirror_scene_dir(
    storage_root: &Path,
    role_id: &str,
    scene_id: &str,
) -> Result<u64> {
    let dir = resolve_session_dir(storage_root, role_id, scene_id)?;
    let bytes = if dir.is_dir() {
        dir_size_bytes(&dir).await?
    } else {
        0
    };
    if dir.is_dir() {
        fs::remove_dir_all(&dir).await.map_err(AppError::IoError)?;
    }
    Ok(bytes)
}

/// Storage stats from SQLite only (for `sqlite` backend).
///
/// # Errors
///
/// Propagates database errors.
pub async fn collect_chat_storage_stats_from_db(db: &DbManager) -> Result<Vec<RoleStorageStat>> {
    let rows = db.list_chat_session_scene_stats().await?;
    let mut by_role: BTreeMap<String, BTreeMap<String, SceneAgg>> = BTreeMap::new();
    for (role_id, scene_id, cnt, last_active) in rows {
        let scenes_map = by_role.entry(role_id).or_default();
        let agg = scenes_map.entry(scene_id).or_default();
        agg.session_count = cnt;
        agg.last_active = last_active;
    }
    Ok(build_role_storage_stats(by_role))
}

/// Bytes for one session mirror file (0 if missing).
pub async fn mirror_file_bytes_for_session(
    storage_root: &Path,
    session: &super::db::SessionRow,
) -> Result<u64> {
    let path = super::mirror::mirror_path_for_session(storage_root, session)?;
    if path.is_file() {
        Ok(fs::metadata(&path).await.map_err(AppError::IoError)?.len())
    } else {
        Ok(0)
    }
}

/// Bytes under `{root}/{role_id}/` before delete (for reporting).
///
/// # Errors
///
/// IO / path errors propagate.
pub async fn role_mirror_tree_bytes(storage_root: &Path, role_id: &str) -> Result<u64> {
    let role_dir = storage_root.join(sanitize_path_segment(role_id)?);
    if role_dir.is_dir() {
        dir_size_bytes(&role_dir).await
    } else {
        Ok(0)
    }
}
