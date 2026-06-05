CREATE INDEX IF NOT EXISTS idx_chat_sessions_role_scene_updated
    ON chat_sessions(role_id, scene_id, updated_at DESC);
