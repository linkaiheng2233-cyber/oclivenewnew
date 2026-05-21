-- 角色包使用后反馈（半私密）：仅本机存储，供创作者在编写器/工具链查看与迭代。
CREATE TABLE IF NOT EXISTS role_feedback (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    role_id TEXT NOT NULL,
    session_id TEXT,
    mood_tag TEXT,
    message TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (role_id) REFERENCES role_runtime(role_id)
);

CREATE INDEX IF NOT EXISTS idx_role_feedback_role ON role_feedback(role_id);
CREATE INDEX IF NOT EXISTS idx_role_feedback_created ON role_feedback(created_at);

