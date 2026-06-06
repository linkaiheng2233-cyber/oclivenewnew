# PLUGIN_V1 — Orchestration contract & backend enums (v2 blueprint · legacy six slots)

**Plugin author learning path:** [PLUGIN_AUTHOR_LEARNING_PATH.md](PLUGIN_AUTHOR_LEARNING_PATH.md)

**Current authority:** role-pack **`pipeline.ocblueprint` → `slot_registry`** ([ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)). This English summary covers host orchestration contracts, facade traits, and **v2 instance resolution**; **legacy** `settings.json` → `plugin_backends` sections are **v1 (deprecated)** for migration only. **Full tables (Chinese authority):** [../../creator-docs/plugin-and-architecture/PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md). Rust anchors: `slot_resolver.rs`, `plugin_host.rs`, `plugin_backends.rs`.

**Index (ZH):** [DOCUMENTATION_INDEX.md](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md) · **Architecture overview:** [../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · **Kernel diagram:** [../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) · **Pack versioning:** [PACK_VERSIONING.md](../../creator-docs/role-pack/PACK_VERSIONING.md) · **Remote JSON-RPC:** [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) · **Directory plugins:** [DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md).

---

## Blueprint v2 role packs (`pipeline.ocblueprint`)

**`schema_version: 2`** packs use [`pipeline.ocblueprint`](../role-pack/ROLE_PACK_SPEC.md) **`slot_registry`** as SSOT (open instance keys), not fixed six keys in `settings.json`. The host resolves via **`SlotResolver` / `SlotRunner`**; folding to `PluginBackends` uses **last-wins** per `type`. **`complex_emotion`** is a first-class `slot_registry` `type`; directory plugins declare **`provides: ["complex_emotion"]`** when serving that slot. Persist pack edits: Tauri **`save_role_slot_registry`** (toolbar add/remove slots; **at least one `llm`**; **last `llm` cannot be removed**); then **`invalidate_role_cache`** + **`load_role`**. Session overrides: **`set_session_slot_override`** (in-memory only).

---

## Design rules

- **Backends = compile-time enums**: legacy via `settings.json`; v2 via **`slot_registry`**. No dynamic `cdylib` loading.
- **Default implementations** are the built-in Rust paths; switching backend **does not rename API fields** (especially **`SendMessageResponse.reply`**).
- **Remote:** the host speaks **HTTP JSON-RPC** ([REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)). Missing `OCLIVE_REMOTE_*` URLs → fall back to builtin / in-process LLM with logs.
- **Directory:** `plugins/*/manifest.json` child processes; same JSON-RPC wire as Remote; slot ids in `plugin_backends.directory_plugins` ([DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)).

---

## `PluginBackends` host slots

Runtime struct **`PluginBackends`** has **six** enum fields: **`memory` · `emotion` · `event` · `prompt` · `llm` · `agent`**. Optional **`directory_plugins`** maps each slot to a manifest **`id`** when that slot is **`directory`**. Resolution: **`PluginHost::resolve_for_role`** → **`Arc<dyn …>`** per facade, then **`chat_engine`** calls them in the **`send_message` order** (see below). **`complex_emotion`** scaffold keys are ignored by Serde; runtime maps to the **complex-emotion facility submodule** (facility submodule 1) ([OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md), [SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md) §II).

### Module numbering (aligned with architecture overview)

| # | `plugin_backends` key | Kind |
|---|------------------------|------|
| Module 1 | `memory` | Backend module |
| Module 2 | `emotion` | Backend module |
| Module 3 | `event` | Backend module |
| Module 4 | `prompt` | Backend module |
| Module 5 | `llm` | Backend module |
| Module 6 | `agent` | Backend module |
| Facility submodule 1 | *(no key; in orchestration)* | Complex-emotion facility submodule |
| Facility submodule 2 | *(no key; in orchestration)* | Expert-model facility submodule (expert routing) |

**Backend-module plugin modules** (Remote / directory, etc.) attach to **module K**; they do **not** consume a “module 7” host slot. Full rules: [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md).

---

## `send_message` order (co-present path)

Entry: **`chat_engine::process_message`** → **`process_co_present`** ([`turn_pipeline.rs`](../../crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs)). Remote / stub branches differ; this list is the **PLUGIN_V1-relevant** sequence:

1. **`PluginHost`**: `resolved_plugins_for` → **`PluginHost::resolve_for_role`** binds six **backend modules** (host needs app-data root for **`mcp-servers/*.json`**).
2. **User emotion (backend module):** `emotion.analyze` → `EmotionDto` in the response.
3. **Personality nudge (facility):** `PersonalityEngine::adjust_by_user_emotion`.
4. **Complex-emotion facility submodule** (no. 1): `BuiltinKeywordComplexEmotionProvider` in `co_present` (future Remote); `narrative_hint` → later Prompt (**not** via `PluginHost`).
5. **Knowledge blocks** (optional · facility): pack `knowledge_index` retrieval; may merge with event augment.
6. **Event impact (backend module):** `event.estimate` → `PersonalityEngine::evolve_by_event` (facility).
7. **Memory (backend module):** repository candidates → scene weighting → `memory.rank_memories`.
8. **Favor & relation stage** (facility): `compute_favor_and_relation`.
9. **Prompt (backend module):** `prompt.top_topic_hint` + `prompt.build_prompt` (`PromptInput`, incl. `previous_complex_emotion_narrative_hint`).
10. **Main LLM (backend module):** `llm.generate` (plus bot emotion, portrait, short-term memory, movement intent, etc. — see the same file).

---

## Backend enums (per slot, condensed)

| Slot | Values (meanings) |
|------|-------------------|
| **memory** | `builtin` · `builtin_v2` · `remote` · `directory` · `local` (local uses `_local_plugins`; see [LOCAL_PLUGIN_BRIDGE_SPEC.md](../../creator-docs/plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)) |
| **emotion** | `builtin` · `builtin_v2` · `remote` · `directory` |
| **event** | `builtin` · `builtin_v2` · `remote` · `directory` |
| **prompt** | `builtin` · `builtin_v2` · `remote` · `directory` |
| **llm** | `ollama` · `remote` · `directory` |
| **agent** | `builtin` (ReAct + MCP) · `remote` · `directory` — see root **`AGENTS.md`**. |

Remote / directory failures generally **fall back** to builtin / ollama as documented in the full Chinese page and in code.

---

## `settings.json` minimal example

```json
{
  "schema_version": 1,
  "plugin_backends": {
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "ollama",
    "agent": "builtin"
  }
}
```

If `plugin_backends` is omitted: memory / emotion / event / prompt / **agent** default to **builtin**, **`llm`** defaults to **`ollama`**. Invalid enum strings fail pack parsing.

---

## Session overrides (Tauri)

**`set_session_plugin_backend`** persists per **role + optional session** namespace; **does not rewrite the pack**. `get_role_info` / `load_role` return **`plugin_backends_effective`** and **`plugin_backends_effective_sources`** (pack vs session vs env precedence).

`backend` field semantics: **omit** = no change; **`null`** = clear override for that slot; **string** = set backend (invalid → error). **`directory_plugins` ids** for `directory` still come from the pack unless a future API exposes session-level directory maps ([DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)).

---

## Frontend alignment

TypeScript **`SendMessageResponse`** (`src/utils/tauri-api.ts`) must match `models/dto.rs`: the assistant text field is **`reply`**. `personality_source`, `reply_is_fallback`, `schema`, `api_version` drive UI ([`replyPresentation.ts`](../../src/utils/replyPresentation.ts)).

**Plugin Manager V2** templates (`endpoint-config`, `slot-selector`, `switch-toggle`, …) and manifest `ui_schema` are documented in the **Chinese** PLUGIN_V1 tail section; behavior is the same in English builds.

---

## HTTP `POST /chat` & `personality_source`

`get_role_info`, `load_role`, and **`POST /chat`** (with `--api`) expose **`personality_source`** as **`vector` | `profile`**, aligned with pack **`evolution.personality_source`**.

For the complete RPC tables and manifest examples, open the **[full PLUGIN_V1 (ZH)](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)**.

---

## Permission specification (directory plugins · A4.2)

Optional **`permissions`** on directory-plugin **`manifest.json`** declares high-risk host capabilities. **`oclive_validation::plugin_permissions`**, runtime **`high_risk_grants.json`**, and this table share the **same permission ids** (runtime enforcement is authoritative).

| Permission id | Meaning | User grant required | Default |
|---------------|---------|---------------------|---------|
| `process:spawn` | Host may spawn the plugin child (`process` block) | Yes | Not granted |
| `network:*` | Outbound HTTP for Remote backends (see below) | Yes | Not granted |
| `mcp:http` | MCP server with `transport=http` | Yes (per server `id`) | Not granted |
| `mcp:stdio` | MCP server with `transport=stdio` | Yes (per server `id`) | Not granted |

```json
{
  "schema_version": 1,
  "id": "com.example.myplugin",
  "version": "1.0.0",
  "permissions": ["process:spawn", "network:*"],
  "process": { "command": "node", "args": ["rpc_server.mjs"] }
}
```

- **Omitted `permissions`**: treated as **`[]`** (validation passes).
- **Legacy**: omitted `permissions` + existing **`process`** block still requires a **`process:spawn`** grant for that plugin `id` (A4.1); new plugins should declare **`process:spawn`** explicitly.
- **Remote sidecars**: before JSON-RPC to `OCLIVE_REMOTE_*`, check **`network:*`** with grant ids **`remote:plugin`** / **`remote:llm`**.
- **MCP**: `{app_data}/mcp-servers/*.json`; grants keyed by server **`id`**.
- **On disk**: `high_risk_grants.json` top-level keys match permission ids. **`grant_high_risk_capability`** accepts spec ids; legacy key names remain readable.

Full Chinese section: **[PLUGIN_V1 §权限规范](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)**.

---

[中文](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)
