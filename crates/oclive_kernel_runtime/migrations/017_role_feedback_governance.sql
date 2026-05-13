-- role_feedback 治理字段：已读/已处理、处理备注、上下文与版本信息。
-- 注意：SQLite 支持 ADD COLUMN（不支持 DROP/ALTER COLUMN）；这里按“向后兼容追加列”设计。

ALTER TABLE role_feedback ADD COLUMN status TEXT NOT NULL DEFAULT 'open';
ALTER TABLE role_feedback ADD COLUMN read_at TIMESTAMP;
ALTER TABLE role_feedback ADD COLUMN handled_at TIMESTAMP;
ALTER TABLE role_feedback ADD COLUMN handled_note TEXT;

-- 上下文（可选）
ALTER TABLE role_feedback ADD COLUMN scene_id TEXT;
ALTER TABLE role_feedback ADD COLUMN presence_mode TEXT;

-- 版本与来源（可选；用于导出与问题定位）
ALTER TABLE role_feedback ADD COLUMN role_version TEXT;
ALTER TABLE role_feedback ADD COLUMN runtime_version TEXT;
ALTER TABLE role_feedback ADD COLUMN client_version TEXT;
ALTER TABLE role_feedback ADD COLUMN source TEXT;

CREATE INDEX IF NOT EXISTS idx_role_feedback_status_created ON role_feedback(status, created_at);

