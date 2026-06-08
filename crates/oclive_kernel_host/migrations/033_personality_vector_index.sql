-- K-PERF-08: composite index for latest personality_vector per role_id
CREATE INDEX IF NOT EXISTS idx_personality_role_created_at
    ON personality_vector (role_id, created_at DESC);
