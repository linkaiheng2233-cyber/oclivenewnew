# 01 · Architecture (simple)

> **Next:** [02 Thirty-minute start](02_THIRTY_MINUTE_START.md) · [06 Kernel learning path](../human-docs/06_KERNEL_LEARNING_PATH.md) (Chinese, detailed).

## One turn (stable path)

```
Tauri / HTTP → process_message → turn_pipeline
  pre_llm → build_prompt → main LLM → post_llm → persist
```

- **Orchestration SSOT:** `crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs`
- **Six slots:** resolved via `plugin_backends` + session overrides → `PluginHost`
- **Facilities** (complex emotion, expert routing): in-pipeline, not slot numbers

## Key IDs

| Term | Meaning |
|------|---------|
| `mrid` | Manifest role id (pack folder) |
| `srid` | Session namespace id |
| `reply` | Response field name (not `response`) |

Glossary: [09_GLOSSARY.md](09_GLOSSARY.md)

Chinese: [human-docs/01_ARCHITECTURE_SIMPLE.md](../human-docs/01_ARCHITECTURE_SIMPLE.md)
