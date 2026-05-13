//! 集成级烟测：`plugin_backends` 全部为 `builtin_v2` 时 `PluginHost` 能解析五条线（不跑完整对话）。

use oclivenewnew_tauri::domain::plugin_host::PluginHost;
use oclivenewnew_tauri::infrastructure::db::DbManager;
use oclivenewnew_tauri::infrastructure::llm::LlmClient;
use oclivenewnew_tauri::infrastructure::MockLlmClient;
use oclivenewnew_tauri::models::{
    EmotionBackend, EventBackend, LlmBackend, MemoryBackend, PluginBackends, PromptBackend, Role,
};
use std::sync::Arc;

#[test]
fn resolve_role_with_all_builtin_v2() {
    let llm: Arc<dyn LlmClient> = Arc::new(MockLlmClient {
        reply: String::new(),
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(async { sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap() });
    rt.block_on(async {
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS plugin_permission_grants (
                plugin_id TEXT NOT NULL,
                permission TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 0,
                granted_by TEXT NOT NULL DEFAULT 'test',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (plugin_id, permission)
            );",
        )
        .execute(&pool)
        .await;
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS plugin_audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plugin_id TEXT NOT NULL,
                action TEXT NOT NULL,
                permission TEXT,
                allowed INTEGER NOT NULL,
                meta_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )
        .execute(&pool)
        .await;
    });
    let db = Arc::new(DbManager::new(pool));
    let host = PluginHost::new(
        db,
        llm,
        None,
        std::env::temp_dir(),
        std::sync::Arc::new(parking_lot::RwLock::new(None)),
    );
    let role = Role {
        plugin_backends: PluginBackends {
            memory: MemoryBackend::BuiltinV2,
            emotion: EmotionBackend::BuiltinV2,
            event: EventBackend::BuiltinV2,
            prompt: PromptBackend::BuiltinV2,
            llm: LlmBackend::Ollama,
            ..Default::default()
        },
        ..Default::default()
    };
    let _pl = host.resolve_for_role(&role);
}
