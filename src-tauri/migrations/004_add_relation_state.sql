ALTER TABLE role_runtime ADD COLUMN relation_state TEXT NOT NULL DEFAULT 'Stranger';
ALTER TABLE role_runtime ADD COLUMN relation_updated_at TIMESTAMP;
