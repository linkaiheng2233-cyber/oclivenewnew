# Cross-host memory and role-pack carried data (contract)

[中文](../../creator-docs/role-pack/CROSS_HOST_MEMORY.md)

**Audience**: VS Code extension, casino POC, launcher, headless `kernel_server`, and other **multi-distro integrators**.  
**Status**: **Phase 1 confirmed** (2026-05-20); aligned with [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) and [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md).

---

## 1. One sentence

**Role pack** = cross-distro readable **identity, content, policy** (no dynamic runtime).  
**Kernel** = unified **load contract** (`load_role`, `config.json` semantics, `POST /chat`).  
**Each host** = **L2 private state** on its own; **L3 companionship continuity** via **shared `app.db`**.

---

## 2. Three-layer model (Phase 1 confirmed)

| Layer | Name | Location | Cross-host |
|-------|------|----------|------------|
| **L1** | Role pack SSOT | `distros/chat-pro/roles/{role_id}/` | ✅ Desktop & VS Code **same** `OCLIVE_ROLES_DIR` |
| **L2** | Host-private | VS Code editor context, etc. | ❌ Prepended to user message; **no new memory API** |
| **L3** | Cross-host runtime | `{app_data}/app.db` | ✅ **Shared DB** (LTM, favor, relation stage) |

```text
distros/chat-pro/roles/{id}/  ──load_role──►  Desktop / VS Code / kernel_server
       │                           │
       │ L1 identity+policy+content  │ L2 editor context → message
       │                           │
       └──────── config.json ──────┴──► L3 shared app.db (single writer)
```

**Host duty (L1)**: Load the same role pack; interpret `config.json` and blueprint `memory_config`; **do not mutate** read-only pack content.

---

## 3. What the role pack carries

Full format: [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md). Summary:

### 3.1 Required / strongly recommended

| Category | Path | Notes |
|----------|------|-------|
| Entry | `pipeline.ocblueprint` (v2) | Identity, `meta`, blueprint `slot_registry` |
| Behavior policy | `config.json` | `time` / `memory` / `relation` / `chat_storage`, etc. |
| Prompts | `prompts/` | System prompt, openings |
| Scenes | `scenes/{scene_id}/` | `scene.json`, descriptions, assets |

### 3.2 Policy vs content (do not confuse)

| Concept | Role pack | `{app_data}/app.db` |
|---------|-----------|----------------------|
| Memory decay half-life, reinforcement | `config.json` → `memory.*` | — |
| Extracted LTM entries | Parameters in pack | **`long_term_memory` table** |
| Initial favor definition | `meta.relations` | Runtime **`favorability`**, etc. |
| Chat log | `chat_storage.location=global` | SQLite `chat_*` |

**Phase 1 memory extraction**: After editor context is prepended to the message, **may** be written to `long_term_memory` at turn end; scene-level filtering is later.

---

## 4. Phase 1 runtime decisions (confirmed)

| Item | Decision |
|------|----------|
| **Single kernel writer** | Only **one** kernel process writes `app.db` at a time |
| **attach vs spawn** | **Profile-aware attach + bundled-first spawn** (shared Rust `resolve_kernel_action`): `/health` OK and profile compatible → **Attach** (no replace just because a stronger local binary exists); profile conflict → **Replace** (restart); no process → spawn **distro bundled** → **shared fallback** (same `OCLIVE_APP_DATA` / profile / roles; plugins reused). `OCLIVE_KERNEL_BINARY` pin → no replace. `binary_upgrade` auto-replace **Frozen**. See [DISTRO_KERNEL_LIFECYCLE.md](../kernel/DISTRO_KERNEL_LIFECYCLE.md) · [KERNEL_SCHEDULER_RESCOPE.md](../../handoff/KERNEL_SCHEDULER_RESCOPE.md) |
| **Port** | Fixed **`8420`** (`OCLIVE_API_PORT`) |
| **`OCLIVE_ROLES_DIR`** | Desktop & extension **same path** |
| **`OCLIVE_APP_DATA`** | Brand dir `%LOCALAPPDATA%/OCLive/data` (required on spawn; see [OCLIVE_APP_DATA.md](../kernel/OCLIVE_APP_DATA.md)) |
| **Tauri migration** | On first canonical start, **copy** legacy `%APPDATA%/com.oclivenewnew.app` → `OCLive/data` |
| **`scene_id`** | Extension **`vscode`**; desktop **`default`** (or pack scene id) |
| **`session_id`** | **Independent** per host; no dual-write of same session |
| **`chat_storage.location`** | **`global`** |
| **Demo role** | **`mumu` v2**; pack must include **`scenes/vscode/`** |
| **HTTP surface** | `GET /health` + `POST /chat` |
| **Tests** | OOCP / Codex track A |

