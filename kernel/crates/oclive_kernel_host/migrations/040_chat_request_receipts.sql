-- Retain identities even when outcomes are unavailable: no lease takeover or TTL
-- deletion may turn an uncertain old request into a second execution.
CREATE TABLE chat_request_receipts (
    scope_key TEXT NOT NULL,
    request_id TEXT NOT NULL,
    payload_sha256 TEXT NOT NULL,
    session_namespace TEXT NOT NULL,
    role_id TEXT NOT NULL,
    scene_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'unconfirmed')),
    response_json TEXT,
    user_message_id TEXT,
    assistant_message_id TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(scope_key, request_id)
);
CREATE INDEX idx_chat_request_role_scene ON chat_request_receipts(role_id, scene_id);
CREATE INDEX idx_chat_request_session ON chat_request_receipts(session_namespace);
CREATE INDEX idx_chat_request_user ON chat_request_receipts(user_message_id);
CREATE INDEX idx_chat_request_assistant ON chat_request_receipts(assistant_message_id);

-- Deleted history must not be resurrected from cached responses. Keep a tombstone
-- to prevent the same request from executing again. Normal old-message pruning
-- invalidates only receipts which actually reference the deleted message.
CREATE TRIGGER chat_request_message_deleted AFTER DELETE ON chat_messages
BEGIN
    UPDATE chat_request_receipts
    SET status = 'unconfirmed', response_json = NULL
    WHERE user_message_id = OLD.id OR assistant_message_id = OLD.id;
END;
CREATE TRIGGER chat_request_session_deleted AFTER DELETE ON chat_sessions
BEGIN
    UPDATE chat_request_receipts
    SET status = 'unconfirmed', response_json = NULL
    WHERE session_namespace = OLD.session_id;
END;
