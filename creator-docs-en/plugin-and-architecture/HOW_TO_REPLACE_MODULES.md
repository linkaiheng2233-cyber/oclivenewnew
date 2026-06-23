# How to replace modules later (swappable stack cheat sheet)

Which **pieces the host already splits**, and **what to touch** to swap one. Contract detail stays in [PLUGIN_V1.md](PLUGIN_V1.md).

**Documentation hub**: [../getting-started/DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)  
**Creator overview (env vars, HTTP methods, bring‑up, “hot update” limits)**: [CREATOR_PLUGIN_ARCHITECTURE.md](CREATOR_PLUGIN_ARCHITECTURE.md)  
**HTTP JSON‑RPC spec & samples**: [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)

[中文](../../creator-docs/plugin-and-architecture/HOW_TO_REPLACE_MODULES.md)

---

## 1. Swappable modules

| Module | Role | Rust trait | `settings.json` (`plugin_backends`) | Default |
|--------|------|------------|----------------------------------------|---------|
| **Memory retrieval** | rank long‑term memory, context, keyword search | `MemoryRetrieval` | `memory`: `builtin` / `remote` / `directory` / `local` (`builtin_v2` read alias) | `BuiltinMemoryRetrieval`; **`directory`** needs `directory_plugins.memory` → `distros/chat-pro/plugins/<id>/` |
| **User sentence emotion** | text → seven‑dim emotion | `UserEmotionAnalyzer` | `emotion`: … | same; **`directory`** → `directory_plugins.emotion` |
| **Event impact** | LLM estimates event type & factor | `EventEstimator` | `event`: … | same; **`directory`** → `directory_plugins.event` |
| **Prompt assembly** | main system/user strings | `PromptAssembler` | `prompt`: … | same; **`directory`** → `directory_plugins.prompt` |
| **LLM** | model calls | `LlmClient` | `llm`: `ollama` / `remote` / `directory` | `ollama`: injected client; `remote`: `OCLIVE_REMOTE_LLM_URL` JSON‑RPC, falls back to default LLM if unset; **`directory`** → `directory_plugins.llm` (child URL, no `OCLIVE_REMOTE_LLM_URL`) |
| **Agent** | tools / ReAct | `AgentProvider` | `agent`: `builtin` / `remote` / `directory` | `builtin`: `BuiltinReActAgent`; **`directory`** → `directory_plugins.agent`; MCP dir **`{app_data}/mcp-servers`** (same `app_data` as `PluginHost::new` arg 3) |
| **Long‑term memory store** | SQLite rows | `MemoryRepository` | *(not on `plugin_backends`; swap via infra)* | `SqliteMemoryRepository` |
| **Policies** | write gates, importance, … | `EmotionPolicy`, … | `config/policy.toml` scene profiles | `Default*` |

**Aggregate**: [`PluginHost`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs) wires concrete impls per enum; per turn use **`ResolvedRolePlugins`** from **`AppState::resolved_plugins_for`** for **memory / emotion / event / prompt / llm / agent**. `AppState.llm` remains the process‑wide default handle (same impl as `plugin_backends.llm = ollama`).

---

## 2. Replace a **built‑in** (compile time — do this first)

1. **Implement the trait** — e.g. `kernel/crates/oclive_kernel_host/src/domain/your_memory_retrieval.rs` implementing `MemoryRetrieval` (or the matching trait).

2. **Register in `PluginHost`** — in [`plugin_host.rs`](../../kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs):
   - add a field, e.g. `memory_foo: Arc<dyn MemoryRetrieval>`;
   - construct `Arc::new(YourMemoryRetrieval)` in `new()`;
   - add a `match` arm in `memory_retrieval()`.

3. **Extend the enum** — in [`models/plugin_backends.rs`](../../kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs) add a variant to `MemoryBackend` (or the right enum), **`serde(rename = "snake_case")`** aligned with JSON.

4. **Pack** — `"plugin_backends": { "memory": "your_variant" }` matching the enum name.

5. **Validate & docs** — update [PLUGIN_V1.md](PLUGIN_V1.md) tables; add tests if needed.

---

## 3. Replace **Remote** (HTTP sidecar — already wired)

- Set **`OCLIVE_REMOTE_PLUGIN_URL`**: when pack selects `remote` for memory/emotion/event/prompt, traffic goes there (methods in [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)).
- Set **`OCLIVE_REMOTE_LLM_URL`**: when `llm = remote`, main generation + tag tasks use that endpoint.
- Missing URLs: same as before — builtin / in‑process LLM fallback + warning.
- Sidecars can be any language as long as JSON‑RPC matches; no `chat_engine` surgery.

---

## 3b. **Directory** (`distros/chat-pro/plugins/` — same protocol as Remote)

- Pack sets **`plugin_backends.* = directory`** and fills **`directory_plugins`** per used slot with the plugin **`manifest.json` `id`** (matches `distros/chat-pro/plugins/<id>/`).
- Host scans `distros/chat-pro/plugins/`, spawns per manifest, reads JSON‑RPC **base URL** from stdout, then uses the same HTTP client as Remote (methods still in [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)).
- Whole shell, `directory_plugin_invoke`, dev mode, minimal sample: **[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)**.

---

## 4. Usually **not** switched via `plugin_backends`

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
