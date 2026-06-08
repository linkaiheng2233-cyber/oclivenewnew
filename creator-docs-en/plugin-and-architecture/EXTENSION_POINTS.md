# Extension points index (host ↔ swappable modules)

**Full documentation hub**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)

Same model as [PLUGIN_V1.md](PLUGIN_V1.md): **v1 uses compile‑time enums** selected via `settings.json` → `plugin_backends`. Memory / emotion / event / prompt / **Agent** default to **builtin**; **`llm` defaults to `ollama`**. Each slot may instead use **`remote`** or **`directory`** (`plugins/*/manifest.json` child process — see [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)).

**How to replace implementations**: [HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md). **HTTP sidecar JSON‑RPC**: [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md).

[中文](../../creator-docs/plugin-and-architecture/EXTENSION_POINTS.md)

---

## Host aggregation

- **`PluginHost`**: holds `Arc<dyn Trait>` per backend and dispatches by enum — [`crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`](../../crates/oclive_kernel_host/src/domain/ports/plugin_host.rs). **Remote** slots use the HTTP client under [`src-tauri/src/infrastructure/remote_plugin/`](../../src-tauri/src/infrastructure/remote_plugin/) when `OCLIVE_REMOTE_*` URLs are set. **Directory** slots call [`DirectoryPluginRuntime::ensure_rpc_url`](../../src-tauri/src/infrastructure/directory_plugins/runtime.rs) to lazily spawn a child, then reuse the same HTTP client stack.
- **`ResolvedRolePlugins`**: `PluginHost::resolve_for_role(role)` resolves **memory / emotion / event / prompt / llm / agent** once per role and is **reused for a whole `send_message` / `RoleManager` turn** to avoid repeated matching.

## Rust traits and source files

| Capability | Trait / type | Default impl | Source |
|------------|--------------|--------------|--------|
| Memory ranking / context | `MemoryRetrieval` | `BuiltinMemoryRetrieval`, `BuiltinMemoryRetrievalV2` | `crates/oclive_kernel_runtime/src/domain/memory_retrieval.rs` |
| User‑sentence emotion | `UserEmotionAnalyzer` | `BuiltinUserEmotionAnalyzer`, … | `crates/oclive_kernel_runtime/src/domain/user_emotion_analyzer.rs` |
| Event impact | `EventEstimator` | `BuiltinEventEstimator`, … | `crates/oclive_kernel_host/src/domain/event_estimator.rs` |
| Prompt assembly | `PromptAssembler` | `BuiltinPromptAssembler`, … | `crates/oclive_kernel_runtime/src/domain/prompt_assembler.rs` |
| LLM | `LlmClient` (`plugin_backends.llm`: `ollama` / `remote` / `directory`) | Injected `OllamaClient`; `remote` when `OCLIVE_REMOTE_LLM_URL` set; **`directory`** uses **`directory_plugins.llm`** URL (see [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)); else built‑in fallback | `src-tauri/src/infrastructure/llm.rs`, `remote_plugin/` |
| Agent | `AgentProvider` (`builtin` / `remote` / `directory`) | `BuiltinReActAgent`; `directory` needs `directory_plugins.agent`; MCP roots under `app_data_dir` | `crates/oclive_kernel_host/src/domain/agent.rs`, `mcp_client.rs` |
| Long‑term memory persistence | `MemoryRepository` | SQLite | `domain/repository.rs`, `infrastructure/repositories` |
| Policies | `EmotionPolicy`, … (trait: `crates/oclive_kernel_contracts/src/policy.rs`) | `Default*` (`crates/oclive_kernel_runtime/src/domain/policy.rs`) | wiring: `crates/oclive_kernel_host/src/infrastructure/policy_registry.rs` |

**World knowledge** (`roles/{id}/knowledge/*.md`, optional manifest `knowledge`) is **pack resources + prompt/rules** — **not** switched via `plugin_backends`; see [WORLDVIEW_KNOWLEDGE.md](../../creator-docs/role-pack/WORLDVIEW_KNOWLEDGE.md).

## Runtime selection

- **`AppState::resolved_plugins_for(role)`**: resolves all six subsystems; **`chat_engine` prefers this** — [`src-tauri/src/state/mod.rs`](../../src-tauri/src/state/mod.rs).
- **`memory_retrieval_for` / …**: single‑slot helpers still parse full `role.plugin_backends` (including **`directory`** ids).
- **`RoleManager`**: holds [`ResolvedRolePlugins`](../../crates/oclive_kernel_host/src/domain/ports/plugin_host.rs); see [`role_manager.rs`](../../crates/oclive_kernel_host/src/domain/role_manager.rs).

## Frontend

- Reply presentation: [`src/utils/replyPresentation.ts`](../../src/utils/replyPresentation.ts). `get_role_info` / `load_role` echo **`plugin_backends`** for UI.

## External integration (roadmap)

- Sidecar JSON‑RPC: [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md).  
- Directory plugins: [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md).
