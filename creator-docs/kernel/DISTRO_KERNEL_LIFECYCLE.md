# Distro kernel lifecycle (SSOT)

Cross-host **single writer** model: one `oclive-kernel-server` on `127.0.0.1:8420` owns `%LOCALAPPDATA%/OCLive/data/app.db`. Desktop, VS Code, and future distros are **HTTP clients** only.

## Architecture: decision up, execution down

**Policy (Rust SSOT, no I/O)** lives in `oclive_kernel_runtime`. **Hosts** (desktop, VS Code, `oclive-cli`) discover binaries, call the policy, then **execute** attach / spawn / replace locally.

```text
GET /health (+ kernel_manifest, distro_id)
        │
        ▼
discover_spawn_kernel_candidates()   ← kernel_discovery.rs
        │
        ▼
resolve_kernel_action()              ← kernel_strategy.rs
        │
        ├── Attach
        ├── ReplaceAndAttach(candidate)
        ├── SpawnBest(candidate)
        └── FallbackBundled(candidate)
        │
        ▼
Host executes: kill :8420 · promote · spawn · status UI
```

| Layer | Module / entry | Role |
|-------|----------------|------|
| **Discovery** | `kernel_discovery.rs` | Tier scores, candidate list (unchanged) |
| **Manifest / compare** | `kernel_manifest.rs` | `KernelBinaryManifest`, `cmp_for_capability` (feature_set → semver → builtAt) |
| **Policy** | `kernel_strategy.rs` | `resolve_kernel_action`, `KernelActionKind`, `KernelActionPlan`, `ReplaceReason` |
| **Profile requirements** | `kernel_distro_profile.rs` | `DistroProfileRequirements`, `ActiveProfileSummary`, `profile_satisfies_caller` |
| **Policy input** | `kernel_policy_input.rs` | `build_resolve_plan`, `PolicyContext`, `PolicyResolution` (CLI + desktop SSOT) |
| **Port ops** | `kernel_port_ops.rs` | `find_listener_pids`, `terminate_listeners_on_port` (CLI + desktop) |
| **Health DTOs** | `oclive_kernel_types::models::kernel` | `KernelHealthJson`, `AttachReason`, `ReplaceReason`, `ProfileCompat` |
| **Promote / backup** | `kernel_runtime_ops.rs` | `promote_with_backup`, shared runtime |
| **CLI** | `oclive kernel ensure` | Same policy + optional execution (`--plan-only`, `--json`) |
| **VS Code** | `oclive-vscode/src/kernelStrategy.ts` | `oclive-cli kernel ensure --plan-only`; spawn/kill in extension |
| **Desktop** | `distros/desktop-tauri/src/kernel_lifecycle/policy.rs` | Direct `resolve_kernel_action`; spawn/kill in Tauri; **attach-first fallback** on failure |

Hosts **no longer** each embed their own capability-compare logic. They **call shared policy** and run side effects (process management, distro env on spawn).

## Discovery SSOT (Rust)

Binary tier scores and promotion threshold live in:

`kernel/crates/oclive_kernel_runtime/src/kernel_discovery.rs`

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

VS Code `src/discovery.ts` mirrors numeric tiers (comment-linked to Rust). **Spawn ordering** is decided by `resolve_kernel_action`, not score alone.

## Startup order (profile-aware attach + bundled-first spawn)

**Two phases** — do not conflate them:

| Phase | Question | SSOT |
|-------|----------|------|
| **A · Running process** | Is `:8420` healthy and profile-compatible? | `resolve_kernel_action` → attach / replace |
| **B · Cold spawn** | Which binary to start when nothing listens? | **Caller bundled first** → shared fallback → dev (see [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) §2) |

### A — When `/health` succeeds

1. `GET http://127.0.0.1:8420/health` with `Accept: application/json` → read `kernel_manifest`, optional `distro_id` / `distro_profile_hash` / **`active_profile_summary`**
2. Parse caller **`DistroProfileRequirements`** from `distro.oclive.toml` (or built-in defaults per `distro_id`)
3. Discover local headless candidates → `resolve_kernel_action` with running manifest + profile summary + caller requirements
4. **Attach** when running profile **satisfies** caller — `attach_reason: profile_compatible` (even if a stronger binary exists locally; **do not** auto-replace for “fullest kernel”)
5. **Replace** (`replace_reason: profile_mismatch`) when running profile conflicts with caller and replace is allowed — **restart + new `OCLIVE_DISTRO_*` env** (no in-process profile hot switch)
6. **Pinned** (`OCLIVE_KERNEL_BINARY`) → always attach; `kernel_pinned_profile_mismatch` when profile conflicts (UI hint, no replace)
7. **`binary_upgrade` replace** — **Freeze (product)**; Rust path retained for legacy kernels without `active_profile_summary`. Default hosts **must not** proactively replace a healthy kernel because a local dev/shared binary scores higher. Opt-in: `OCLIVE_ALLOW_BINARY_UPGRADE=1` (see KERNEL_SCHEDULER_RESCOPE §3.2)

