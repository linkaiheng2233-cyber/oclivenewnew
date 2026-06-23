# Blueprint and system configuration (SETTINGS_REFERENCE)

> **`pipeline.ocblueprint` is the single source of system configuration.** Fields below are **blueprint / host admin** only — not entry-level role-pack edits. Creator-facing fields: **[ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) §0** · **[ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)**.

## 0. Blueprint-only fields

### `runtime_config` (v3 target SSOT)

| Field | Notes |
|-------|--------|
| `interaction_mode` | `immersive` \| `pure_chat` |
| `memory_config` | Memory policy object |
| `reply_quality_anchor` | Quality anchor prose |
| `remote_fallback_to_builtin` | Pack-level hint (host `app_settings` still authoritative) |
| `dual_core.enabled` | Dual-core switch; default **false** |

On **schema_version 2**, `runtime_config` triggers a **pack validate warning** and is **ignored** at load.

[中文](../cli/SETTINGS_REFERENCE.md)

---

### Slots and other blueprint sections

| Category | Fields |
|----------|--------|
| Slots | **`slot_registry`** (`type`, `backend`, `plugin`, `model`, `url`, `position`, …) |
| Graph | **`groups`**; **`module_relations`** must **not** be stored (derived at runtime) |
| Engine | **`interaction_mode`**, **`memory_config`**, **`identity_binding`**, **`evolution`** (engine params), **`remote_presence`**, **`autonomous_scene`** — target: **`runtime_config.*`** (today often still under **`meta.*`**) |
| Dual-core (RFC) | **`runtime_config.dual_core.enabled`**, **`pipeline.*`**, **`zone`** — default off; creators must not enable alone |
| Host app (not in pack) | **`remote_fallback_to_builtin`**, **`monolith.toml`** |

[中文全文](../cli/SETTINGS_REFERENCE.md)

---

**v2:** backends in **`slot_registry`**. Legacy **`settings.json` → `plugin_backends`** sections below are **deprecated** comparison only.

This document describes configuration semantics shared by the **desktop host (Tauri)** and **`oclive-cli` scaffolds**. Single sources of truth remain code:

