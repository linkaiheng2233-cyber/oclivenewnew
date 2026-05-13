ALTER TABLE role_runtime ADD COLUMN current_emotion TEXT NOT NULL DEFAULT 'Neutral';
ALTER TABLE role_runtime ADD COLUMN emotion_updated_at TIMESTAMP;
