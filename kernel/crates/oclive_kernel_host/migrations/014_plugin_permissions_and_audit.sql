-- Plugin permission grants + minimal audit metadata (v1)

CREATE TABLE IF NOT EXISTS plugin_permission_grants (
    plugin_id TEXT NOT NULL,
    permission TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (plugin_id, permission)
);

CREATE INDEX IF NOT EXISTS idx_plugin_permission_grants_plugin_id
    ON plugin_permission_grants (plugin_id);

CREATE TABLE IF NOT EXISTS plugin_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    action TEXT NOT NULL,
    permission TEXT,
    allowed INTEGER NOT NULL,
    meta_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_plugin_audit_log_plugin_id
    ON plugin_audit_log (plugin_id);

CREATE INDEX IF NOT EXISTS idx_plugin_audit_log_created_at
    ON plugin_audit_log (created_at);

