# 05 · Debugging (without AI)

[中文](../human-docs/05_DEBUGGING.md)

> **Audience**: Engineers reproducing issues locally who need logs or DB access.  
> **After reading**: Configure `RUST_LOG`, locate `app.db`, use `ProcessMessageError` `stage` to narrow scope.  
> **Time**: ~30 minutes.  
> **Next**: [06 Kernel learning path](06_KERNEL_LEARNING_PATH.md).

---

## RUST_LOG recipes

Default `info`; initialized by [`init_tracing`](../kernel/crates/oclive_kernel_host/src/lib.rs). Set **`RUST_LOG`** and restart the app.

**PowerShell**:

```powershell
$env:RUST_LOG = "info,oclive_chat=debug,oclive_plugin=debug,oclive_llm=debug"
npm run tauri:dev
```

**bash**:

```bash
RUST_LOG=info,oclive_chat=debug,oclive_plugin=debug npm run tauri:dev
```

Substring `json` enables JSON line format (see `OCLIVE_LOG_FORMAT`).

---

## tracing targets (explicit `target:` in repo)

| target | Typical use |
|--------|-------------|
| **`oclive_chat`** | `process_message` failures, turn orchestration |
| **`oclive_plugin`** | Directory plugin resolve, slot degradation |
| **`oclive_llm`** | LLM calls, Ollama / Remote |
| **`oclive_deep_link`** | `oclive://` deep link install |
| **`oclive_hotkey`** | Global hotkey registration |
| **`oclive_desktop`** | Desktop host integration |

**Module filter**: e.g. `RUST_LOG=oclive_kernel_host::domain::chat_engine=debug`.

**Refresh this table**:

```bash
rg 'target: "oclive' kernel/crates distros/desktop-tauri
```

---

## Log files

| Variable / mode | Effect |
|---------------|--------|
| **`OCLIVE_LOG_DIR`** | Also write rolling files |
| **`--api` headless** | Default `temp/oclive_api_app_data/logs/` |

---

## app.db and SQLite

| Item | Notes |
|------|-------|
| **Path** | `{app_data}/app.db` (Windows often under `%APPDATA%` app id) |
| **Doc** | [CONFIGURATION_FILES.md](../creator-docs-en/guides/CONFIGURATION_FILES.md) |
| **Open** | [DB Browser for SQLite](https://sqlitebrowser.org/) or `sqlite3` |
| **Migrations SSOT** | [`kernel/crates/oclive_kernel_host/migrations/`](../kernel/crates/oclive_kernel_host/migrations/) |

Common tables: `role_runtime` (key **`srid`**), `chat_messages`, `long_term_memory` (decoupled from chat storage).

---

## ProcessMessageError and `stage`

Errors look like `send_message[{stage}]: …`. Example stages:

- `ensure_role_loaded`
- `startup_health`
- `dual_core_experimental` (only with `dual_core` feature)

Definition: [`message_error.rs`](../kernel/crates/oclive_kernel_host/src/domain/chat_engine/message_error.rs)

Search logs: `oclive_chat` target + `stage` name in error text.

---

## Skip probes (local speed)

| Variable | Effect |
|----------|--------|
| `OCLIVE_SKIP_STARTUP_HEALTH` | Skip first-turn health check |
| `OCLIVE_SKIP_LLM_STARTUP_PROBE` | Skip LLM startup probe |
| `OCLIVE_HTTP_API_MOCK_LLM=1` | HTTP smoke mock LLM |

---

## Checklist

- [ ] Can filter `RUST_LOG` to `oclive_chat` debug only
- [ ] Know `app.db` is under `{app_data}`, not the repo
- [ ] `send_message[ensure_role_loaded]` → check role load path

---

## Deep links

- [ERROR_CODES](../creator-docs-en/getting-started/ERROR_CODES.md)
- [USER_MANUAL § troubleshooting](../creator-docs-en/getting-started/USER_MANUAL.md)
