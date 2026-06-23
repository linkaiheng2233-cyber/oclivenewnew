-- User Identity Prompt Template session state (orthogonal to user_relation; synced via maps_to_relation_id).

ALTER TABLE role_runtime ADD COLUMN active_user_identity_id TEXT;
ALTER TABLE role_runtime ADD COLUMN use_manifest_default_identity INTEGER NOT NULL DEFAULT 1;

ALTER TABLE role_scene_identity ADD COLUMN user_identity_id TEXT;
