//! Sync UI-shell LLM settings with canonical kernel `app.db`.

use oclive_kernel_host::domain::user_llm_env::{
    apply_user_llm_env, KEY_LOCAL_MODELS_DIR, KEY_REMOTE_TOKEN, LLM_APP_SETTING_KEYS,
};
use oclive_kernel_host::infrastructure::user_llm_secrets::{read_token_file, write_token_file};
use oclive_kernel_host::state::{is_managed_legacy_models_path, AppState};
use std::path::{Path, PathBuf};

async fn open_canonical_pool() -> Option<(sqlx::SqlitePool, PathBuf)> {
    use oclive_kernel_runtime::{resolve_app_data_dir_for_host, resolve_db_path};

    let app_data = resolve_app_data_dir_for_host();
    let db_path = resolve_db_path(&app_data);
    if !db_path.is_file() {
        return None;
    }
    let url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = sqlx::SqlitePool::connect(&url).await.ok()?;
    Some((pool, app_data))
}

async fn upsert_canonical_app_setting(
    pool: &sqlx::SqlitePool,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

/// Copy UI shell LLM settings into canonical `OCLive/data/app.db` (kernel single writer).
pub async fn sync_shell_llm_settings_to_canonical(state: &AppState) {
    let Some((pool, app_data)) = open_canonical_pool().await else {
        return;
    };
    for key in LLM_APP_SETTING_KEYS {
        let Ok(Some(v)) = state.db_manager.get_app_setting(key).await else {
            continue;
        };
        let t = v.trim();
        if t.is_empty() {
            continue;
        }
        let _ = upsert_canonical_app_setting(&pool, key, t).await;
    }
    if let Ok(Some(t)) = oclive_kernel_host::domain::user_llm_env::resolve_remote_token(
        state.db_manager.as_ref(),
        state.directory_plugins.app_data_dir(),
    )
    .await
    {
        let _ = write_token_file(&app_data, t.trim());
        let _ = upsert_canonical_app_setting(&pool, KEY_REMOTE_TOKEN, t.trim()).await;
    }
    pool.close().await;
    tracing::info!(
        target: "oclive_llm",
        "synced LLM app_settings to canonical kernel DB"
    );
}

/// Mirror session model override into canonical `role_runtime`.
pub async fn sync_session_ollama_model_to_canonical(session_ns: &str, model: Option<&str>) {
    use chrono::Utc;

    let Some((pool, _)) = open_canonical_pool().await else {
        return;
    };
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO role_runtime (role_id, current_favorability, updated_at) VALUES (?, 0.0, ?)",
    )
    .bind(session_ns)
    .bind(&now)
    .execute(&pool)
    .await;
    if let Some(m) = model.filter(|s| !s.trim().is_empty()) {
        let _ = sqlx::query(
            "UPDATE role_runtime SET session_ollama_model_override = ?, updated_at = ? WHERE role_id = ?",
        )
        .bind(m.trim())
        .bind(&now)
        .bind(session_ns)
        .execute(&pool)
        .await;
    } else {
        let _ = sqlx::query(
            "UPDATE role_runtime SET session_ollama_model_override = NULL, updated_at = ? WHERE role_id = ?",
        )
        .bind(&now)
        .bind(session_ns)
        .execute(&pool)
        .await;
    }
    pool.close().await;
}

/// Seed in-memory UI shell from canonical DB on desktop bootstrap.
pub async fn seed_shell_llm_from_canonical(state: &AppState) {
    let Some((pool, app_data)) = open_canonical_pool().await else {
        return;
    };
    let mut copied = 0usize;
    for key in LLM_APP_SETTING_KEYS {
        let Ok(Some(v)) =
            sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ? LIMIT 1")
                .bind(key)
                .fetch_optional(&pool)
                .await
        else {
            continue;
        };
        let t = v.trim();
        if t.is_empty() {
            continue;
        }
        if state.db_manager.upsert_app_setting(key, t).await.is_ok() {
            copied += 1;
        }
    }
    if let Some(token) = read_token_file(&app_data) {
        let t = token.trim();
        if !t.is_empty() {
            let _ = state
                .db_manager
                .upsert_app_setting(KEY_REMOTE_TOKEN, t)
                .await;
        }
    }
    pool.close().await;
    if copied > 0 {
        state.mark_user_llm_env_dirty();
        let _ = apply_user_llm_env(state).await;
        tracing::info!(
            target: "oclive_llm",
            copied,
            "seeded UI shell LLM settings from canonical DB"
        );
    }
}

/// Update canonical `OCLive/data/app.db` when it still points at legacy model folders.
pub async fn sync_canonical_db_models_dir(canonical: &Path, app_data: &Path) {
    use oclive_kernel_runtime::resolve_db_path;

    let db_path = resolve_db_path(app_data);
    if !db_path.is_file() {
        return;
    }
    let url = format!("sqlite:{}?mode=rwc", db_path.display());
    let Ok(pool) = sqlx::SqlitePool::connect(&url).await else {
        return;
    };
    let stored =
        sqlx::query_scalar::<_, String>("SELECT value FROM app_settings WHERE key = ? LIMIT 1")
            .bind(KEY_LOCAL_MODELS_DIR)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();
    let canonical_str = canonical.to_string_lossy().into_owned();
    let should_patch = stored.as_deref().is_none_or(|s| {
        let t = s.trim();
        t.is_empty() || is_managed_legacy_models_path(Path::new(t), canonical, app_data)
    });
    if should_patch
        && sqlx::query(
            "INSERT INTO app_settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(KEY_LOCAL_MODELS_DIR)
        .bind(canonical_str.trim())
        .execute(&pool)
        .await
        .is_ok()
    {
        tracing::info!(
            target: "oclive_models",
            path = %canonical.display(),
            "patched canonical app.db local models dir"
        );
    }
    pool.close().await;
}
