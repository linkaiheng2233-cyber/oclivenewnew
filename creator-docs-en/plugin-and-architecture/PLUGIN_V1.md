# PLUGIN_V1 — Orchestration contract & backend enums (v2/v3/v4 blueprints · legacy six slots)

**SSOT scope:** six-slot DTOs, backend enums, resolution, and slot calls inside Stable's fixed stages. Event Ring wire has a separate contract.

**Last updated:** 2026-09-05.

**Plugin author learning path:** [PLUGIN_AUTHOR_LEARNING_PATH.md](PLUGIN_AUTHOR_LEARNING_PATH.md)

**Current authority:** role-pack **`pipeline.ocblueprint` → `slot_registry`** ([ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md)). This English page is **condensed** (not a quiet 1:1 of ZH). It covers host orchestration contracts, facade traits, and **v2/v3/v4 instance resolution**; new Stable packs use v4. **Legacy** `settings.json` → `plugin_backends` sections are **v1 (deprecated)** for migration only. **Full tables (Chinese SSOT):** [../../creator-docs/plugin-and-architecture/PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md). Rust anchors: `slot_resolver.rs`, `plugin_host.rs`, `plugin_backends.rs`.

**Index (ZH):** [DOCUMENTATION_INDEX.md](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md) · **Architecture overview:** [../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md) · **Kernel diagram:** [../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md) · **Event Ring:** [EVENT_RING.md](EVENT_RING.md) · **Pack versioning:** [PACK_VERSIONING.md](../../creator-docs/role-pack/PACK_VERSIONING.md) · **Remote JSON-RPC:** [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) · **Directory plugins:** [DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md).