**Phase 1 note**: Desktop and VS Code are **HTTP clients** (shared `resolve_kernel_action` for attach/spawn/replace); sole writer is `oclive-kernel-server @ :8420`. See [DISTRO_KERNEL_LIFECYCLE.md](../kernel/DISTRO_KERNEL_LIFECYCLE.md).

**Not in Phase 1**: WebSocket push, scheduler layer (Phase 3 optional `oclive-runtimed`), casino POC, pack `memories/` load, scene-level memory filter.

---

## 5. `scene_id` and `session_id`

| Field | Semantics | Phase 1 |
|-------|-----------|---------|
| `role_path` | Absolute role pack dir | `{OCLIVE_ROLES_DIR}/mumu` |
| `scene_id` | Chat / narrative bucket | Extension **`vscode`**; must be in pack `meta.scenes` |
| `session_id` | Multiple sessions per scene | Extension UUID in `globalState` |

**Note**: `long_term_memory` is per-role; **not** auto-isolated by `scene_id`.

---

## 6. Single kernel, multiple roles (confirmed)

- One kernel process can serve multiple **`role_id`** with natural isolation.
- Multi-role scenes are solved by **blueprint / orchestration**, not “one kernel per role”.
- Bottleneck is **LLM**; multiple kernels writing one `app.db` causes SQLite conflicts — hence **single writer**.

---

## 7. Roadmap

### Phase 2: Single daemon, multiple hosts (**desktop spawn-only delivered**)

- Desktop & VS Code are **HTTP clients**; scheduling policy in `oclive_kernel_runtime` (`kernel_strategy.rs`); `/health` enhanced (`kernel_manifest`, `distro_id`, `distro_profile_hash`).
- Hosts **call shared policy, execute spawn/replace/attach**; desktop `kernel_lifecycle/policy.rs`, VS Code via `oclive-cli kernel ensure --plan-only`.
- P0 IPC proxied over HTTP; `/role_snapshot` for cross-host UI poll.
- **User Identity HTTP**: `GET /user_identity/state`, `POST /user_identity/set`, `POST /user_identity/scene_set` — same impl as Tauri; VS Code / attach must use HTTP.
- Spec: [DISTRO_KERNEL_LIFECYCLE.md](../kernel/DISTRO_KERNEL_LIFECYCLE.md).

### Phase 3: Thin scheduler (`oclive-runtimed`)

- Optional binary: health supervision + per-role `POST /chat` queue to upstream `:8420`.
- **No AI logic**; may reuse `resolve_kernel_action` as `:8420` guardian.

### Casino POC (Phase 2+)

| Item | Convention |
|------|------------|
| `scene_id` | `casino` |
| Round state | **L2**; optional L3 write at round end |

---

## 8. Related

| Doc | Purpose |
|-----|---------|
| [ROLE_PACK_SPEC.md](ROLE_PACK_SPEC.md) | Disk format |
| [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md) | Chat vs memory tables |
| [OOCP_TEST_SUITE.md](../testing/OOCP_TEST_SUITE.md) | HTTP contract |
| [KERNEL_PLATFORM_DEVELOPER_PATH.md](../getting-started/KERNEL_PLATFORM_DEVELOPER_PATH.md) | Headless integration |
