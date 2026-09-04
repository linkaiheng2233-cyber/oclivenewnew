# Blueprint and system configuration (SETTINGS_REFERENCE)

> For the current reference host's v2/v3/v4 combined directories, **`pipeline.ocblueprint` is the single source of in-pack runtime configuration.** It does not schedule the Stable main path through `steps[]`; host settings, distro capability ceilings, and in-memory session overrides are outside the pack. Fields below are normally **reference-host blueprint / host-admin** concerns and are not part of the kernel-minimal role contract. Stable v4 `inference_profile` is the sole exception that an editor may expose through a non-technical creator form. Creator-facing fields: **[ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) §0** · **[ROLE_PACK_BOUNDARY.md](../../handoff/ROLE_PACK_BOUNDARY.md)**.

## 0. Blueprint-only fields

### `runtime_config` (Stable v4 SSOT; v3 dual-core Beta)

| Field | Notes |
|-------|--------|
| `interaction_mode` | `immersive` \| `pure_chat` |
| `memory_config` | Memory policy object |
| `reply_quality_anchor` | Quality anchor prose |
| `remote_fallback_to_builtin` | Pack-level hint (host `app_settings` still authoritative) |
| `dual_core.enabled` | Dual-core switch; default **false** |
| `inference_profile` | Stable v4 portable ideal generation behavior; never selects a model, GGUF file, local runtime, or machine-specific values |

On **schema_version 2**, `runtime_config` triggers a **pack validate warning** and is **ignored** at load.
On **schema_version 4**, `runtime_config` is active on the Stable path and `dual_core` is rejected. Frozen **schema_version 3** remains only for the dual-core Beta.

#### `runtime_config.inference_profile` (Stable v4)

The pack editor may expose this as a creator-facing “ideal configuration blueprint.” It expresses how a role would like replies to be generated. The host still makes the final decision and may clamp values to user settings, installed-model limits, device capacity, and kernel safety limits. **Chat Pro settings continue to own the actual backend and model.** This object must not contain Ollama/llama.cpp paths, model names, GGUF files, GPU-layer counts, or thread counts.

| Path | Type / range | Meaning |
|------|--------------|---------|
| `generation.temperature` | number, `0.0–2.0` | Sampling temperature preference |
| `generation.top_p` | number, `>0.0–1.0` | Nucleus-sampling preference |
| `generation.preferred_output_tokens` | integer, `1–32768` | Ideal response budget |
| `generation.maximum_output_tokens` | integer, `1–32768` | Hard response limit; not below preferred when both are set |
| `context.minimum_tokens` | integer, `1–262144` | Smallest acceptable context intent |
| `context.preferred_tokens` | integer, `1–262144` | Ideal context window; not below minimum when both are set |
| `reasoning.mode` | `instant` \| `adaptive` \| `deep` | Model-independent reasoning-mode intent |
| `reasoning.effort` | number, `0.0–1.0` | Reasoning-effort intent |
| `performance_intent.priority` | `latency` \| `balanced` \| `quality` | Latency, balanced, or quality priority |
| `performance_intent.prefer_prefix_cache` | boolean | Prefer stable-prefix cache reuse |
| `performance_intent.prefer_model_residency` | boolean | Prefer model residency; `false` makes the current Ollama adapter explicitly request unload after the response (`keep_alive: 0`) |
| `performance_intent.allow_context_reduction` | boolean | Permit context reduction on constrained devices |
| `performance_intent.allow_output_reduction` | boolean | Permit output-budget reduction on constrained devices |

The current kernel forwards `temperature`, `top_p`, the output limit, and preferred context to supported main-LLM adapters. Other fields remain stable forward-compatible intent. A host may leave a hint unsupported, but must never reinterpret it as local model selection.

[中文](../cli/SETTINGS_REFERENCE.md)

---

### Slots and other blueprint sections

