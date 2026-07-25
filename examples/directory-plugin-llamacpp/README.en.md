# Directory plugin example: LLM slot → llama.cpp (`com.oclive.example.llamacpp_llm`)

[中文](README.md)

This example forwards host **`llm.generate` / `llm.generate_tag`** calls (JSON-RPC; see [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §4.6) to a **local llama.cpp HTTP server**, so chat and low-temperature tag tasks can run **without Ollama** when the role pack uses **`plugin_backends.llm = directory`**.

## Requirements

- **Node.js 18+** (built-in `fetch`) for `rpc_server.mjs`.
- A **llama.cpp** build with an HTTP server you start yourself (not bundled in the host).

## Start llama.cpp server (example)

Adjust paths and port for your machine; the port must match **`OCLIVE_LLAMACPP_SERVER_URL`** (default `http://127.0.0.1:8080`).

```bash
# Common: OpenAI-compatible HTTP (this plugin tries /v1/chat/completions first)
llama-server -m /path/to/model.gguf --host 127.0.0.1 --port 8080
```

If your build has **no** `/v1/chat/completions`, the plugin falls back to **`POST /completion`** (`prompt` + `n_predict`). If both fail, align `rpc_server.mjs` with your server’s HTTP API or use a build that exposes the OpenAI-compatible layer.

## Install on the host

Copy this folder to:

`<roles parent>/plugins/com.oclive.example.llamacpp_llm/`

(e.g. next to repo `roles/` → `plugins/com.oclive.example.llamacpp_llm/`).

Or use **developer mode** `extra_plugin_roots` ([DIRECTORY_PLUGINS.md](../../creator-docs-en/plugin-and-architecture/DIRECTORY_PLUGINS.md) §1).

## High-risk grants

`manifest.json` declares **`process:spawn`** (host spawns Node) and **`network:*`** (Node calls local llama HTTP). Grant these in-app before first use ([PLUGIN_V1.md](../../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md) permissions, [DIRECTORY_PLUGINS.md](../../creator-docs-en/plugin-and-architecture/DIRECTORY_PLUGINS.md) §2).

For automation only: **`OCLIVE_SKIP_HIGH_RISK_GRANTS=1`** (not for end-user production builds).

## Role pack `settings.json` (excerpt)

`directory_plugins.llm` must match manifest **`id`**:

```json
{
  "plugin_backends": {
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "directory",
    "agent": "builtin",
    "directory_plugins": {
      "llm": "com.oclive.example.llamacpp_llm"
    }
  }
}
```

The host still passes **`effective_ollama_model`** as the **`model`** string; llama-server may ignore it or use it as a slot name—override mapping in `rpc_server.mjs` if needed.

## Environment variables

| Variable | Description |
|----------|-------------|
| **`OCLIVE_LLAMACPP_SERVER_URL`** | llama.cpp HTTP root; default **`http://127.0.0.1:8080`**. Read by the **plugin child process (Node)**; export in the shell before starting oclive or set system-wide. |

## Coexist with Ollama

- Roles that keep **`llm: "ollama"`** still use Ollama.
- Only packs that set **`directory`** + this plugin id use llama.cpp for that role—no Rust host changes.

## Files

| File | Role |
|------|------|
| `manifest.json` | Plugin id, `provides: ["llm"]`, process, `permissions` |
| `rpc_server.mjs` | JSON-RPC entry, proxy to llama.cpp HTTP |

## Troubleshooting

1. Confirm **`OCLIVE_READY http://...`** on plugin stdout (host handshake).  
2. `curl` the llama base URL; connection refused means the server is not up.  
3. Host logs **`remote_llm`** / **`oclive_plugin`**; JSON-RPC errors prefixed with **`llamacpp proxy:`** usually mean upstream non-2xx or unexpected JSON.  
4. For a long-lived proxy, consider **`plugin_backends.llm = remote`** with the same logic in a standalone sidecar ([REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)).

## Use as a LoRA expert plugin

This plugin can also target a LoRA model alias already registered by llama.cpp,
vLLM, or another OpenAI-compatible inference service. A non-empty
`adapter_model` in the plugin configuration overrides the base model name sent
by the host. The inference service remains responsible for loading weights,
VRAM management, and adapter hot switching.

Predeclare the plugin as a separate `llm + directory` blueprint instance:

```json
{
  "slot_registry": {
    "llm": {
      "type": "llm",
      "label": "Default LLM",
      "backend": "ollama",
      "position": 0
    },
    "mumu_lora": {
      "type": "llm",
      "label": "Mumu LoRA",
      "backend": "directory",
      "position": 10,
      "plugin": "com.oclive.example.llamacpp_llm",
      "zone": "experimental"
    }
  }
}
```

Select the same plugin id from an expert route:

```json
{
  "action": "slot.lora.apply",
  "params": {
    "plugin_id": "com.oclive.example.llamacpp_llm"
  }
}
```

Requirements:

- Build the host with `dual_core`; use blueprint v3 with
  `runtime_config.dual_core.enabled`.
- Include `slot.expert.invoke` and a final `slot.<llm-key>.generate` in
  `pipeline.experimental`.
- The plugin must declare `provides: ["llm"]` and receive its
  `process:spawn` / `network:*` grants.
- `adapter_model` is an inference-service model alias. The kernel intentionally
  does not interpret `.safetensors` or hard-code one framework's loader API.
- The plugin converts OpenAI-compatible SSE into OCLive
  `llm.generate_stream` NDJSON. Its manifest explicitly declares the method, so
  dual-core Stable completion forwards tokens incrementally. If the upstream
  lacks SSE, the plugin falls back to full generation and one callback.
- The selection is session-scoped. Invalid configuration, an unavailable
  plugin, or generation failure logs `LORA_ADAPTER_INVALID`,
  `LORA_ADAPTER_UNAVAILABLE`, or `LORA_ADAPTER_GENERATE_FAILED` and falls back
  to the normal LLM. A failure after streaming starts records
  `LORA_ADAPTER_STREAM_PARTIAL` and preserves the partial reply instead of
  appending a second model response.
