# Distro kernel lifecycle (SSOT)

Cross-host **single writer** model: one `oclive-kernel-server` on `127.0.0.1:8420` owns `%LOCALAPPDATA%/OCLive/data/app.db`. Desktop, VS Code, and future distros are **HTTP clients** only.

## Discovery SSOT (Rust)

Binary tier scores and promotion threshold live in:

`crates/oclive_kernel_runtime/src/kernel_discovery.rs`

| Constant | Value | Tier |
|----------|-------|------|
| `PROMOTE_SCORE_THRESHOLD` | 88 | promote dev → shared runtime |
| `SCORE_ENV` | 100 | `OCLIVE_KERNEL_BINARY` |
| `SCORE_DEV_FULL_DEBUG` | 95 | `oclivenewnew-tauri --api` debug |
| `SCORE_DEV_FULL_RELEASE` | 94 | release |
| `SCORE_DEV_HEADLESS_DEBUG` | 90 | `oclive-kernel-server` debug |
| `SCORE_DEV_HEADLESS_RELEASE` | 89 | release |
| `SCORE_SHARED` | 88 | `%LOCALAPPDATA%/OCLive/runtime/oclive-kernel-server` |
| `SCORE_SETTINGS` | 85 | user settings path |
| `SCORE_BUNDLED` | 50 | extension / bundle `bin/` |

VS Code `src/discovery.ts` imports the same numeric values (comment-linked to Rust).

## Startup order (all distros)

1. `GET http://127.0.0.1:8420/health` — attach if OK
2. Else discover binary → optional promote (score ≥ 88) → spawn with env:
   - `OCLIVE_APP_DATA` = `%LOCALAPPDATA%/OCLive/data`
   - `OCLIVE_USE_CANONICAL_APP_DATA=1`
   - `OCLIVE_ROLES_DIR`
3. Poll health up to 30×500ms
4. Watchdog every ~20s; on loss emit upstream lost → respawn (desktop) or reconnect command

## Desktop (`src-tauri/src/kernel_lifecycle/`)

- `ensure.rs` — attach-first bring-up
- `spawn.rs` — child process + env
- `connection.rs` — `DesktopKernelMode`: `attached` | `spawned` | `offline` | `reconnecting`
- `watchdog.rs` — health + respawn; Tauri events `kernel:upstream_lost` / `kernel:reconnected`

Desktop **does not** bind in-process `:8420` or open canonical `app.db` for writes. UI shell uses in-memory state for directory plugins only; P0 IPC proxies to HTTP.

## HTTP routes (kernel)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness + schema metadata |
| POST | `/chat` | Turn |
| GET | `/role_info` | Full `RoleInfo` |
| GET | `/role_snapshot` | Lightweight UI poll |
| POST | `/role/load` | Initialize role runtime |
| GET | `/chat/sessions` | Session list |
| GET | `/chat/messages` | Message page |

## E2E scenarios

Run from repo root (requires built `oclive-kernel-server`):

```bash
node scripts/e2e-distro-kernel.mjs --scenario spawn
node scripts/e2e-distro-kernel.mjs --scenario attach
node scripts/e2e-distro-kernel.mjs --scenario role-snapshot
```

See also `scripts/e2e-cross-host-memory.mjs` for canonical app-data chat smoke.

## Distro capability profile (P1 contract)

Each distribution may ship `distro.oclive.toml` at its install root (alongside bundled `bin/`). The file declares **capability ceiling** and defaults for when that host connects to the kernel (P4: `HostProfile` merge). It is **not** role-pack `settings.json`.

- Spec: [DISTRO_CAPABILITY_PROFILE.md](./DISTRO_CAPABILITY_PROFILE.md)
- Examples: `examples/distro-profiles/desktop.oclive.toml`, `examples/distro-profiles/vscode.oclive.toml`

## Logical seed (bundled binary)

- **Definition**: The distro-bundled **full** `oclive-kernel-server` binary (`SCORE_BUNDLED = 50`) acts as a **logical seed** on first install—not a smaller “seed build.”
- **Lifecycle**: First launch spawns bundled when nothing listens on `:8420` and shared runtime is empty. When a stronger binary is discovered (manifest newer or score ≥ 88), the **host** runs `promote_with_backup` into `%LOCALAPPDATA%/OCLive/runtime/` (P3a). Later hosts attach to the shared copy.
- **Not in scope**: In-process seed self-upgrade or connection handoff (hosts coordinate; single writer on `:8420`).

Kernel binary manifest and `/health` fields: P2a (`KernelBinaryManifest`, `--version-json`).

## Deferred (post P1–P4)

| Item | Description |
|------|-------------|
| **P2b** | Richer per-distro manifest fields (hardware SKU, feature flags beyond `feature_set`) |
| **P3b** | In-process kernel self-upgrade / connection handoff (v1: **host-coordinated** `promote_with_backup` only) |

## Related

- [DISTRO_CAPABILITY_PROFILE.md](./DISTRO_CAPABILITY_PROFILE.md)
- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
- VS Code Phase 1: `oclive-vscode/AGENTS.md`
