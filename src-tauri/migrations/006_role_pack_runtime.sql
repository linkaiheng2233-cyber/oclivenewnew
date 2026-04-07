-- 角色包运行时：用户关系、演化系数、性格 core/delta、记忆场景、性格历史改为 JSON

ALTER TABLE role_runtime ADD COLUMN user_relation TEXT NOT NULL DEFAULT 'friend';
ALTER TABLE role_runtime ADD COLUMN event_impact_factor REAL NOT NULL DEFAULT 1.0;
ALTER TABLE role_runtime ADD COLUMN core_personality TEXT;
ALTER TABLE role_runtime ADD COLUMN delta_personality TEXT;

ALTER TABLE long_term_memory ADD COLUMN scene_id TEXT;

-- 性格历史：仅存有效七维快照 JSON（审计）
ALTER TABLE personality_vector RENAME TO personality_vector_legacy;

CREATE TABLE IF NOT EXISTS personality_vector (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    effective_personality TEXT NOT NULL,
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

DROP TABLE personality_vector_legacy;
