# How to replace modules later (swappable stack cheat sheet)

Which **pieces the host already splits**, and **what to touch** to swap one. Contract detail stays in [PLUGIN_V1.md](PLUGIN_V1.md).

**Documentation hub**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Creator overview (env vars, HTTP methods, bring‑up, “hot update” limits)**: [CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md)  
**HTTP JSON‑RPC spec & samples**: [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)

[中文](../../creator-docs/plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)

---

## 1. Swappable modules

| Module | Role | Rust trait | Current blueprint instance | Default |
|--------|------|------------|----------------------------|---------|
| **Memory retrieval** | rank long-term memory, context, keyword search | `MemoryRetrieval` | `type: memory`; `builtin` / `remote` / `directory` / `local` / `none` | `BuiltinMemoryRetrieval`; directory uses instance `plugin` |
| **User sentence emotion** | text → seven-dim emotion | `UserEmotionAnalyzer` | `type: emotion`; `builtin` / `remote` / `directory` / `none` | `BuiltinUserEmotionAnalyzer` |
| **Event impact** | estimate event type and factor | `EventEstimator` | `type: event`; `builtin` / `remote` / `directory` / `none` | `BuiltinEventEstimator` |
| **Prompt assembly** | main system/user strings | `PromptAssembler` | `type: prompt`; `builtin` / `remote` / `directory` / `none` | `BuiltinPromptAssembler`; the healthy co-present path needs an effective prompt |
| **LLM** | model calls | `LlmClient` | `type: llm`; `ollama` / `remote` / `directory` / `none` | `ollama`: injected client; `remote`: `OCLIVE_REMOTE_LLM_URL`; directory uses instance `plugin` |
| **Agent** | tools / ReAct | `AgentProvider` | `type: agent`; `builtin` / `remote` / `directory` / `none` | `BuiltinReActAgent`; the folded single Agent is executable today, while `plugins[]` tool-union execution remains `K-AGENT-MERGE-01`; MCP under **`{app_data}/mcp-servers`** |
| **Long-term memory store** | SQLite rows | `MemoryRepository` | *not a stable slot; swap via infrastructure* | `SqliteMemoryRepository` |
| **Policies** | write gates, importance, … | `EmotionPolicy`, … | `config/policy.toml` scene profiles | `Default*` |

**Aggregate**: the host first builds an effective session `slot_registry`, folds its six stable types into the `PluginBackends` compatibility view, then [`PluginHost`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs) binds concrete implementations while `SlotRunner` retains instance semantics. `ResolvedRolePlugins` supplies all six facades for a turn.

---

## 2. Replace a **built‑in** (compile time — do this first)

1. **Implement the trait** — e.g. `kernel/crates/oclive_kernel_host/src/domain/your_memory_retrieval.rs` implementing `MemoryRetrieval` (or the matching trait).

2. **Register in `PluginHost`** — in [`plugin_host.rs`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs):
   - add a field, e.g. `memory_foo: Arc<dyn MemoryRetrieval>`;
   - construct `Arc::new(YourMemoryRetrieval)` in `new()`;
   - add a `match` arm in `memory_retrieval()`.

3. **Extend enum and validation** — add the variant to the matching enum, then update `oclive_validation::allowed_backends_for_type`, resolution, degradation, and tests. Keep the wire in `snake_case`.

4. **Pack** — set `"type": "memory", "backend": "your_variant"` on the target `pipeline.ocblueprint.slot_registry` instance. Treat a public backend-enum expansion under the Breaking process.

5. **Validate & docs** — update [PLUGIN_V1.md](PLUGIN_V1.md) tables; add tests if needed.

---

## 3. Replace **Remote** (HTTP sidecar — already wired)

- Set **`OCLIVE_REMOTE_PLUGIN_URL`**: when pack selects `remote` for memory/emotion/event/prompt, traffic goes there (methods in [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)).
- Set **`OCLIVE_REMOTE_LLM_URL`**: when `llm = remote`, main generation + tag tasks use that endpoint.
- Missing URLs: same as before — builtin / in‑process LLM fallback + warning.
- Sidecars can be any language as long as JSON‑RPC matches; no `chat_engine` surgery.

---

## 3b. **Directory** (`distros/chat-pro/plugins/` — same protocol as Remote)

- Set the target registry instance to **`backend: directory`** and put the plugin's `manifest.id` in `plugin` or, for merge-capable types, `plugins[]`.
- The host discovers configured plugin roots, spawns by manifest, reads the JSON-RPC **base URL** from stdout, then uses the same HTTP client as Remote.
- Whole shell, `directory_plugin_invoke`, dev mode, minimal sample: **[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)**.

---

## 4. Usually **not** switched through stable-slot backends

- **Process‑wide `LlmClient`**: swap gateway/cloud in [`infrastructure/llm.rs`](../../kernel/crates/oclive_kernel_host/src/infrastructure/llm.rs) + `AppState::new`, or use **`OCLIVE_REMOTE_LLM_URL`** ([REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)).
- **`MemoryRepository`**: vector DB etc. lives in storage — abstract separately or add a repository impl before binding to manifest.

---

## 5. File index

| Purpose | Path |
|---------|------|
| Host aggregate | `kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs` |
| Remote HTTP client | `kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/` |
| Directory scan / child / RPC URL | `kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/` |
| Runtime resolve | `AppState::resolved_plugins_for` — `kernel/crates/oclive_kernel_host/src/state/mod.rs` |
| Chat orchestration | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/turn_pipeline/mod.rs`, … |
| Test hook | `RoleManager::with_memory_retrieval` — `kernel/crates/oclive_kernel_host/src/domain/role_manager.rs` |
