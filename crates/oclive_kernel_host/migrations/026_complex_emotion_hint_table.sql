-- Dedicated persistence for co-present complex emotion narrative_hint (TTL on read).
-- Replaces use of role_runtime.complex_emotion_hint column from migration 015.
CREATE TABLE IF NOT EXISTS complex_emotion_hint (
    srid TEXT PRIMARY KEY NOT NULL,
    narrative_hint TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_complex_emotion_hint_updated_at ON complex_emotion_hint(updated_at);
