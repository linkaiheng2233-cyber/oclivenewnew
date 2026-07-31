-- Scene-local narrative continuity state. The state payload remains in the role pack;
-- runtime stores only the selected scene/state ids and a CAS revision.
ALTER TABLE role_runtime ADD COLUMN continuity_scene_id TEXT;
ALTER TABLE role_runtime ADD COLUMN continuity_state_id TEXT;
ALTER TABLE role_runtime ADD COLUMN continuity_revision INTEGER NOT NULL DEFAULT 0;
