-- Wave F: ephemeral situation archive + Deep latch (role_runtime; distinct from mutable_personality)

ALTER TABLE role_runtime ADD COLUMN ephemeral_personality TEXT NOT NULL DEFAULT '';
ALTER TABLE role_runtime ADD COLUMN ephemeral_ttl_turns INTEGER NOT NULL DEFAULT 0;
ALTER TABLE role_runtime ADD COLUMN deep_latch_active INTEGER NOT NULL DEFAULT 0;
