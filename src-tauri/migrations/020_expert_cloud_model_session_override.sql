-- Module 9: optional OpenAI-compatible model id override for the current session when expert graph activates cloud LLM.
-- Persists with session namespace row (same as expert_models_session_override_json).

ALTER TABLE role_runtime ADD COLUMN expert_cloud_model_session_override TEXT;
