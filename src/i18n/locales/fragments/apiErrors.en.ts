/** Tauri `[CODE]` → en-US copy. */
export default {
  TXN_BEGIN_FAILED: "Transaction failed to start. Try again.",
  TXN_RUNTIME_ENSURE_FAILED: "Role runtime state failed to initialize.",
  TXN_PERSONALITY_INSERT_FAILED: "Failed to save personality data.",
  TXN_FAVORABILITY_UPDATE_FAILED: "Failed to update favorability.",
  TXN_FAVORABILITY_HISTORY_INSERT_FAILED: "Failed to write favorability history.",
  TXN_MEMORY_INSERT_FAILED: "Failed to save memory.",
  TXN_SHORT_TERM_INSERT_FAILED: "Failed to write chat history.",
  TXN_SHORT_TERM_TRIM_FAILED: "Failed to trim chat history.",
  TXN_EVENT_INSERT_FAILED: "Failed to write events.",
  TXN_FAVORABILITY_READ_FAILED: "Failed to read favorability.",
  TXN_COMMIT_FAILED: "Transaction commit failed. Try again.",
  TXN_ROLLBACK_FAILED: "Transaction rollback failed. Contact support.",
  DB_ERROR: "Database error. Try again.",
  IO_ERROR:
    "Local file I/O failed. Check: (1) app data dir is writable (Settings → General → Environment check); (2) antivirus/permissions; (3) do not put data on read-only media. See CONFIGURATION_FILES.md.",
  IO_ERROR_HOST_JSON:
    "Plugin bridge returned data that could not be serialized to JSON; host/plugin API mismatch. Check logs.",
  API_PLUGIN_NOT_FOUND: "Directory plugin not found or not scanned. Check plugin id and install path.",
  API_PERMISSION_DENIED: "Insufficient plugin permissions. Declare required permissions in manifest.json.",
  API_INVALID_MANIFEST: "Invalid plugin manifest. Check manifest.json.",
  LLM_ERROR:
    "Model call failed. Check: (1) Ollama is installed and running; (2) `ollama list` shows the model and run `ollama pull` if needed; (3) `OLLAMA_MODEL` / pack model name matches; (4) `OLLAMA_BASE_URL` points to the right port (default http://localhost:11434). Settings → General → Environment check runs a quick probe.",
  ROLE_NOT_FOUND: "Role not found. Verify role_id and your `OCLIVE_ROLES_DIR` layout.",
  ROLE_NOT_FOUND_DETAIL: "Role not found or manifest missing. {detail}",
  ROLE_PACK_EXISTS: "This role id already exists. Choose overwrite to replace the local copy.",
  INVALID_PARAMETER: "Invalid parameter. Check your input.",
  INVALID_PARAMETER_DETAIL: "Invalid parameter: {detail}",
  OLLAMA_TIMEOUT: "The model took too long. Try again.",
  TXN_ROLLBACK: "Operation failed. Try again.",
  SERDE_ERROR: "Data parse error. Try again.",
  UNKNOWN_ERROR: "Unknown error. Try again.",
} as Record<string, string>;
