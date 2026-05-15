# PLUGIN_V1 — Orchestration contract & backend enums (summary)

**Plugin author learning path:** [PLUGIN_AUTHOR_LEARNING_PATH.md](PLUGIN_AUTHOR_LEARNING_PATH.md)

This is an **English summary** of the v1 contract between the host (Tauri / `chat_engine`) and swappable subsystems: naming, DTO shape, and `settings.json` enums. **Source of truth** remains Rust: `src-tauri/src/domain/*_*.rs`, `src-tauri/src/models/plugin_backends.rs`. **Full tables and edge cases (Chinese):** [../../creator-docs/plugin-and-architecture/PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md).

**Index (ZH):** [DOCUMENTATION_INDEX.md](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md) · **Kernel diagram:** [../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) · **Pack versioning:** [PACK_VERSIONING.md](../../creator-docs/role-pack/PACK_VERSIONING.md) · **Remote JSON-RPC:** [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) · **Directory plugins:** [DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md).

---

## Design rules

- **v1 backends = compile-time enums** chosen via `settings.json`; no dynamic `cdylib` loading.
- **Default implementations** are the built-in Rust paths; switching backend **does not rename API fields** (especially **`SendMessageResponse.reply`**).
- **Remote:** the host speaks **HTTP JSON-RPC** ([REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)). Missing `OCLIVE_REMOTE_*` URLs → fall back to builtin / in-process LLM with logs.
- **Directory:** `plugins/*/manifest.json` child processes; same JSON-RPC wire as Remote; slot ids in `plugin_backends.directory_plugins` ([DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)).

---

## `PluginBackends` host slots

Runtime struct **`PluginBackends`** has **six** enum fields: **`memory` · `emotion` · `event` · `prompt` · `llm` · `agent`**. Optional **`directory_plugins`** maps each slot to a manifest **`id`** when that slot is **`directory`**. Resolution: **`PluginHost::resolve_for_role`** → **`Arc<dyn …>`** per facade, then **`chat_engine`** calls them in the **`send_message` order** (see below). **`complex_emotion`** keys in scaffolds may be ignored by Serde and are **not** one of the six host slots ([SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md)).

---

## `send_message` order (co-present path)

Entry: **`chat_engine::process_message`** → **`process_co_present`** ([`co_present.rs`](../../src-tauri/src/domain/chat_engine/co_present.rs)). Remote / stub branches differ; this list is the **PLUGIN_V1-relevant** sequence:

1. **`PluginHost`**: `resolved_plugins_for` → **`PluginHost::resolve_for_role`** binds **memory / emotion / event / prompt / llm / agent** (host needs app-data root for **`mcp-servers/*.json`**).
2. **User emotion:** `emotion.analyze` → `EmotionDto` in the response.
3. **Personality nudge:** `PersonalityEngine::adjust_by_user_emotion`.
4. **Knowledge blocks** (optional): pack `knowledge_index` retrieval; may merge with event augment.
5. **Event impact:** `event.estimate` → `PersonalityEngine::evolve_by_event`.
6. **Memory:** repository candidates → scene weighting → `memory.rank_memories`.
7. **Favor & relation stage:** `compute_favor_and_relation`.
8. **Prompt:** `prompt.top_topic_hint` + `prompt.build_prompt` (`PromptInput`).
9. **Main LLM:** `llm.generate` (plus bot emotion, portrait, short-term memory, movement intent, etc. — see the same file).

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
