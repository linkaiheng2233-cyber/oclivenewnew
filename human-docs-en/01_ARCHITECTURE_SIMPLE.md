# 01 · Architecture (simple)

> **Next:** [02 Thirty-minute start](02_THIRTY_MINUTE_START.md) · **Full human edition (CN):** [human-docs/01](../human-docs/01_ARCHITECTURE_SIMPLE.md) · **Module registry:** [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md)

## One turn (co-present path)

```
UI → Tauri/HTTP → process_message → co_present → turn_pipeline → PluginHost (six slots)
```

- **Orchestration SSOT:** `kernel/crates/oclive_kernel_host/.../process_message.rs`
- **Blueprint `steps[]` does not schedule** the first turn — Rust code does.

## Three memory stores (do not conflate)

| Store | Purpose | In prompt? |
|-------|---------|------------|
| Chat log (`chat_messages`) | UI history, export, replay source | **No** |
| Short-term (`short_term_memory`) | Recent-turn buffer | **Yes** |
| Long-term (`long_term_memory`) | AI archive, decay | **Yes** |

Deleting chat **does not** clear memory tables. Details: [CHAT_STORAGE_ARCHITECTURE](../handoff/CHAT_STORAGE_ARCHITECTURE.md).

## Six slots + non-slots

| Slots | `memory` · `emotion` · `event` · `prompt` · `llm` · `agent` |
| Facilities | Complex emotion, expert routing, portrait, visual stage — **not** slot keys |
| Side channels | User identity, reply post-process, theater director API |

Per-slot definitions: [MODULE_MAP §4–§9](../handoff/MODULE_MAP_AND_HANDOFF.md).

## Key IDs

| Term | Meaning |
|------|---------|
| `mrid` | Manifest role id (pack folder) |
| `srid` | Session namespace id |
| `reply` | Response field name (not `response`) |

Glossary: [09_GLOSSARY.md](09_GLOSSARY.md)
