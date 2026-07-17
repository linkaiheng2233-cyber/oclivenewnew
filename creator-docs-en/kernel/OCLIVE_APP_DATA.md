# OCLIVE_APP_DATA — cross-host data directory

[中文](../../creator-docs/kernel/OCLIVE_APP_DATA.md)

**Audience**: VS Code extension, launcher, `oclive-kernel-server`, CI maintainers.

---

## Canonical brand path

| Platform | Default `OCLive/data` |
|----------|----------------------|
| Windows | `%LOCALAPPDATA%/OCLive/data` |
| macOS | `~/Library/Application Support/OCLive/data` |
| Linux | `$XDG_DATA_HOME/OCLive/data` or `~/.local/share/OCLive/data` |

Parallel to `%LOCALAPPDATA%/OCLive/runtime` (shared kernel binary); data and runtime are separate.

SQLite SSOT: `{OCLIVE_APP_DATA}/app.db` (via `resolve_db_path`).

---

## Environment variables

| Variable | Semantics |
|----------|-----------|
| `OCLIVE_APP_DATA` | **Explicit** app data root; preferred on spawn / desktop |
| `OCLIVE_USE_CANONICAL_APP_DATA=1` | Headless `--api` uses brand dir when `OCLIVE_APP_DATA` unset |
| `OCLIVE_API_USE_TEMP_APP_DATA=1` | Force temp DB (**CI / OOCP default**) |
| `OCLIVE_SKIP_APP_DATA_MIGRATION=1` | Skip Tauri legacy → canonical one-time copy (tests) |
| `OCLIVE_API_TOKEN` | Required headless HTTP API access token; only `/health` remains public |
| `OCLIVE_API_ALLOW_UNAUTHENTICATED=1` | Explicit unauthenticated escape hatch for isolated local development only; never for production or persistent data |

---

## Headless `--api` resolution order

1. `OCLIVE_APP_DATA` set → persistent  
2. `OCLIVE_API_USE_TEMP_APP_DATA=1` → temp (deleted on exit)  
3. `OCLIVE_USE_CANONICAL_APP_DATA=1` → brand dir  
4. Else → temp (historical CI behavior)

---

## One-time migration

When canonical `app.db` **does not exist** but Tauri legacy path has `app.db`:

- **Copy** (not move) entire legacy `app_data` to `OCLive/data`  
- Write `.migrated_from_tauri` marker  
- On failure, **do not** open DB as writer  

Legacy path (`identifier: com.oclivenewnew.app`):

- Windows: `%APPDATA%/com.oclivenewnew.app`  
- macOS: `~/Library/Application Support/com.oclivenewnew.app`  
- Linux: `~/.local/share/com.oclivenewnew.app`

CLI: `cargo run -p oclive-cli -- migrate-app-data [--target PATH] [--dry-run]`

---

## Single writer

Only one process opens `app.db` as writer; other distros **attach** `GET http://127.0.0.1:8420/health`.

Desktop Phase 2 (**spawn-only thin client**): attach or spawn `oclive-kernel-server`; **no** in-process DB writes. Chat via `POST /chat` — see [DISTRO_KERNEL_LIFECYCLE.md](DISTRO_KERNEL_LIFECYCLE.md).

---

## Related

- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
- [VSCODE_DISTRIBUTION.md](../../handoff/vscode/VSCODE_DISTRIBUTION.md)
- [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md)
