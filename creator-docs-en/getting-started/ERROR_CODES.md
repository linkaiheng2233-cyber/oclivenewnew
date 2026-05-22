# Error codes and quick triage

Audience: users and developers. Goal: **self-serve first**, then file a high-quality issue.

**Normative machine `code` + JSON shape** (naming, transports, JSON-RPC boundary): **[KERNEL_ERROR_CODE_CONVENTION.md](KERNEL_ERROR_CODE_CONVENTION.md)**.

---

## 1) Runtime HTTP API (`/chat`) error body (same as kernel / Tauri)

On `POST /chat` failures, JSON uses **`error` = `KernelErrorBody`** (same fields as the JSON string Tauri `invoke` may return):

- `code`: **`SCREAMING_SNAKE_CASE`**, aligned with [`AppError::code`](../../crates/oclive_kernel_runtime/src/error.rs).
- `message`: kernel `Display` (default English technical text); shells localize via `code`.
- `hint`: optional next step; HTTP may attach extra hints for editor try-chat.

Example:

```json
{
  "error": {
    "code": "INVALID_ROLE_PATH",
    "message": "role_path is not a directory: D:\\roles\\demo",
    "hint": "Pass an absolute path to the role directory that contains manifest.json"
  }
}
```

| code | Meaning | Common cause | What to try |
|------|---------|--------------|-------------|
<!-- code:EMPTY_MESSAGE -->
| `EMPTY_MESSAGE` | Empty message | Only whitespace / newlines | Type at least one visible character |
<!-- code:INVALID_ROLE_PATH -->
| `INVALID_ROLE_PATH` | Path is not a directory | Typo, pointed at a file | Pass `{roles_root}/{role_id}` as an absolute directory |
<!-- code:ROLE_NOT_FOUND -->
| `ROLE_NOT_FOUND` | Pack invalid or missing | Missing `manifest` / `settings` or bad structure | Run full checks in the pack editor; same `code` as Tauri `load_role` |
<!-- code:LLM_ERROR -->
| `LLM_ERROR` | LLM provider failure | Ollama down, model not pulled, remote timeout | Start Ollama and `ollama pull`; verify model name in role pack; use `OCLIVE_HTTP_API_MOCK_LLM=1` for bench | See §1.5 |
<!-- code:DB_ERROR -->
| `DB_ERROR` | Database error | Corrupt `app.db`, disk full, migration failure | Ensure data dir is writable; see `Database error` in logs |
<!-- code:ROLE_RUNTIME_NOT_READY -->
| `ROLE_RUNTIME_NOT_READY` | Role not loaded | No `load_role` / no role selected in UI | Load a role before chatting |
<!-- code:STARTUP_HEALTH_FAILED -->
| `STARTUP_HEALTH_FAILED` | Startup health failed | Slots, manifest, DB ping, LLM probe | Run in-app environment diagnostics; see `startup_health` logs |
<!-- code:LOAD_ROLE_TASK_PANIC -->
| `LOAD_ROLE_TASK_PANIC` | Load task panicked | Rare | File an issue with logs |
<!-- code:IO_ERROR -->
| `IO_ERROR` | File or disk I/O failed | Missing path, permissions, disk full | Ensure role dir and `app_data` are writable |
<!-- code:ROLE_PACK_EXISTS -->
| `ROLE_PACK_EXISTS` | Import target exists without overwrite | Duplicate role id | Remove old dir or enable overwrite on import |
<!-- code:INVALID_PARAMETER -->
| `INVALID_PARAMETER` | Invalid request parameter | Empty `role_id`, unknown module, v2 pack without `slot_registry` | See API docs and [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) |
<!-- code:HIGH_RISK_CAPABILITY_NOT_GRANTED -->
| `HIGH_RISK_CAPABILITY_NOT_GRANTED` | High-risk capability not granted | MCP stdio/http or directory `process:spawn` / `network:*` | Grant in plugin manager / MCP settings |
<!-- code:REMOTE_SERVICE_UNAVAILABLE -->
| `REMOTE_SERVICE_UNAVAILABLE` | Remote backend down and fallback disabled | Bad URL, timeout, `remote_fallback_to_builtin` off | Fix `OCLIVE_REMOTE_*` or enable fallback / use local backend |
<!-- code:SERDE_ERROR -->
| `SERDE_ERROR` | JSON parse/serialize failure | Corrupt config or non-JSON body | `pack validate`; check manifests |
<!-- code:UNKNOWN_ERROR -->
| `UNKNOWN_ERROR` | Unclassified internal error | Catch-all | File issue with logs; UI may show `UNKNOWN_WITH_CODE` |