**Single writer**: one process on `:8420` owns `app.db`. VS Code and Desktop **attach** when profile-compatible; profile conflict → **replace** (kill `:8420`, respawn), not parallel multi-port kernels.

### B — When offline (spawn)

**Target order** (product; **K-SCHED-05** — `discover_spawn_kernel_candidates` still sorts by discovery score today):

1. **Caller bundled** `oclive-kernel-server` (install root / extension `bin/` / Tauri `resources/bin/`)
2. **Shared fallback** — `%LOCALAPPDATA%/OCLive/runtime/oclive-kernel-server` (full build; same `OCLIVE_APP_DATA` + `OCLIVE_DISTRO_PROFILE` + `OCLIVE_ROLES_DIR` → plugins under `{app_data}/distros/chat-pro/plugins/` **reuse**, no copy)
3. **Dev builds** — monorepo / `OCLIVE_KERNEL_BINARY` pin (developers only)

If bundled spawn fails (crash, bad manifest, health timeout) → retry shared with **unchanged** distro env. **Fault attribution**: bundled fails but shared + same plugins works → suspect **distro binary**; both fail → suspect **plugins / config / app_data**.

Poll health after spawn; watchdog / reconnect reuse phase **A** policy.

**Profile on spawn**: hosts pass `OCLIVE_DISTRO_ID` + `OCLIVE_DISTRO_PROFILE` so the new kernel loads the caller's profile.

### `/health` active profile (runtime truth)

**发行版能力以 `active_profile_summary` 为准，不是 `kernel_manifest.feature_set`。** Sidecar manifest 描述二进制构建元数据（版本、feature_set 枚举）；调度 attach/replace 与宿主 UI 展示的「当前发行版能力」均读取 **`HostProfile` 运行时摘要**（`enabled_modules` / `disabled_modules`、prompt/post_process profile、interaction 默认值等）。勿用 manifest 的 `feature_set: full|bundled` 推断六槽或 agent/complex_emotion 开关。

`active_profile_summary` is built from the loaded **`HostProfile`** (`host_profile.active_profile_summary()`), not by re-parsing TOML at health time. When no distro profile is loaded (`distro_id = default`, no `OCLIVE_DISTRO_PROFILE`), the field is omitted.

### Profile compatibility (tightened)

When `active_profile_summary` is **missing**: `distro_id` match alone is **not** enough. Requires matching `distro_profile_hash`, or falls through to **Unknown** → binary compare path.

### Policy priority (healthy kernel — phase A only)

| Order | Condition | Action |
|-------|-----------|--------|
| 1 | `kernel_pinned` | Attach (warn on profile mismatch) |
| 2 | Profile compatible (summary / hash / satisfies caller) | Attach |
| 3 | Profile incompatible + `allow_replace_running` | ReplaceAndAttach (`profile_mismatch`) |
| 4 | Profile unknown + binary weaker + **`OCLIVE_ALLOW_BINARY_UPGRADE=1`** | ReplaceAndAttach (`binary_upgrade`) — **not default** |
| 5 | Otherwise | Attach (`running_kernel_ok`) |

**Spawn failure (graded)**: Policy spawn failure → **profile-aware attach-only** (still runs `build_resolve_plan`; only `Attach` allowed) → retry next spawn candidate (bundled → shared → bundled tier degraded). VS Code without `oclive-cli` attaches only when `/health` summary looks VS Code–compatible (agent disabled).

## Desktop (`distros/desktop-tauri/src/kernel_lifecycle/`)

- `policy.rs` — `build_resolve_plan` + spawn/replace execution
- `ensure.rs` — entry; graded fallback (profile-aware attach → legacy)
- `port_ops.rs` — terminate `:8420` listeners for replace
- `spawn.rs` — child process + `OCLIVE_DISTRO_ID` / `OCLIVE_DISTRO_PROFILE` via `HostProfile`
- `connection.rs` — `DesktopKernelMode`; optional `degraded` / `status_message` on status DTO
- `reconnect.rs` — same policy path as ensure
- `watchdog.rs` — health + respawn; Tauri events `kernel:upstream_lost` / `kernel:reconnected`

Desktop **does not** bind in-process `:8420` or open canonical `app.db` for writes. UI shell uses in-memory state for directory plugins only; P0 IPC proxies to HTTP.

## VS Code (`oclive-vscode`)

- `kernelStrategy.ts` — locates `oclive-cli`, runs `kernel ensure --json --plan-only --distro vscode`
- `kernelClient.ts` — executes plan (attach / spawn / replace); `kernelPort.ts` for kill
- Requires built `oclive-cli` in monorepo dev; ships `distro.oclive.toml` on spawn only

## CLI

```bash
cargo run -p oclive-cli -- kernel ensure --json --plan-only --distro vscode --path .
OCLIVE_API_TOKEN="replace-with-a-long-random-token" cargo run -p oclive-cli -- kernel ensure --distro desktop --roles-dir ./roles
```

实际 spawn 长生命周期 kernel 时必须提供 `OCLIVE_API_TOKEN`；CLI 不会生成一个调用方无法获得的临时 token。`GET /health` 可无令牌探活，其他 HTTP 路由发送 `x-oclive-api-token`。

