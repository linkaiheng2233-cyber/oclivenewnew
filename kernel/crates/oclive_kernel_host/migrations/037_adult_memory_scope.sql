ALTER TABLE long_term_memory
ADD COLUMN content_scope TEXT NOT NULL DEFAULT 'ordinary'
CHECK (content_scope IN ('ordinary', 'adult'));

ALTER TABLE short_term_memory
ADD COLUMN content_scope TEXT NOT NULL DEFAULT 'ordinary'
CHECK (content_scope IN ('ordinary', 'adult'));

CREATE INDEX IF NOT EXISTS idx_ltm_role_scope_created
ON long_term_memory(role_id, content_scope, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_stm_role_scope_created
ON short_term_memory(role_id, content_scope, created_at DESC);
