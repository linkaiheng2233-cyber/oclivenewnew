# Local plugin bridge specification (Phase 2 draft)

Spec-first description of oclive’s **local plugin** contract for future **WASM** and **native process** providers.

[中文](../../creator-docs/plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)

## 1. Goals

- Unified discovery, registration, and capability declaration for local providers.
- Version gates **before** runtime wiring so old runtimes do not misread newer specs.
- Compatible with existing `settings.json` → `plugin_backends`; behavior unchanged when local providers are off.

## 2. Provider descriptor

Each local provider must expose a descriptor (JSON‑equivalent):

```json
{
  "provider_id": "local.demo.memory",
  "schema_version": 1,
  "min_runtime_version": "0.2.0",
  "capabilities": ["memory", "prompt"]
}
```

Fields:

- `provider_id`: non‑empty string; should be globally unique (e.g. `local.vendor.feature`).
- `schema_version`: bridge spec version; only **`1`** is supported today.
- `min_runtime_version`: optional **SemVer** (same rules as pack `manifest.min_runtime_version`, e.g. `0.2.0`); invalid strings or host below requirement → registration rejected.
- `capabilities`: optional array: `memory` / `emotion` / `event` / `prompt` / `llm` (snake_case in JSON, matching host `serde`).

## 3. Version gate rules

- `schema_version == 0`: invalid, reject.
- `schema_version > 1`: current runtime rejects (prompt to upgrade oclive).
- `min_runtime_version` not satisfied: reject (same semantics as manifest).
- On any gate failure: provider is **not** registered; main chat path is unaffected.

### 3.1 Host version & error copy

- Comparison uses the **host app version** (aligned with `CARGO_PKG_VERSION`).
- Errors distinguish **role pack manifest** vs **local plugin descriptor** (implementation uses `validate_min_runtime_version_for_local_plugin` for local paths so messages do not collide with pack load errors).

## 4. Runtime behavior (current phase)

- Runtime exposes `LocalPluginBridge` and `LocalPluginRegistry`.
- Registry today only:
  - validates (`schema_version` / `min_runtime_version`)
  - indexes capabilities (lookup providers per module)
- Local providers are **not** yet wired end‑to‑end into the `send_message` main path; future work plugs concrete providers behind this skeleton.

### 4.1 File manifest discovery (`file_manifest`)

- Directory: **`<roles root>/_local_distros/chat-pro/plugins/`** (same `roles_dir` as `RoleStorage`; in dev often `distros/chat-pro/roles/_local_distros/chat-pro/plugins/` in the repo).
- Scan `*.json` files (case‑insensitive); each file deserializes to one `LocalPluginProviderDescriptor`.
- Parse/read errors: skip file, log `oclive_plugin` warning, do not block startup.
- On startup the host `register_provider` for each discovery; **duplicate `provider_id` → later registration wins** (directory order is platform‑dependent — **do not rely on override order**; configure each id once).

## 5. Relation to the existing backend stack

- `BackendRegistry + PluginResolver` stay stable.
- Local providers register as a **skeleton** first (default resolution unchanged).
- Future: resolver‑level selection policy while preserving remote/builtin fallback semantics.

### 5.1 `plugin_backends.memory`

- When pack or session override sets `plugin_backends.memory` to **`local`**, the host picks a registered provider with the `memory` capability.
- Optional sibling field **`plugin_backends.local_memory_provider_id`**: when non‑empty, match that `provider_id` exactly; on miss, fall back lexicographically and `warn`.
- Multiple memory providers without `local_memory_provider_id`: take **lexicographically first** `provider_id` and `warn` ambiguity (recommend setting id explicitly in the pack).
- **Current behavior**: ranking still delegates to **`builtin`** (local path selects a provider via registry, then uses built-in ranking); `MemoryRetrieval::diagnostic_local_provider_id` exposes the chosen id.
- **Session scope**: aligned with `SendMessageRequest.session_id`, use Tauri **`set_session_plugin_backend`** (`module = memory`, optional **`local_memory_provider_id`**) for overrides; read merged `plugin_backends_effective` via **`get_role_info`** with the same **`session_id`** (see [PLUGIN_V1.md](PLUGIN_V1.md) “session overrides”).
