/** Kernel `KernelErrorBody.code` → en-US copy. */
export default {
  EMPTY_MESSAGE: 'Message must not be empty or whitespace-only. Type at least one visible character.',
  INVALID_ROLE_PATH: 'role_path is not a valid directory. Pass an absolute path to a folder that contains manifest.json.',
  LOAD_ROLE_TASK_PANIC: 'The load-role task panicked. Check logs and retry; file an issue with repro if it persists.',
  TXN_BEGIN_FAILED: 'Transaction failed to start. Try again.',
  TXN_RUNTIME_ENSURE_FAILED: 'Role runtime state failed to initialize.',
  TXN_PERSONALITY_INSERT_FAILED: 'Failed to save personality data.',
  TXN_FAVORABILITY_UPDATE_FAILED: 'Failed to update favorability.',
  TXN_FAVORABILITY_HISTORY_INSERT_FAILED: 'Failed to write favorability history.',
  TXN_MEMORY_INSERT_FAILED: 'Failed to save memory.',
  TXN_SHORT_TERM_INSERT_FAILED: 'Failed to write chat history.',
  TXN_SHORT_TERM_TRIM_FAILED: 'Failed to trim chat history.',
  TXN_EVENT_INSERT_FAILED: 'Failed to write events.',
  TXN_FAVORABILITY_READ_FAILED: 'Failed to read favorability.',
  TXN_COMMIT_FAILED: 'Transaction commit failed. Try again.',
  TXN_ROLLBACK_FAILED: 'Transaction rollback failed. Contact support.',
  TXN_MEMORY_ID_FETCH_FAILED: 'Could not read memory row id after insert. Retry or check DB logs.',
  TXN_EVENT_ID_FETCH_FAILED: 'Could not read event row id after insert. Retry or check DB logs.',
  TXN_IDENTITY_ENSURE_FAILED: 'Identity/relation bootstrap failed. Retry.',
  TXN_IDENTITY_FAVOR_UPDATE_FAILED: 'Identity favorability link update failed. Retry.',
  TXN_RUNTIME_MIRROR_FAILED: 'Runtime mirror sync failed. Retry.',
  TXN_MEMORY_FIFO_TRIM_FAILED: 'Short-term memory FIFO trim failed. Retry.',
  DB_ERROR: 'Database error. Try again.',
  DB_MIGRATION_FAILED:
    'Database migration failed. Back up app.db under your app data directory and retry; if it persists, check migration_failed.json and logs.',
  IO_ERROR:
    'Local file I/O failed. Check: (1) app data dir is writable (Settings → General → Environment check); (2) antivirus/permissions; (3) do not put data on read-only media. See CONFIGURATION_FILES.md.',
  IO_ERROR_HOST_JSON:
    'Plugin bridge returned data that could not be serialized to JSON; host/plugin API mismatch. Check logs.',
  API_PLUGIN_NOT_FOUND: 'Directory plugin not found or not scanned. Check plugin id and install path.',
  API_PERMISSION_DENIED: 'Insufficient plugin permissions. Declare required permissions in manifest.json.',
  API_INVALID_MANIFEST: 'Invalid plugin manifest. Check manifest.json.',
  PLUGIN_MANIFEST_INVALID: 'Invalid plugin manifest. Check manifest.json against PLUGIN_V1.',
  LLM_ERROR:
    'Model call failed. Check: (1) If `OCLIVE_LLM_BACKEND=ollama` (default): Ollama is running, `ollama list` / `ollama pull` matches `OLLAMA_MODEL`, and `OLLAMA_BASE_URL` is correct; (2) If **remote**: `OCLIVE_REMOTE_LLM_URL` is reachable, timeouts (`OCLIVE_REMOTE_LLM_TIMEOUT_MS`) are reasonable, and upstream is healthy. Settings → General → Environment check probes local Ollama.',
  VOICE_RPC_TIMEOUT:
    'Voice plugin RPC timed out (CosyVoice warm/synth is slow). In Settings → Voice, run “Warm TTS sidecar” and wait; first synthesis may take several minutes. Optional: raise `OCLIVE_VOICE_RPC_TIMEOUT_MS` (defaults: speak 600000 / warm 900000 ms).',
  ROLE_NOT_FOUND: 'Role not found. Verify role_id and your `OCLIVE_ROLES_DIR` layout.',
  ROLE_NOT_FOUND_DETAIL: 'Role not found or manifest missing. {detail}',
  ROLE_RUNTIME_NOT_READY:
    'Role runtime is not initialized yet (call load_role / pick the role in the UI first).',
  STARTUP_HEALTH_FAILED:
    'Startup health checks failed: {detail}. Verify manifest.json, plugin backend slots, DB writable; or set `OCLIVE_SKIP_STARTUP_HEALTH=1` temporarily for troubleshooting only.',
  PLUGIN_BACKENDS_DIRECTORY_SLOT:
    'Plugin backend configuration is incomplete: when a slot uses a directory backend, `directory_plugins` must list a non-empty plugin id for that slot. Use Plugins & backends → Backends or edit pack settings.',
  ROLE_PACK_EXISTS: 'This role id already exists. Choose overwrite to replace the local copy.',
  INVALID_PARAMETER: 'Invalid parameter. Check your input.',
  INVALID_PARAMETER_DETAIL: 'Invalid parameter: {detail}',
  HIGH_RISK_CAPABILITY_NOT_GRANTED:
    'This high-risk capability is not granted yet (MCP transport or directory plugin process). Grant it under Plugins & backends → Agent debug, or use your distro’s explicit consent flow.',
  REMOTE_SERVICE_UNAVAILABLE:
    'The remote HTTP plugin or sidecar is unavailable, and automatic fallback to built-in is disabled. Check that `OCLIVE_REMOTE_PLUGIN_URL` / `OCLIVE_REMOTE_LLM_URL` are reachable, re-enable fallback under Settings → General, or set `OCLIVE_REMOTE_FALLBACK_TO_BUILTIN=1`.',
  OLLAMA_TIMEOUT: 'The model took too long. Try again.',
  TXN_ROLLBACK: 'Operation failed. Try again.',
  SERDE_ERROR: 'Data parse error. Try again.',
  UNKNOWN_ERROR:
    'Unknown error. Retry; if it looks network- or service-related, check proxy/firewall and env vars (see ERROR_CODES §1.6). If it persists, capture `oclive_chat` / `oclive_plugin` log snippets.',
  UNKNOWN_WITH_CODE: 'Something went wrong ({code}). Retry or check logs; restart the app if the UI stays broken.',
} as Record<string, string>