- Enums and structs: [`kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs)
- Resolution and binding: [`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs)
- Protocol and tables: [`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)

**Standard JSON has no comments**: use **`_`-prefixed keys** for prose (ignored at load), or out-of-pack docs. `oclive-cli` sample packs use `_comment_*` keys per slot.

---

## I. Six host slots (`PluginBackends`)

The runtime struct **`PluginBackends`** has these **6** fields (Serde **ignores unknown fields**, so JSON may contain extra keys such as scaffold `complex_emotion` without host parse errors).

| Field | Facade trait (orchestration entry) | Common built-in (in-process) |
|-------|-----------------------------------|-------------------------------|
| `memory` | [`MemoryRetrieval`](../../kernel/crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs) | default `MemoryBackend::Builtin` |
| `emotion` | user emotion analysis (see `plugin_host` / `EmotionAnalyzer`) | `EmotionBackend::Builtin` |
| `event` | event impact estimation (`EventEstimator`) | `EventBackend::Builtin` |
| `prompt` | `PromptAssembler` / `PromptBuilder` | `PromptBackend::Builtin` |
| `llm` | `LlmClient` | **`LlmBackend::Ollama`** (default local client; **no `builtin` literal**) |
| `agent` | [`AgentProvider`](../../kernel/crates/oclive_kernel_host/src/domain/agent.rs) | `AgentBackend::Builtin` |

When the whole `plugin_backends` block is omitted: memory / emotion / event / prompt / agent behave as **`builtin`**, **`llm` is `ollama`** (see PLUGIN_V1 examples).

### 1.1 Per-slot values (v1 enum summary)

Full table and JSON-RPC method names are in **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**. Common operator-facing values:

| Slot | Common values | When choosing `remote` / `directory` |
|------|---------------|--------------------------------------|
| memory | `builtin` / `remote` / `directory` / `local` (`builtin_v2` read alias) | `remote`: `OCLIVE_REMOTE_PLUGIN_URL`; `directory`: configure `directory_plugins.memory` |
| emotion | `builtin` / `remote` / `directory` (`builtin_v2` read alias) | same |
| event | `builtin` / `remote` / `directory` (`builtin_v2` read alias) | same |
| prompt | `builtin` / `remote` / `directory` (`builtin_v2` read alias) | same |
| llm | **`ollama`** / `remote` / `directory` | **`remote`**: `OCLIVE_REMOTE_LLM_URL`; **`OCLIVE_LLM_BACKEND`** may override at load |
| agent | `builtin` / `remote` / `directory` | `remote`: sidecar JSON-RPC; `directory`: configure `directory_plugins.agent` |

**Strings not in the v1 enum** (e.g. literal `none`) cause **role pack parse failure**. If scaffolds or docs say “none”, that means **logically off / undeclared**; for host-loadable JSON **omit the key** (fall back to default) or use a legal enum value.

### 1.2 `directory_plugins` object

When any slot is **`directory`**, fill **`manifest.id`** (string) for that slot under `plugin_backends.directory_plugins`. See [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md).

---

## II. Complex emotion: `plugin_backends` extension key (not a host slot)

**Architecture:** **facility submodule 1** (normative name: **complex-emotion facility submodule**). Naming and **facility submodule 2** (expert-model facility submodule / expert routing): **[OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)** ([中文](../../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)).

**`PluginBackends` has no `complex_emotion` field.** `oclive-cli` writes the key inside **`plugin_backends`** for factory presets; the host **ignores** it on deserialize. The hot path uses `BuiltinKeywordComplexEmotionProvider` in `co_present`—**not** `PluginHost`.

| Item | Detail |
|------|--------|
| vs **emotion backend module** | emotion produces `EmotionResult`; this facility outputs `narrative_hint` for the **prompt backend module** |
| vs **backend-module plugin modules** | Sidecar `complex_emotion.resolve_turn` (`OCLIVE_COMPLEX_EMOTION_URL`) exists; **not** switched via this JSON key yet (roadmap); **not** “module 7” |
| vs **Monolith** | Weld key `complex_emotion` (one of seven weld keys), ≠ host slot |

- Sidecar wire: [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md).
- Matches **`oclive-cli`** `CONFIG_REFERENCE.md` and **`init --help`** preset matrix.

---

## III. `oclive-cli` preset matrix (logic → JSON)

| Slot | minimal | mixed | full |
|------|---------|-------|------|
| memory / emotion / event / prompt | builtin | builtin | builtin |
| llm | ollama | ollama | remote |
| agent | **omit key** (semantic none) | builtin | builtin |
| complex_emotion | none | builtin | remote |

---

## IV. Switching from `builtin` / `ollama` to `remote` (steps)

1. Prepare an HTTP JSON-RPC sidecar implementing PLUGIN_V1 / REMOTE_PLUGIN_PROTOCOL methods.
2. Set URLs in the environment, e.g. **`OCLIVE_REMOTE_PLUGIN_URL`** (shared sidecar) and **`OCLIVE_REMOTE_LLM_URL`** (LLM only).
3. Edit **`settings.json` → `plugin_backends`**: set the target slot to **`remote`** (`llm` becomes **`remote`**, not `builtin`).
4. Restart the host or reload the role; watch logs for downgrade/fallback when URLs are missing.

---

## V. `monolith.toml` (compile-time, not runtime)

Written by **`oclive-cli init`** when Monolith is enabled at **project root**; consumed **only at compile time** (**`cargo run -p oclive-cli -- build`** reads it and regenerates `process_message_monolith.rs`; you may also use **`cargo build --features monolith`** alone). **Orthogonal** to **`settings.json` → `plugin_backends`**: role pack load **does not** read this file.

| Field | Meaning |
|-------|---------|
| **`[monolith].enabled`** | Whether Monolith compile path is enabled for this project (`oclive build` skips the second `cargo build` with `monolith` when `false`). |
| **`weld_modules`** | List of welded module names; **empty array** means “weld all weldable slots, then apply `exclude`”. **Must not be non-empty together with `exclude`.** |
| **`exclude`** | When **`weld_modules` is empty**, exclude listed slots from full weld; those slots stay trait/PluginHost placeholders in generated code. |

**Bench report JSON Schema** (`oclive bench`): [`kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json`](../../kernel/crates/oclive-cli/schemas/oclive_bench_report.schema.json) (relative paths assume repo clone layout).

See [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) section 4.

---

## VI. Role pack `config.json` → `chat_storage`

Optional **`chat_storage`** object in `distros/chat-pro/roles/{role_id}/config.json`. Loaded by `RoleStorage::load_role`; type: `oclive_kernel_types::RolePackChatStorageConfig`. **Not** part of `pipeline.ocblueprint`.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `backend` | `"hybrid"` \| `"file"` \| `"sqlite"` | `"hybrid"` | Chat storage backend (overridden by `OCLIVE_CHAT_STORAGE_BACKEND` at process level) |
| `max_messages_per_session` | `u32` | host **500** | Per-session FIFO cap (user + assistant rows) |
| `auto_cleanup_days` | `u32` | unset = off | Delete sessions with `updated_at` older than N days |
| `auto_cleanup_max_sessions` | `u32` | unset = off | Keep at most N sessions per role (drop oldest) |
| `replay_similarity_threshold` | `f64` | `0.6` | Memory replay dedupe similarity (**0.1–1.0**); higher = stricter, fewer duplicates merged |

Selection guide: [STORAGE_BACKEND_GUIDE.md](../storage/STORAGE_BACKEND_GUIDE.md) · Architecture: [CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md).

---

## VII. Related doc index

| Topic | Doc |
|-------|-----|
| CLI usage and flags | [OCLIVE_CLI_GUIDE.md](OCLIVE_CLI_GUIDE.md) |
| Preset table inside generated projects | **`CONFIG_REFERENCE.md`** after `init` |
| Chat storage backend selection | [STORAGE_BACKEND_GUIDE.md](../storage/STORAGE_BACKEND_GUIDE.md) |
| Plugins & sidecars overview | [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| Directory plugins | [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| Compile-time Monolith | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) (`monolith.toml`, `build` / `bench`, dual `[[bin]]`) |

---

[中文](../../creator-docs/cli/SETTINGS_REFERENCE.md)
