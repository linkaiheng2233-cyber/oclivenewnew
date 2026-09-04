# Slot pack · `llm` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/llm.md`](../../../human-docs/modules/slots/llm.md)
> Definition SSOT: [MODULE_MAP §8](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: blueprint `slot_registry.type: llm` → folded `PluginBackends.llm` (legacy v1: `settings.json.plugin_backends.llm`) · trait `LlmClient` · hook `co_present` generate/stream.

**Do**: Ollama / remote / directory backends · blueprint `slot_registry` · for multiple LLMs, respect the last LLM entry's non-streaming `policy`: default `ensemble` (serial last-wins), `fastest`, or `fallback`. Streaming currently normalizes all three to serial last-wins.

**Don't**: Call LLM from UI for portrait pick · use `none` on co-present path · stack logic in Tauri `api/*.rs`.

**Read next**: [PLUGIN_V1](../../../creator-docs-en/plugin-and-architecture/PLUGIN_V1.md) (if mirrored) or [ZH PLUGIN_V1](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) · [DIRECTORY_PLUGINS](../../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md).
