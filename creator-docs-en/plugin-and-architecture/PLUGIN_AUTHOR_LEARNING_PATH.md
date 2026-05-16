# Plugin author learning path

For **directory plugins**, **remote sidecars**, and **host slot** work. Contracts: [PLUGIN_V1.md](PLUGIN_V1.md). Diagram: [KERNEL_AND_MODULES_ARCHITECTURE.md](../getting-started/KERNEL_AND_MODULES_ARCHITECTURE.md).

---

## Beginner (~30 min)

| Step | Goal | Read |
|------|------|------|
| 1 | Learn **six host slots + `agent`** | [PLUGIN_V1.md](PLUGIN_V1.md) diagram and `PluginBackends` (`complex_emotion` etc. are scaffold keys, not host slots) |
| 2 | Map `plugin_backends` + `directory_plugins` | [SETTINGS_REFERENCE.md](../cli/SETTINGS_REFERENCE.md) |
| 3 | **builtin / remote / directory** | **builtin**: in-process defaults; **remote**: HTTP JSON-RPC; **directory**: `plugins/<id>/` child process + same wire ([DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md)) |

**Done when:** You can list allowed backends per slot and how `directory_plugins` maps to manifest `id`.

---

## Intermediate (~1–2 h)

| Topic | Read |
|-------|------|
| **Directory plugins** | [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) |
| **Remote** | [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md) |
| **Permissions & grants** | [PLUGIN_V1.md](PLUGIN_V1.md) permission section · [A4 closure](../../handoff/A4_CLOSURE_SUMMARY.md) |
| **Bridge `invoke`** | [BRIDGE_API_REFERENCE.md](BRIDGE_API_REFERENCE.md) |

**Done when:** You can declare required `permissions` and describe UX when grants are missing (error codes / fallback).

---

## Advanced (~half day)

| Topic | Read |
|-------|------|
| **Market flow** | [../roadmap/PLUGIN_WEB_SECTION.md](../roadmap/PLUGIN_WEB_SECTION.md) · [MARKET_LAUNCHER_INTEGRATION.md](../roadmap/MARKET_LAUNCHER_INTEGRATION.md) |
| **`oclive_validation`** | `crates/oclive_validation`; three-way test `src-tauri/tests/permission_three_way_consistency.rs` |
| **Debug** | Plugin manager **Ctrl+Shift+F**; [FAQ.md](../FAQ.md); [ERROR_CODES.md](../getting-started/ERROR_CODES.md); `tracing` |

**Done when:** You can ship a minimal directory or remote demo and debug it from the manager UI.

---

## Next

- **Directory LLM + llama.cpp (no Ollama for that role):** [`examples/directory-plugin-llamacpp/README.en.md`](../../examples/directory-plugin-llamacpp/README.en.md) (Chinese: [README.md](../../examples/directory-plugin-llamacpp/README.md))  
- Swap a built-in backend in Rust: [HOW_TO_REPLACE_MODULES.md](HOW_TO_REPLACE_MODULES.md)  
- Monolith: [RFC_OCLIVE_MONOLITH_MODE.md](../rfc/RFC_OCLIVE_MONOLITH_MODE.md) · [OCLIVE_CLI_GUIDE.md](../cli/OCLIVE_CLI_GUIDE.md)
