-- 各用户身份（manifest 关系键）独立的好感与关系阶段；可选「跟随创作者默认」

CREATE TABLE IF NOT EXISTS role_identity_stats (
    role_id TEXT NOT NULL,
    user_relation_key TEXT NOT NULL,
    favorability REAL NOT NULL DEFAULT 0,
    relation_state TEXT NOT NULL DEFAULT 'Stranger',
    updated_at TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (role_id, user_relation_key)
);

CREATE INDEX IF NOT EXISTS idx_role_identity_stats_role ON role_identity_stats(role_id);

ALTER TABLE role_runtime ADD COLUMN use_manifest_default INTEGER NOT NULL DEFAULT 0;

-- 将现有全局好感迁入当前 user_relation 对应键（便于按身份续玩）
INSERT OR IGNORE INTO role_identity_stats (role_id, user_relation_key, favorability, relation_state, updated_at)
SELECT role_id, user_relation, current_favorability, COALESCE(relation_state, 'Stranger'), updated_at
FROM role_runtime;
