-- Per-session Ollama / remote model id override (UI model manager).
ALTER TABLE role_runtime ADD COLUMN session_ollama_model_override TEXT;
