CREATE INDEX IF NOT EXISTS idx_memory_role_created_at
ON long_term_memory(role_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_events_role_created_at
ON events(role_id, created_at DESC);
