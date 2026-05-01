-- 角色运行时状态
CREATE TABLE IF NOT EXISTS role_runtime (
    role_id TEXT PRIMARY KEY,
    current_favorability REAL NOT NULL DEFAULT 0.0,
    current_scene TEXT,
    last_interaction_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 性格向量历史
CREATE TABLE IF NOT EXISTS personality_vector (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    warmth REAL NOT NULL,
    humor REAL NOT NULL,
    formality REAL NOT NULL,
    assertiveness REAL NOT NULL,
    sensitivity REAL NOT NULL,
    creativity REAL NOT NULL,
    stability REAL NOT NULL,
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

-- 短期记忆（会话级）
CREATE TABLE IF NOT EXISTS short_term_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    user_input TEXT NOT NULL,
    bot_reply TEXT NOT NULL,
    emotion TEXT,
    scene TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

-- 长期记忆
CREATE TABLE IF NOT EXISTS long_term_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    content TEXT NOT NULL,
    importance REAL NOT NULL DEFAULT 0.5,
    weight REAL NOT NULL DEFAULT 1.0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    accessed_at TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

-- 好感度历史
CREATE TABLE IF NOT EXISTS favorability_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    delta REAL NOT NULL,
    reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

-- 事件日志
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    user_emotion TEXT,
    bot_emotion TEXT,
    resolution TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

-- 操作日志
CREATE TABLE IF NOT EXISTS operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    operator_type TEXT NOT NULL,
    operation TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_personality_role ON personality_vector(role_id);
CREATE INDEX IF NOT EXISTS idx_memory_role ON long_term_memory(role_id);
CREATE INDEX IF NOT EXISTS idx_events_role ON events(role_id);
