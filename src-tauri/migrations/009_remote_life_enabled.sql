-- 异地「生活轨迹」：用户侧开关（持久化）
ALTER TABLE role_runtime ADD COLUMN remote_life_enabled INTEGER NOT NULL DEFAULT 0;
