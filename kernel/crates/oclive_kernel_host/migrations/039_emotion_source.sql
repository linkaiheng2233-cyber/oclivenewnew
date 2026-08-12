-- B M1 slice 3: archival provenance for the bot complex-emotion source.
-- emotion_source: llm (main-LLM [EMO] marker), degraded (degraded keep), or the
-- raw provider source for fast / plugin / unknown paths. NULL for user rows and
-- for rows written before this migration. Independent from provider source="none".
ALTER TABLE chat_messages ADD COLUMN emotion_source TEXT;