**Transactions**: [`AppError::TransactionError`](../../crates/oclive_kernel_runtime/src/error.rs) uses a **dynamic** `code` string; not listed above. `oclive explain` covers static `AppError` variants and HTTP supplement codes only.

### 1.5) First install: Ollama and role paths (subset)

| Symptom | Common cause | Next step |
|---------|--------------|-----------|
| Chat fails; logs/UI mention **Ollama** unreachable | Daemon not installed/running, wrong port, model not pulled | Install/start [Ollama](https://ollama.com); run `ollama list` / `ollama pull <model>`; verify model names in pack or env |
| **`INVALID_ROLE_PATH` / `ROLE_NOT_FOUND`, etc.** | **`OCLIVE_ROLES_DIR`** not the **parent** of role folders, or missing `manifest.json` under the role dir | Point the var at the **roles root**; use [oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher) or [CREATOR_WORKFLOW.md](CREATOR_WORKFLOW.md) |
| **Directory not writable** (OS or Rust I/O errors) | AV blocking, permissions, read-only media | Use a writable path; avoid read-only shares for `app.db` (see [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)) |
| **Settings → General → Environment check** | Quick confirmation of Ollama, roles root, app data dir | **Ctrl+Shift+S** → General → **Run check** (Tauri `run_environment_diagnostics`) |

### 1.6) Offline / weak network (A2.3)

| Scenario | Runtime behavior | What to try |
|----------|------------------|-------------|
| **Community plugin index** (workbench → Community index → **Sync online index**) | If online `plugins.json` fails, the host reads **`plugin_index_cache.json`** under app data, returns `offlineMode=true` and a `warning` string (technical); UI + toast use i18n for the headline | Check network, proxy, firewall; set **`OCLIVE_PLUGIN_INDEX_URL`** to a reachable mirror; sync again when online |
| **Never synced successfully** | Cache may be empty; list stays empty | Complete one successful sync, or install from folder / zip offline |
| **Ollama / Remote LLM** | Chat path returns **`KernelErrorBody` JSON** (e.g. `LLM_ERROR`); very old logs may still show `[CODE]` prefixes | See **§1.5** and frontend `apiErrors` mapping |
| **Extra Tauri hints (JSON; legacy `[CODE]` fallback)** | First-chat startup checks or missing runtime row | `STARTUP_HEALTH_FAILED`: manifest, slots, DB; `ROLE_RUNTIME_NOT_READY`: call `load_role` / pick the role in UI; directory-slot codes in `apiErrors` |

If the GUI still shows raw English backend strings, track under **A6** cleanup; unknown machine codes fall back to **`apiErrors.UNKNOWN_WITH_CODE`**. Self-serve with the table above first.

### 1.7) Crash reporting & privacy (A3)

| Topic | Notes |
|-------|--------|
| **When it runs** | **`@sentry/vue`** may initialize only if **`VITE_SENTRY_DSN`** was set at **frontend build** time; no DSN → **no telemetry**. |
| **What is sent** | **Uncaught Vue** errors; **not** chat text; Rust still relies mainly on **local logs**. |
| **User opt-out** | **Settings → General** shows **Crash diagnostics (Sentry)** on DSN-enabled builds; **Disable crash reporting** writes **`localStorage`** key **`oclive.telemetry.sentryOptOut`** (`1` = opted out) and closes the client; clearing it requires an **app restart** to resume. |
| **More detail** | Root [README.md](../../README.md) / [README.en.md](../../README.en.md) (Observability); closure notes [EN](../../handoff/A3_CLOSURE_SUMMARY.en.md) / [ZH](../../handoff/A3_CLOSURE_SUMMARY.md). |

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
