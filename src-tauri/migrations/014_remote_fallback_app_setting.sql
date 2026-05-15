-- 远端 HTTP 插件失败时是否自动降级内置（与主应用设置「高级」开关及 app_settings 键 remote_fallback_to_builtin 一致）
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('remote_fallback_to_builtin', '1');
