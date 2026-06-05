CREATE INDEX IF NOT EXISTS idx_favorability_history_role
ON favorability_history(role_id, created_at DESC);
