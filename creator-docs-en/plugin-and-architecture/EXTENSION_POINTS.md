# Extension points index (host ↔ swappable modules)

**Full documentation hub**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)

Aligned with [PLUGIN_V1.md](PLUGIN_V1.md): current packs declare instances and backends in `pipeline.ocblueprint.slot_registry`. The six stable types are memory, emotion, event, prompt, llm, and agent; `complex_emotion` is a facility type. Compile-time enums and registered providers still bound the implementations, and remote/directory do not gain kernel authority. Legacy v1 `settings.json` → `plugin_backends` is migration-only.

**How to replace implementations**: [HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md). **HTTP sidecar JSON‑RPC**: [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md).

[中文](../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)

---

## Host aggregation

- **`PluginHost`**: holds `Arc<dyn Trait>` per backend and dispatches by enum — [`kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs). **Remote** slots use the HTTP client under [`kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/`](../../kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/) when `OCLIVE_REMOTE_*` URLs are set. **Directory** slots call [`DirectoryPluginRuntime::ensure_rpc_url`](../../kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/runtime/mod.rs) to lazily spawn a child, then reuse the same HTTP client stack.
- **`ResolvedRolePlugins`**: the session path folds the six stable types from the effective registry and resolves **memory / emotion / event / prompt / llm / agent** once for the turn. No-session and legacy paths can still resolve role defaults.

## Rust traits and source files

| Capability | Trait / type | Default impl | Source |
|------------|--------------|--------------|--------|
| Memory ranking / context | `MemoryRetrieval` | `BuiltinMemoryRetrieval` (`builtin_v2` is a read-compat alias only; no separate V2 impl, see D-SLOT-01) | `kernel/crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs` |
| User‑sentence emotion | `UserEmotionAnalyzer` | `BuiltinUserEmotionAnalyzer` | `kernel/crates/oclive_kernel_runtime/src/domain/user_emotion_analyzer.rs` |
| Event impact | `EventEstimator` | `BuiltinEventEstimator` | `kernel/crates/oclive_kernel_host/src/domain/event_estimator.rs` |
| Prompt assembly | `PromptAssembler` | `BuiltinPromptAssembler` | `kernel/crates/oclive_kernel_runtime/src/domain/prompt_assembler.rs` |
| LLM | `LlmClient` (`type: llm`: `ollama` / `remote` / `directory` / `none`) | Injected `OllamaClient`; `remote` uses `OCLIVE_REMOTE_LLM_URL`; directory uses instance `plugin` | `kernel/crates/oclive_kernel_host/src/infrastructure/llm.rs`, `remote_plugin/` |
| Agent | `AgentProvider` (`type: agent`: `builtin` / `remote` / `directory` / `none`) | `BuiltinReActAgent`; directory uses instance `plugin` / `plugins`; MCP roots under `app_data_dir` | `kernel/crates/oclive_kernel_host/src/domain/agent.rs`, `mcp_client.rs` |
| Long‑term memory persistence | `MemoryRepository` | SQLite | `domain/repository.rs`, `infrastructure/repositories` |
| Policies | `EmotionPolicy`, … (trait: `kernel/crates/oclive_kernel_contracts/src/policy.rs`) | `Default*` (`kernel/crates/oclive_kernel_runtime/src/domain/policy.rs`) | wiring: `kernel/crates/oclive_kernel_host/src/infrastructure/policy_registry.rs` |

**World knowledge** (`distros/chat-pro/roles/{id}/knowledge/*.md`, optional current blueprint `meta.knowledge`; legacy manifest/settings remain migration-readable) is **pack resources + prompt/rules**—**not** selected through six-slot backends; see [WORLDVIEW_KNOWLEDGE.md](../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md).

## Runtime selection

- **`AppState::resolved_plugins_for_session(role, session_namespace)`**: the main chat path resolves all six subsystems from the effective registry — [`kernel/crates/oclive_kernel_host/src/state/mod.rs`](../../kernel/crates/oclive_kernel_host/src/state/mod.rs).
- **`resolved_plugins_for(role)` / single-slot helpers**: no-session or compatibility paths; their folded `role.plugin_backends` input is not the current disk authority.
- **`RoleManager`**: holds [`ResolvedRolePlugins`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs); see [`role_manager.rs`](../../kernel/crates/oclive_kernel_host/src/domain/role_manager.rs).

## Frontend

- Reply presentation: [`distros/shared/src/utils/replyPresentation.ts`](../../distros/shared/src/utils/replyPresentation.ts). `get_role_info` / `load_role` expose pack/effective registries plus the folded six-slot diagnostics; UI should prefer the instance registry.

## External integration (roadmap)

- Sidecar JSON‑RPC: [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md).  
- Directory plugins: [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md).

## Contract extension envelope (V-CONTRACT Phase 0)

**Principle**: **minimal kernel interpretation, unlimited carriage** — the kernel understands only core fields; plugin-specific state travels in envelopes instead of endlessly growing `PromptInput` hint fields.

| Type | Location | Role |
|------|----------|------|
| `SlotExtension { schema_id, data }` | `oclive_kernel_types::slot_extension` | Opaque JSON envelope for slot plugin output; `schema_id` names the payload schema |
| `EmotionResult.extension` | `emotion.rs` | Heterogeneous projections beyond seven-dim scores (e.g. CHS triple); `#[serde(default)]`, absent when omitted |
| `ComplexEmotionOutput.extension` | `complex_emotion.rs` | Optional private fields from complex-emotion sidecars |
| `PromptInput.extra_sections` | `prompt.rs` | Host-orchestrated generic prompt blocks `{ title, body }[]`; rendered **before** the reply-quality anchor as `【title】\nbody` in order |

Phase 1–3 (capability negotiation via `plugin.describe`, per-slot `slot_state` table, fused-provider publishing) are tracked under **V-CONTRACT** / **V-FUSED** in `handoff/TECHNICAL_DEBT_INVENTORY.md`. **`SCHEMA_VERSION` unchanged**; six-slot enums and blueprint `slot_registry` keys unchanged.

## Contract evolution rules

Aligned with [BREAKING_CHANGE_PROCESS.md](../../handoff/BREAKING_CHANGE_PROCESS.md); extension-point specifics:

1. **Additive-only**: new DTO / JSON-RPC fields must be `#[serde(default)]` or optional at the protocol layer; older clients/plugins must parse when new fields are omitted.
2. **Enum evolution**: externally visible Rust enums that may grow should use `#[non_exhaustive]`; matches need `_` fallbacks or explicit degradation — never assume a closed variant set.
3. **Interpret vs carry**: orchestration depends only on **documented core fields**; new hint-like capabilities should prefer `SlotExtension` or `extra_sections` over new top-level `PromptInput` fields (existing fields stay compatible; further stacking is discouraged).
4. **Breaking changes**: removing fields, changing semantics, bumping `SendMessageResponse.schema` / `SCHEMA_VERSION`, or renaming six-slot keys → follow the Breaking process (compat layers, `oclive_validation`, contract docs, bilingual CHANGELOG).
5. **Remote protocol**: new methods (e.g. Phase 1 `plugin.describe`) are **optional**; not implemented = zero capability — must not force upgrades.
6. **Persistence**: attaching `extension` to types already stored in DB (e.g. `Memory`) requires a separate migration assessment (`slot_state` in Phase 2 is the preferred private-state channel).