| ZH section (normative) | EN coverage |
|------------------------|-------------|
| Blueprint v2/v3/v4 / design rules / six slots / `send_message` order | Condensed below |
| Per-slot input/output facet tables | Backend enum table only → [ZH](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| Plugin Manager V2 `ui_template` / `ui_schema` / `provides` | Pointer → [ZH §前端 UI](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| `reply_post_process` / `theater_director` / `voice.asr` / `com.user.tts.*` side channels | Permission + pointer; full RPC in ZH |
| Directory `permissions` / `slot_attachment` | Condensed permission table below |

---

## Blueprint role packs (`pipeline.ocblueprint`)

Supported **v2/v3/v4** packs use [`pipeline.ocblueprint`](../role-pack/ROLE_PACK_SPEC.md) **`slot_registry`** as SSOT (open instance keys), not fixed six keys in `settings.json`; new Stable packs use v4 and v3 remains the frozen dual-core Beta. The host resolves via **`SlotResolver` / `SlotRunner`**; folding to `PluginBackends` uses **last-wins** per `type`. **`complex_emotion`** is a first-class facility `type` in the open `slot_registry`, but not one of the six stable host slots; directory plugins declare **`provides: ["complex_emotion"]`** when serving that facility. Persist pack edits: Tauri **`save_role_slot_registry`** (toolbar add/remove instances; **at least one `llm`**; **last `llm` cannot be removed**); then **`invalidate_role_cache`** + **`load_role`**. Session overrides: **`set_session_slot_override`** (in-memory only).

---

## Design rules

- **Backends = compile-time enums**: legacy via `settings.json`; v2/v3/v4 via **`slot_registry`**. No dynamic `cdylib` loading.
- **Default implementations** are the built-in Rust paths; switching backend **does not rename API fields** (especially **`SendMessageResponse.reply`**).
- **Remote:** the host speaks **HTTP JSON-RPC** ([REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)). Missing `OCLIVE_REMOTE_*` URLs → fall back to builtin / in-process LLM with logs.
- **Directory:** child processes discovered under the host's plugin roots; same JSON-RPC wire as Remote. Current blueprint instances select `manifest.id` through `plugin` / `plugins`; only legacy v1 uses `plugin_backends.directory_plugins` ([DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)).
- **Auto-attachment:** `slot_attachment.backend` must be legal for its declared slot type and survive the same final-blueprint validation. OpenAI-compatible LLMs use the `remote` backend; `openai_compatible` is an implementation mode, not a blueprint backend token.

---

## `PluginBackends` host slots

Runtime struct **`PluginBackends`** has **six** enum fields: **`memory` · `emotion` · `event` · `prompt` · `llm` · `agent`**. For current packs it is a folded compatibility view of the effective `slot_registry`; optional **`directory_plugins`** carries directory ids after that fold and is not the pack SSOT. **`PluginHost`** binds each stable facade to **`Arc<dyn …>`**, while `SlotRunner` retains instance semantics. **`complex_emotion`** is resolved as a facility type in the open registry, not as a seventh stable slot ([OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md), [SETTINGS_REFERENCE.md](../../creator-docs/cli/SETTINGS_REFERENCE.md) §II).

### Module numbering (aligned with architecture overview)

| # | `slot_registry.type` (legacy key) | Kind |
|---|-----------------------------------|------|
| Module 1 | `memory` | Backend module |
| Module 2 | `emotion` | Backend module |
| Module 3 | `event` | Backend module |
| Module 4 | `prompt` | Backend module |
| Module 5 | `llm` | Backend module |
| Module 6 | `agent` | Backend module |
| Facility submodule 1 | `complex_emotion` *(no legacy six-slot key)* | Complex-emotion facility submodule |
| Facility submodule 2 | *(no stable slot type; in orchestration)* | Expert-model facility submodule (expert routing) |

**Backend-module plugin modules** (Remote / directory, etc.) attach to **module K**; they do **not** consume a “module 7” host slot. Full rules: [OCLIVE_ARCHITECTURE_OVERVIEW.md](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md).

### Multi-instance execution truth

Same-type **last-wins folding into `PluginBackends` is only a compatibility view**; it is not the execution merge rule for every slot.

| Type | Current execution policy |
|------|--------------------------|
| `memory` | Serial retrieval, dedupe by memory id, then re-sort/truncate |
| `emotion` / `event` / `prompt` / `complex_emotion` | Serial last-wins |
| non-streaming `llm` | The last LLM entry's `policy`: default `ensemble` = serial last-wins, `fastest` = first concurrent success, `fallback` = first ordered success |
| streaming `llm` | All three policies currently normalize to serial last-wins; only the final instance emits tokens, avoiding mixed streams |
| `agent` | The folded single provider executes. Multiple entries / `plugins[]` only populate `merged_agent_directory_plugin_ids` diagnostics; `wrap_agent_if_merged` is a no-op. Tool-union execution remains `K-AGENT-MERGE-01` |

---

## `send_message` fixed stages and slot calls (co-present path)

Stable entry: [`chat_engine::process_message`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs) → [`dispatch_turn`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/dispatch.rs) → remote stub, remote-life, or [`turn_pipeline::execute_turn`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs). Co-present middle lives in [`co_present/run_middle.rs`](../../kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/co_present/run_middle.rs). The list below describes slot calls inside kernel-owned fixed stages, **not** a linear pipe in which all six slots call one another:

1. **Effective config and `PluginHost`:** `EffectiveSessionConfig` merges the pack registry with session instance overrides, folds the six stable types into the `PluginBackends` compatibility view, and binds the six **backend modules** through `PluginHost`; `SlotRunner` keeps instance semantics. Legacy v1 starts directly from `role.plugin_backends`.
2. **Agent (module 6):** after preflight but before ordinary co-present pre, attempted only for a normal user, non-staged turn. `handled=true` returns through `build_minimal_response` and short-circuits Stable chat. That branch independently invokes the emotion slot and persists minimal state/chat, but bypasses ordinary reply post-processing and `reply_mode`.
3. **Pre:** module 2 `emotion.analyze` for user input and module 1 `memory.rank_memories`, plus personality, relation, identity, and recent-context loading.
4. **First half of middle:** a rules-only event estimate is produced first and helps resolve Turn Thinking; deterministic Fast affect-intensity fallback and knowledge retrieval also run here. Only when policy allows does module 3 `event.estimate` invoke an LLM/plugin and replace the rules estimate. Module 4 later receives only a content-free signal from the prior stored hint, never the current turn's not-yet-produced hint.
5. **Event Ring:** the dialogue estimate crosses `kernel.chat.event_impact.estimated`; memory may propose `kernel.memory.recall.candidate`, with one-turn `kernel.memory.recollection.activated` on admission. See [EVENT_RING.md](EVENT_RING.md).
6. **Prompt (module 4):** `top_topic_hint` + `build_prompt` / `build_prompt_segments` consume resolved character, affect, event, relation, memory, and external-observation context.
7. **Main LLM (module 5):** `generate` / `generate_stream` produces the raw reply and optional `[EMO]` metadata.
8. **Post:** strip `[EMO]`; resolve current reply emotion / complex emotion with a valid main marker authoritative and remote/directory used only when the marker is missing or invalid. The semantic reply first feeds emotion, relation, memory, portrait, and other state consumers and persistence; when enabled, the next-turn hint is stored. Then the single reply post-processor, ordinary co-present `reply_mode`, chat append, and `SendMessageResponse` assembly run in that order. Post-processing is a side channel, not a seventh slot; an arbitrary multi-processor chain is not currently implemented.

---

## Backend enums (per slot, condensed)

| Slot | Values (meanings) |
|------|-------------------|
| **memory** | `builtin` · `remote` · `directory` · `local` (`builtin_v2` is a **deprecated read alias**, same as `builtin`; local uses `_local_plugins`; see [LOCAL_PLUGIN_BRIDGE_SPEC.md](../../creator-docs/plugin-and-architecture/LOCAL_PLUGIN_BRIDGE_SPEC.md)) |
| **emotion** | `builtin` · `remote` · `directory` (`builtin_v2` read alias) |
| **event** | `builtin` · `remote` · `directory` (`builtin_v2` read alias) |
| **prompt** | `builtin` · `remote` · `directory` (`builtin_v2` read alias) |
| **llm** | `ollama` · `remote` · `directory` |
| **agent** | `builtin` (ReAct + MCP) · `remote` · `directory` — see root **`AGENTS.md`**. |

Remote / directory failures generally **fall back** to builtin / ollama as documented in the full Chinese page and in code.

---

## Legacy v1 `settings.json` example (migration reference only)

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

For legacy v1 only, omitting `plugin_backends` defaults memory / emotion / event / prompt / **agent** to **builtin** and **`llm`** to **`ollama`**. New packs must use `pipeline.ocblueprint.slot_registry`; do not copy this as the current pack format.

---

## Session slot overrides (Tauri)

**`set_session_slot_override`** applies an in-memory patch by **role + optional session + `slot_key`** and does **not** rewrite the pack or survive a host restart. It can patch `backend`, `plugin`, `plugins`, `model`, and `local_memory_provider_id`, so directory ids can be selected per session. `get_role_info` / `load_role` expose `slot_registry_pack`, `slot_registry_effective`, and `slot_session_overridden_keys`; folded six-slot diagnostics remain under `plugin_backends_effective` and `plugin_backends_effective_sources`.

Later non-empty fields merge into an existing patch. An all-empty patch clears the complete override for that instance; prefer **`clear_session_slot_override`** for explicit clearing and **`clear_all_session_slot_overrides`** for the session namespace. The old **`set_session_plugin_backend`** command remains a thin wrapper for the six default instance keys, requires a blueprint registry, and cannot select arbitrary instance keys or directory plugin ids.

---

## Frontend alignment

TypeScript **`SendMessageResponse`** (`distros/shared/src/api/`) must match `models/dto.rs`: the assistant text field is **`reply`**. `personality_source`, `reply_is_fallback`, `schema`, `api_version` drive UI ([`replyPresentation.ts`](../../distros/shared/src/utils/replyPresentation.ts)).

**Plugin Manager V2** templates (`endpoint-config`, `slot-selector`, `switch-toggle`, …) and manifest `ui_schema` are documented in the **Chinese** PLUGIN_V1 tail section; behavior is the same in English builds.

---

## HTTP `POST /chat` & `personality_source`

`get_role_info`, `load_role`, and **`POST /chat`** (with `--api`) expose **`personality_source`** as **`vector` | `profile`**, aligned with pack **`evolution.personality_source`**.

For the complete RPC tables and manifest examples, open the **[full PLUGIN_V1 (ZH)](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)**.

### Side-channel capabilities (condensed)

Directory / remote plugins may declare **`provides`** beyond the six slots. Host-enforced side channels (not six-slot `SlotResolver`):

| `provides` / pattern | Channel | Notes |
|------------|---------|-------|
| `reply_post_process` | Reply Post-Processor | `config.json` → `reply_post_processor`; RPC `reply_post_process.process` |
| `theater_director` | Theater Scene Director | Distro `[theater].director_plugin`; RPC `theater.build_prompt` |
| `voice.asr` | Voice ASR (official) | Host UI via `plugin_rpc_invoke`; see ZH PLUGIN_V1 + [RFC §4.1 summary](../rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS_SUMMARY.md) |
| **`com.user.tts.*`** | Community TTS sidecar | Same **`voice.*` RPC namespace** as official voice; **not** K-VOICE-02 productization; **no** new runtime permissions — see below |
| `complex_emotion` | Slot type (v2) | Blueprint `type: complex_emotion` when `backend: directory` |

#### Capability Registry v1 (blueprint v4, read-only plan)

- A manifest may advertise a namespaced v4 capability in `provides`, but the Plan Compiler selects it only when the host has registered a real consumer. An arbitrary string does not expand kernel behavior.
- A directory Provider must pass manifest `schema_version: 1` validation, declare the capability, have an executable `process`, and satisfy dependencies, per-role enablement, and high-risk grants. Legacy process manifests without `permissions` still require `process:spawn`.
- Provider `version` is reported for diagnostics. The v4 envelope currently has no Provider API semver range, so the displayed version is not an API-compatibility promise.
- The first registered v4 consumer is Chat Pro `voice.asr`; other capabilities degrade or block until their consumer and call path exist.
- Neither entry point spawns a Provider or rewrites a role pack. `oclive doctor execution-plan` / pure Plan Compiler diagnostics do not probe devices, report `resource_coordination: not_evaluated`, and omit `resource_plan`; desktop `get_execution_plan_diagnostics` refreshes the Resource Coordinator and attaches a read-only candidate plan without executing transitions or starting a model.

DTO and implementation anchors: [`models/execution_plan.rs`](../../kernel/crates/oclive_kernel_types/src/models/execution_plan.rs) · [`capability_registry.rs`](../../kernel/crates/oclive_kernel_host/src/infrastructure/capability_registry.rs) · [`execution_plan.rs`](../../kernel/crates/oclive_kernel_host/src/domain/execution_plan.rs).

#### Community TTS (`com.user.tts.*`)

Community directory TTS plugins share the official **`voice.*` method namespace** and the same per-plugin authorization path. This documents the allowed RPC surface; it does **not** broaden host-global whitelists or implement ChatTTS/XTTS (K-VOICE-02).

| Item | Contract |
|------|----------|
| **Plugin ID** | `com.user.tts.*` (creator namespace; e.g. `com.user.tts.xtts-sidecar`) |
| **Bridge gate** | manifest **`bridge.invoke`** must include **`plugin_rpc_invoke`** ([DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)) |
| **RPC gate** | method ∈ **this plugin's** manifest **`rpcMethods`**; **`process`** block required; enforced by [`validate_rpc_method_for_manifest`](../../distros/desktop-tauri/src/api/plugin_bridge.rs) (**per-plugin allowlist**, not a host-global table) |
| **`provides`** | **No** separate `voice.tts` token. TTS-only sidecars **need not** declare `voice.asr`; plugins that also serve the ASR UI channel **may** declare **`voice.asr`** (same token as official; **no** new permission surface) |
| **Recommended minimal `rpcMethods`** | at least **`voice.speak`**; typical sidecars also declare **`voice.probe_tts`**, **`voice.warm`**, **`voice.list_tts_adapters`**. Full `voice.*` list: [RFC §4.1 (ZH SSOT)](../../creator-docs/rfc/RFC_SIDE_CHANNEL_CAPABILITY_ENHANCEMENTS.md#41-voiceasr-插件通道windows-已交付--宿主侧) — each method must be listed in **this** manifest to be invocable |

Host UI / `ui_slots` call declared methods via **`plugin_rpc_invoke`**; undeclared methods are rejected (same as official voice plugins). Unified resource coordination currently recognizes only official `com.oclive.voice.asr` with `bundled-cosyvoice2-zh`. Community `com.user.tts.*`, user-hosted HTTP, and cloud TTS are not treated as host-managed GPU runtimes merely because they share `voice.*` method names.

Full Chinese normative section: **[PLUGIN_V1 §社区 TTS](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md)**.

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
