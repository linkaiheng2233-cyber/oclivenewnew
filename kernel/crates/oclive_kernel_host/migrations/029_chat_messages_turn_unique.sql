-- Enforce one user row per (session_id, turn_index); assistant shares turn_index on a separate row.
CREATE UNIQUE INDEX IF NOT EXISTS idx_chat_messages_session_turn ON chat_messages(session_id, turn_index) WHERE sender = 'user';
