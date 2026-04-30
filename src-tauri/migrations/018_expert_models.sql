-- Module 9: Expert Models + PromptStyle runtime overrides (JSON, nullable)
--
-- Persistence rule: no new tables. Store as JSON blobs on role_runtime.
-- - role-level default lives on manifest role_id row
-- - session override lives on session namespace row (role_id__sess__xxx)

ALTER TABLE role_runtime ADD COLUMN expert_models_role_default_json TEXT;
ALTER TABLE role_runtime ADD COLUMN expert_models_session_override_json TEXT;
ALTER TABLE role_runtime ADD COLUMN expert_prompt_style_role_default_json TEXT;
ALTER TABLE role_runtime ADD COLUMN expert_prompt_style_session_override_json TEXT;

