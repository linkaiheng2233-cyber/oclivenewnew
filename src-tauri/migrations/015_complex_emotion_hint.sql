-- complex_emotion: persist previous narrative hint for next turn injection
ALTER TABLE role_runtime ADD COLUMN complex_emotion_hint TEXT;