| Category | Fields |
|----------|--------|
| Slots | **`slot_registry`** (`type`, `backend`, `plugin`, `model`, `url`, `position`, …) |
| Graph | **`groups`**; **`module_relations`** must **not** be stored (derived at runtime) |
| Engine | **`interaction_mode`**, **`memory_config`**, **`identity_binding`**, **`evolution`**, **`remote_presence`**, **`autonomous_scene`** — Stable v4 uses **`runtime_config.*`** only; **`meta.*`** is a v2 compatibility fallback |
| Dual-core (RFC) | **`runtime_config.dual_core.enabled`**, **`pipeline.*`**, **`zone`** — default off; creators must not enable alone |
| Host app (not in pack) | **`remote_fallback_to_builtin`**, **`monolith.toml`** |

[中文全文](../cli/SETTINGS_REFERENCE.md)

---

**Current reference-host v2/v3/v4 blueprints:** backends live in **`slot_registry`**; new Stable examples in this format family use v4. Legacy **`settings.json` → `plugin_backends`** sections below are **deprecated** comparison only.

The `portable-core` validation profile is a historical reference-host visual baseline that requires a core persona and seven default emotion images. It is not the cross-distro kernel-minimal role contract.

This document describes configuration semantics shared by the **desktop host (Tauri)** and **`oclive-cli` scaffolds**. Single sources of truth remain code:

