# `settings.json` → `plugin_backends` authoritative reference (kernel-oriented)

This document describes configuration semantics shared by the **desktop host (Tauri)** and **`oclive-cli` scaffolds**. Single sources of truth remain code:

- Enums and structs: [`src-tauri/src/models/plugin_backends.rs`](../../src-tauri/src/models/plugin_backends.rs)
- Resolution and binding: [`src-tauri/src/domain/plugin_host.rs`](../../src-tauri/src/domain/plugin_host.rs)
- Protocol and tables: [`creator-docs/plugin-and-architecture/PLUGIN_V1.md`](../plugin-and-architecture/PLUGIN_V1.md)

**Standard JSON has no comments**: use **`_`-prefixed keys** for prose (ignored at load), or out-of-pack docs. `oclive-cli` sample packs use `_comment_*` keys per slot.

---

## I. Six host slots (`PluginBackends`)

The runtime struct **`PluginBackends`** has these **6** fields (Serde **ignores unknown fields**, so JSON may contain extra keys such as scaffold `complex_emotion` without host parse errors).

| Field | Facade trait (orchestration entry) | Common built-in (in-process) |
|-------|-----------------------------------|-------------------------------|
| `memory` | [`MemoryRetrieval`](../../src-tauri/src/domain/memory_retrieval.rs) | default `MemoryBackend::Builtin` |
| `emotion` | user emotion analysis (see `plugin_host` / `EmotionAnalyzer`) | `EmotionBackend::Builtin` |
| `event` | event impact estimation (`EventEstimator`) | `EventBackend::Builtin` |
| `prompt` | `PromptAssembler` / `PromptBuilder` | `PromptBackend::Builtin` |
| `llm` | `LlmClient` | **`LlmBackend::Ollama`** (default local client; **no `builtin` literal**) |
| `agent` | [`AgentProvider`](../../src-tauri/src/domain/agent.rs) | `AgentBackend::Builtin` |

When the whole `plugin_backends` block is omitted: memory / emotion / event / prompt / agent behave as **`builtin`**, **`llm` is `ollama`** (see PLUGIN_V1 examples).

### 1.1 Per-slot values (v1 enum summary)

Full table and JSON-RPC method names are in **[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)**. Common operator-facing values:

| Slot | Common values | When choosing `remote` / `directory` |
|------|---------------|--------------------------------------|
| memory | `builtin` / `builtin_v2` / `remote` / `directory` / `local` | `remote`: `OCLIVE_REMOTE_PLUGIN_URL`; `directory`: configure `directory_plugins.memory` |
| emotion | `builtin` / `builtin_v2` / `remote` / `directory` | same |
| event | `builtin` / `builtin_v2` / `remote` / `directory` | same |
| prompt | `builtin` / `builtin_v2` / `remote` / `directory` | same |
| llm | **`ollama`** / `remote` / `directory` | **`remote`**: `OCLIVE_REMOTE_LLM_URL`; **`OCLIVE_LLM_BACKEND`** may override at load |
| agent | `builtin` / `remote` / `directory` | `remote`: sidecar JSON-RPC; `directory`: configure `directory_plugins.agent` |

**Strings not in the v1 enum** (e.g. literal `none`) cause **role pack parse failure**. If scaffolds or docs say “none”, that means **logically off / undeclared**; for host-loadable JSON **omit the key** (fall back to default) or use a legal enum value.

### 1.2 `directory_plugins` object

When any slot is **`directory`**, fill **`manifest.id`** (string) for that slot under `plugin_backends.directory_plugins`. See [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md).

---

## II. Seventh slot: `complex_emotion` (scaffold & roadmap)

**Current `PluginBackends` has no such field.** `oclive-cli` writes `complex_emotion` inside **`plugin_backends`** for readability; the host **ignores** this key on deserialize without affecting `load_role`.

- If you need a separate process for experiments: same wiring as other `remote` subsystems, see [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md).
- Matches **`oclive-cli`** `CONFIG_REFERENCE.md` and the matrix at the end of **`init --help`**.

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

**Bench report JSON Schema** (`oclive bench`): [`crates/oclive-cli/schemas/oclive_bench_report.schema.json`](../../crates/oclive-cli/schemas/oclive_bench_report.schema.json) (relative paths assume repo clone layout).

See [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) section 4.

---

## VI. Related doc index

| Topic | Doc |
|-------|-----|
| CLI usage and flags | [OCLIVE_CLI_GUIDE.md](OCLIVE_CLI_GUIDE.md) |
| Preset table inside generated projects | **`CONFIG_REFERENCE.md`** after `init` |
| Plugins & sidecars overview | [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) |
| Directory plugins | [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) |
| Compile-time Monolith | [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) (`monolith.toml`, `build` / `bench`, dual `[[bin]]`) |
