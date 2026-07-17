# Distro kernel lifecycle (SSOT)

[中文](../../creator-docs/kernel/DISTRO_KERNEL_LIFECYCLE.md)

Cross-host **single writer** model: one `oclive-kernel-server` on `127.0.0.1:8420` owns `%LOCALAPPDATA%/OCLive/data/app.db`. Desktop, VS Code, and future distros are **HTTP clients** only.

## Architecture: decision up, execution down

**Policy (Rust SSOT, no I/O)** lives in `oclive_kernel_runtime`. **Hosts** discover binaries, call policy, then **execute** attach / spawn / replace locally.

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

| Layer | Module | Role |
|-------|--------|------|
| Discovery | `kernel_discovery.rs` | Tier scores, candidate list |
| Manifest | `kernel_manifest.rs` | `KernelBinaryManifest`, `cmp_for_capability` |
| Policy | `kernel_strategy.rs` | `resolve_kernel_action`, `KernelActionPlan` |
| Profile | `kernel_distro_profile.rs` | `DistroProfileRequirements`, `profile_satisfies_caller` |
| Policy input | `kernel_policy_input.rs` | `build_resolve_plan` (CLI + desktop SSOT) |
| Port ops | `kernel_port_ops.rs` | `terminate_listeners_on_port` |
| Health DTOs | `oclive_kernel_types::models::kernel` | `KernelHealthJson`, `ActiveProfileSummary` |
| CLI | `oclive kernel ensure` | Same policy + optional execution |
| VS Code | `oclive-vscode/src/kernelStrategy.ts` | `oclive-cli kernel ensure --plan-only` |
| Desktop | `distros/desktop-tauri/src/kernel_lifecycle/policy.rs` | Direct policy; attach-first fallback |

Hosts **no longer** each embed capability-compare logic.

## Discovery SSOT (Rust)

`kernel/crates/oclive_kernel_runtime/src/kernel_discovery.rs`

| Constant | Value | Tier |
|----------|-------|------|
| `PROMOTE_SCORE_THRESHOLD` | 88 | promote dev → shared |
| `SCORE_ENV` | 100 | `OCLIVE_KERNEL_BINARY` |
| `SCORE_DEV_FULL_RELEASE` | 94 | release |
| `SCORE_SHARED` | 88 | `%LOCALAPPDATA%/OCLive/runtime/` |
| `SCORE_BUNDLED` | 50 | extension / bundle `bin/` |

**Spawn ordering** is decided by `resolve_kernel_action`, not score alone.

## Startup order (profile-aware attach + bundled-first spawn)

| Phase | Question | SSOT |
|-------|----------|------|
| **A · Running** | Is `:8420` healthy and profile-compatible? | attach / replace |
| **B · Cold spawn** | Which binary when nothing listens? | **Caller bundled first** → shared → dev |

### A — When `/health` succeeds

1. `GET http://127.0.0.1:8420/health` → `kernel_manifest`, `active_profile_summary`
2. Parse caller **`DistroProfileRequirements`** from `distro.oclive.toml`
3. **Attach** when profile **satisfies** caller — even if a stronger local binary exists
4. **Replace** (`profile_mismatch`) when profiles conflict
5. **Pinned** `OCLIVE_KERNEL_BINARY` → always attach; warn on mismatch
6. **`binary_upgrade` replace** — **Frozen** for product; opt-in `OCLIVE_ALLOW_BINARY_UPGRADE=1`

### B — When offline (spawn)

1. **Caller bundled** `oclive-kernel-server`
2. **Shared fallback** — same `OCLIVE_APP_DATA` + profile + roles
3. **Dev builds** — monorepo / env pin

Bundled fail + shared OK → suspect **distro binary**; both fail → suspect **plugins / config**.

**Runtime truth**: Distro capabilities come from **`active_profile_summary`** (loaded `HostProfile`), **not** `kernel_manifest.feature_set`.

## Desktop / VS Code / CLI

- Desktop: `kernel_lifecycle/` — `policy.rs`, `ensure.rs`, `spawn.rs`, `watchdog.rs`
- VS Code: `kernelStrategy.ts` + `kernelClient.ts`; requires built `oclive-cli` in dev
- CLI: `cargo run -p oclive-cli -- kernel ensure --json --plan-only --distro vscode`; spawning a long-lived kernel requires `OCLIVE_API_TOKEN` (the CLI will not create an unrecoverable token). `GET /health` remains public for readiness; send `x-oclive-api-token` on all other routes.

## HTTP routes (kernel)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Liveness + manifest + profile summary |
| POST | `/chat` | Turn |
| GET | `/role_info` | Full `RoleInfo` |
| POST | `/role/load` | Initialize role |
| GET/POST | `/chat/*`, `/user_identity/*`, `/llm/*`, … | See ZH SSOT for full list |

## E2E

```bash
node scripts/e2e-distro-kernel.mjs --scenario spawn
node scripts/e2e-kernel-profile.mjs
node scripts/e2e-cross-host-memory.mjs
```

## Distro capability profile

Each distro ships `distro.oclive.toml`. Merge semantics: [DISTRO_CAPABILITY_PROFILE.md](DISTRO_CAPABILITY_PROFILE.md) §4.

## Bundled vs shared

| Term | Meaning |
|------|---------|
| **Distro bundled kernel** | Full server shipped with install — **default spawn** |
| **Shared fallback** | `%LOCALAPPDATA%/OCLive/runtime/` copy |
| **`promote_with_backup`** | Developer maintenance only |

## Related

- [DISTRO_CAPABILITY_PROFILE.md](DISTRO_CAPABILITY_PROFILE.md)
- [DISTRO_DEFAULT_PLUGINS.md](DISTRO_DEFAULT_PLUGINS.md)
- [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md)
- [CROSS_HOST_MEMORY.md](../role-pack/CROSS_HOST_MEMORY.md)
