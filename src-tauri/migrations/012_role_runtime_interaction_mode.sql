-- 每角色交互模式（沉浸 vs 纯聊）；NULL 时在 load_role 中从 settings 或旧版 app_settings 解析并回填
ALTER TABLE role_runtime ADD COLUMN interaction_mode TEXT;