See also `kernel status|promote|rollback`.

## HTTP routes (kernel)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness + `kernel_manifest` + optional `distro_id` / `distro_profile_hash` / `active_profile_summary` (`Accept: application/json`) |
| POST | `/chat` | Turn |
| GET | `/role_info` | Full `RoleInfo` |
| GET | `/role_snapshot` | Lightweight UI poll |
| POST | `/role/load` | Initialize role runtime |
| GET | `/chat/sessions` | Session list |
| GET | `/chat/messages` | Message page |
| POST | `/chat/storage` | Chat storage proxy ops |
| GET | `/time/state` | Time state |
| POST | `/time/jump` | Jump time |
| POST | `/scene/switch` | Switch scene |
| GET | `/user_identity/state` | User identity state |
| POST | `/user_identity/set` | Set global identity |
| POST | `/user_identity/scene_set` | Set per-scene identity |
| POST | `/scene/user_presence` | Set user presence scene |
| POST | `/event/create` | Create event |
| GET | `/high_risk/grants` | List high-risk grants |
| POST | `/high_risk/grant` | Grant high-risk capability |
| POST | `/high_risk/revoke` | Revoke high-risk capability |
| POST | `/bridge/dispatch` | Bridge command dispatch |
| POST | `/llm/reload` | Reload user LLM env |
| GET | `/llm/user_settings` | User LLM settings (`role_id`, optional `session_id`) |
| POST | `/llm/user_settings` | Save user LLM settings → `RoleInfo` |
| GET | `/llm/ollama_models` | List Ollama models (optional `ollama_base_url`) |
| POST | `/llm/session_model` | Session Ollama model override |

## E2E scenarios

Run from repo root (requires built `oclive-kernel-server`):

```bash
node scripts/e2e-distro-kernel.mjs --scenario spawn
node scripts/e2e-kernel-profile.mjs
```

See also `scripts/e2e-cross-host-memory.mjs` for canonical app-data chat smoke.

## Distro capability profile (P1 contract)

Each distribution may ship `distro.oclive.toml` at its install root (alongside bundled `bin/`). The file declares **distro defaults** and optional **`[plugin_backends]` override** for when that host **spawns** the kernel (P4: `HostProfile`). It is **not** role-pack blueprint / legacy `settings.json`. Merge semantics: see [DISTRO_CAPABILITY_PROFILE.md](./DISTRO_CAPABILITY_PROFILE.md) §4.

- Spec: [DISTRO_CAPABILITY_PROFILE.md](./DISTRO_CAPABILITY_PROFILE.md)
- Examples: `examples/distro-profiles/desktop.oclive.toml`, `examples/distro-profiles/vscode.oclive.toml`

On **attach**, the running kernel’s `distro_id` from `/health` reflects whoever **spawned** that process; hosts log mismatches but do not auto-switch profile on attach alone (v1).

Running side: `ActiveProfileSummary` on `/health` lists effective enabled/disabled modules. Policy may **replace** when profile requirements conflict (restart with caller `OCLIVE_DISTRO_*` env).

## Bundled binary vs shared fallback

| Term | Meaning |
|------|---------|
| **发行版 bundled 内核** | Full `oclive-kernel-server` shipped with the distro install / VSIX `bin/` — **default spawn candidate** (discovery `SCORE_BUNDLED = 50` is tier label only, not spawn priority) |
| **shared 兜底核** | `%LOCALAPPDATA%/OCLive/runtime/` copy — used when bundled fails or is absent |
| **logical seed** (legacy doc term) | Same artifact as bundled today; prefer **「发行版 bundled 内核」** in new text |

- **First launch**: nothing on `:8420` → spawn **caller bundled**; on failure → shared with same `OCLIVE_APP_DATA` / profile / roles.
- **`promote_with_backup` (P3a)**: **Developer / maintenance** — copy a local dev build into shared runtime (`score ≥ 88`). **Not** the default product path for end users; see [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) §3.2.
- **Not in scope**: In-process self-upgrade (P3b deferred); per-distro **trimmed** kernel binaries (Deferred).

Kernel binary manifest and `/health` fields: P2a (`KernelBinaryManifest`, `--version-json`).

## Deferred (post P1–P4)

| Item | Description |
|------|-------------|
| **P2b** | Richer per-distro manifest fields (hardware SKU, feature flags beyond `feature_set`) |
| **P3b** | In-process kernel self-upgrade / connection handoff (v1: **host-coordinated** `promote_with_backup` only) |
| **`oclive-runtimed`** | Optional supervisor on `:8420` (Phase 3); may reuse same policy crate |

## Related

- [DISTRO_CAPABILITY_PROFILE.md](./DISTRO_CAPABILITY_PROFILE.md)
- [DISTRO_DEFAULT_PLUGINS.md](./DISTRO_DEFAULT_PLUGINS.md)
- [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) — bundled-first spawn · single `:8420` · `binary_upgrade` freeze
- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
- VS Code: `oclive-vscode/AGENTS.md`, `oclive-vscode/docs/VSCODE_DISTRIBUTION.md`
