# Writing Rust integration tests for swappable backends (ADAPTING_TEST_PLUGIN)

## `PluginHost` and `builtin_v2`

- **Smoke example**: [`src-tauri/tests/plugin_backends_v2_resolve.rs`](../../src-tauri/tests/plugin_backends_v2_resolve.rs)  
  Verifies that when `memory` / `emotion` / `event` / `prompt` are **`builtin_v2`** and `llm` is **`ollama`**, `PluginHost::resolve_for_role` resolves **six subsystem lines** (including default **`agent`**).
- **Constructing the host**: `PluginHost::new(llm, None, std::env::temp_dir())`  
  The third argument is the **app data root** (production app data; tests may use a **temp directory**) for MCP config scan and related subsystem init.

## LLM stand-in

- Use **`MockLlmClient`** (`src-tauri/src/infrastructure/`) implementing **`LlmClient`** so integration tests do not hit real Ollama.

## Directory / remote plugins

- **Remote**: see CI job **`remote-plugin-demo`** (minimal Python sidecar + `memory.rank` JSON-RPC).
- **Directory**: needs `DirectoryPluginRuntime` and on-disk `plugins/` layout; higher integration cost—prefer **unit tests + smoke** layering.

## See also

- [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)  
- [EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)