- Enums and structs: [`kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs)
- Resolution and binding: [`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs)
- Protocol and tables: [`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)

**Standard JSON has no comments**: use **`_`-prefixed keys** for prose (ignored at load), or out-of-pack docs. `oclive-cli` sample packs use `_comment_*` keys per slot.

---

## I. Six stable slot types and runtime `PluginBackends`

Current blueprints express selection through each `slot_registry` instance's `type` and `backend`; runtime folds the six stable types into the six-field **`PluginBackends`** compatibility view. Legacy `settings.json.plugin_backends` ignores unknown fields, so a scaffold `complex_emotion` key does not fail parsing, but it does not enable the facility through those six fields.

| Field | Facade trait (orchestration entry) | Common built-in (in-process) |
|-------|-----------------------------------|-------------------------------|
| `memory` | [`MemoryRetrieval`](../../kernel/crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs) | default `MemoryBackend::Builtin` |
| `emotion` | user emotion analysis (see `plugin_host` / `EmotionAnalyzer`) | `EmotionBackend::Builtin` |
| `event` | event impact estimation (`EventEstimator`) | `EventBackend::Builtin` |
| `prompt` | `PromptAssembler` / `PromptBuilder` | `PromptBackend::Builtin` |
| `llm` | `LlmClient` | **`LlmBackend::Ollama`** (default local client; **no `builtin` literal**) |
| `agent` | [`AgentProvider`](../../kernel/crates/oclive_kernel_host/src/domain/agent.rs) | `AgentBackend::Builtin` |

For legacy v1 only, omitting `plugin_backends` defaults memory / emotion / event / prompt / agent to **`builtin`** and llm to **`ollama`**. A current blueprint must retain at least one LLM instance; a healthy co-present path also needs an effective Prompt.

### 1.1 Per-type backend values

Full table and JSON-RPC method names are in **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**. Common operator-facing values:

| Slot | Common values | When choosing `remote` / `directory` |
|------|---------------|--------------------------------------|
| memory | `builtin` / `remote` / `directory` / `local` / `none` (`builtin_v2` read alias) | `remote`: `OCLIVE_REMOTE_PLUGIN_URL`; directory: instance `plugin` |
| emotion | `builtin` / `remote` / `directory` / `none` (`builtin_v2` alias) | same |
| event | `builtin` / `remote` / `directory` / `none` (`builtin_v2` alias) | same |
| prompt | `builtin` / `remote` / `directory` / `none` (`builtin_v2` alias) | same; `none` removes a required healthy-path capability |
| llm | **`ollama`** / `remote` / `directory` / `none` | remote: `OCLIVE_REMOTE_LLM_URL`; directory: instance `plugin`; `none` cannot produce a normal reply |
| agent | `builtin` / `remote` / `directory` / `none` | remote: Agent sidecar; directory: instance `plugin` / `plugins` |

`none` is a legal explicit-off backend for the six current types, although disabling prompt or llm prevents a healthy co-present reply. Any other token outside the allowed set for its type fails legacy parsing or blueprint validation.

### 1.2 Directory plugin ids

For a current **`directory`** instance, put the plugin's `manifest.id` in `plugin` or, for merge-capable types, `plugins[]`. Only legacy v1 uses `plugin_backends.directory_plugins`. See [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md).

---

## II. Complex-emotion facility type (not a seventh stable slot)

**Architecture:** **facility submodule 1** (normative name: **complex-emotion facility submodule**). Naming and **facility submodule 2** (expert-model facility submodule / expert routing): **[OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)** ([中文](../../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)).

**`PluginBackends` has no `complex_emotion` field.** The current legacy CLI scaffold still writes a hint key inside **`plugin_backends`**, which the host **ignores** on deserialize; that is not how a current pack enables the facility. The Stable path resolves `type: complex_emotion` + `backend: builtin|remote|directory|none` from blueprint `slot_registry` through `PluginHost` / `SlotRunner`, last-wins. **Omitted or `none` disables hint reads/writes**; `builtin` enables carry-over storage and deterministic Fast intensity; `remote` / `directory` additionally provide post-LLM fallback. A valid main-LLM `[EMO]` always wins.

| Item | Detail |
|------|--------|
| vs **emotion backend module** | emotion produces user-utterance `EmotionResult` as one fallback input; this facility resolves reply emotion and `narrative_hint` after generation, and the next **prompt backend module** sees only a content-free carry-over signal |
| vs **backend-module plugin modules** | Sidecar `complex_emotion.resolve_turn` (`OCLIVE_COMPLEX_EMOTION_URL`) exists; current packs select `builtin` / `remote` / `directory` / `none` through the `slot_registry` entry, not through an ignored `plugin_backends` extension key; **not** “module 7” |
| vs **Monolith** | Weld key `complex_emotion` (one of seven weld keys), ≠ host slot |

- Sidecar wire: [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md).
- Matches **`oclive-cli`** `CONFIG_REFERENCE.md` and **`init --help`** preset matrix.

---

## III. `oclive-cli` preset matrix (logic → JSON)

| Slot | minimal | mixed | full |
|------|---------|-------|------|
| memory / emotion / event / prompt | builtin | builtin | builtin |
| llm | ollama | ollama | remote |
| agent | Logical preset is none; non-dual legacy output **omits the key and therefore resolves to builtin**, while the v3 dual-core blueprint writes explicit `none` | builtin | builtin |
| complex_emotion | none | builtin | remote |

---

## IV. Switching from `builtin` / `ollama` to `remote` (steps)

1. Prepare an HTTP JSON-RPC sidecar implementing PLUGIN_V1 / REMOTE_PLUGIN_PROTOCOL methods.
2. Set URLs in the environment, e.g. **`OCLIVE_REMOTE_PLUGIN_URL`** (shared sidecar) and **`OCLIVE_REMOTE_LLM_URL`** (LLM only).
3. Edit **`pipeline.ocblueprint` → `slot_registry`**: set the target instance's `backend` to **`remote`** (LLM uses `remote`; its local default is `ollama`).
4. Restart the host or reload the role; watch logs for downgrade/fallback when URLs are missing.

---

## V. `monolith.toml` (compile-time, not runtime)

Written by **`oclive-cli init`** when Monolith is enabled at **project root** and consumed **only at compile time**. It is orthogonal to runtime pack `slot_registry` and legacy `plugin_backends`; role loading does **not** read this file.

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
