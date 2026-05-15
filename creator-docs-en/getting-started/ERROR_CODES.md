# Error codes and quick triage

Audience: users and developers. Goal: **self-serve first**, then file a high-quality issue.

---

## 1) Runtime HTTP API (`/chat`) errors

Example response body:

```json
{
  "error": {
    "code": "invalid_role_path",
    "message": "role_path is not a directory: D:\\roles\\demo",
    "hint": "Pass an absolute path to the role directory that contains manifest.json"
  }
}
```

| code | Meaning | Common cause | What to try |
|------|---------|--------------|-------------|
| `empty_message` | Empty message | Only whitespace / newlines | Type at least one visible character |
| `invalid_role_path` | Path is not a directory | Typo, pointed at a file | Pass `{roles_root}/{role_id}` as an absolute directory |
| `load_role_failed` | Failed to load role dir | Missing `manifest` / `settings` or bad structure | Run full checks in the pack editor; verify tree |
| `chat_engine_failed` | Chat engine internal failure | Sidecar timeout, model down, runtime state | Check logs `oclive_chat` / `oclive_plugin` |

### 1.5) First install: Ollama and role paths (subset)

| Symptom | Common cause | Next step |
|---------|--------------|-----------|
| Chat fails; logs/UI mention **Ollama** unreachable | Daemon not installed/running, wrong port, model not pulled | Install/start [Ollama](https://ollama.com); run `ollama list` / `ollama pull <model>`; verify model names in pack or env |
| **`invalid_role_path` / `load_role_failed`** | **`OCLIVE_ROLES_DIR`** not the **parent** of role folders, or missing `manifest.json` under the role dir | Point the var at the **roles root**; use [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher) or [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) |
| **Directory not writable** (OS or Rust I/O errors) | AV blocking, permissions, read-only media | Use a writable path; avoid read-only shares for `app.db` (see [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)) |
| **Settings → General → Environment check** | Quick confirmation of Ollama, roles root, app data dir | **Ctrl+Shift+S** → General → **Run check** (Tauri `run_environment_diagnostics`) |

### 1.6) Offline / weak network (A2.3)

| Scenario | Runtime behavior | What to try |
|----------|------------------|-------------|
| **Community plugin index** (workbench → Community index → **Sync online index**) | If online `plugins.json` fails, the host reads **`plugin_index_cache.json`** under app data, returns `offlineMode=true` and a `warning` string (technical); UI + toast use i18n for the headline | Check network, proxy, firewall; set **`OCLIVE_PLUGIN_INDEX_URL`** to a reachable mirror; sync again when online |
| **Never synced successfully** | Cache may be empty; list stays empty | Complete one successful sync, or install from folder / zip offline |
| **Ollama / Remote LLM** | Chat path returns `[CODE]`-style errors on timeout/unavailable | See **§1.5** and frontend `apiErrors` mapping |
| **Extra Tauri `[CODE]` hints** | First-chat startup checks or missing runtime row | `[STARTUP_HEALTH_FAILED]`: manifest, slots, DB; `[ROLE_RUNTIME_NOT_READY]`: call `load_role` / pick the role in UI; `[PLUGIN_BACKENDS_DIRECTORY_SLOT]`: directory slot without `directory_plugins` id |

If the GUI still shows raw English backend strings, track under **A3.2 / A6** cleanup; self-serve with the table above first.

---

## 2) Remote JSON-RPC errors (sidecar convention)

The host logs `code` / `message` / `data` and may fall back to built-ins. Suggested JSON-RPC codes:

| code | name | Meaning |
|------|------|---------|
| `-32700` | `parse_error` | Body is not valid JSON |
| `-32600` | `invalid_request` | JSON-RPC envelope shape is wrong |
| `-32601` | `method_not_found` | Method does not exist |
| `-32602` | `invalid_params` | Missing params or wrong types |
| `-32603` | `internal_error` | Sidecar internal error |
| `-32010` | `plugin_timeout` | Upstream call timed out |
| `-32011` | `auth_failed` | Bad token or insufficient permission |
| `-32012` | `rate_limited` | Rate limited |
| `-32013` | `upstream_unavailable` | Upstream unavailable |

---

## 3) Minimum info for an issue

1. `error.code`, `error.message`, `error.hint` (if any)  
2. What you were doing (API probe / send message / auto-start)  
3. Environment variable **names** only (no secrets):  
   - `OCLIVE_LLM_BACKEND` (`ollama` / `remote`; launcher may override pack `plugin_backends.llm`)  
   - `OCLIVE_REMOTE_PLUGIN_URL`  
   - `OCLIVE_REMOTE_LLM_URL`  
   - `OCLIVE_REMOTE_PLUGIN_TIMEOUT_MS`  
   - `OCLIVE_REMOTE_LLM_TIMEOUT_MS`
   - `OCLIVE_PLUGIN_INDEX_URL` (community `plugins.json` mirror; offline notes in **§1.6**)
4. Short log excerpt (`oclive_chat` / `oclive_plugin`)

---

[中文原文](../../creator-docs/getting-started/ERROR_CODES.md)
