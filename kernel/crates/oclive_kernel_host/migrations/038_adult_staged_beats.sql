-- Cancellable, ordered background generation for Chat Pro adult beats.
-- Only structured text is staged. Chat/memory side effects happen at commit.

CREATE TABLE IF NOT EXISTS adult_stage_generations (
    generation_id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'cancelled', 'completed')),
    next_sequence INTEGER NOT NULL DEFAULT 0,
    next_commit_sequence INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_adult_stage_active_chat
ON adult_stage_generations(session_id, scene_id)
WHERE status = 'active';

CREATE TABLE IF NOT EXISTS adult_staged_beats (
    generation_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'memory_committed', 'committed')),
    response_json TEXT NOT NULL,
    transcript_reply TEXT NOT NULL,
    model_name TEXT,
    response_ms INTEGER NOT NULL DEFAULT 0,
    bot_emotion TEXT,
    created_at TEXT NOT NULL,
    committed_at TEXT,
    PRIMARY KEY (generation_id, sequence),
    FOREIGN KEY (generation_id) REFERENCES adult_stage_generations(generation_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_adult_staged_beats_pending
ON adult_staged_beats(generation_id, status, sequence);
