# Plugin author path

> **Audience**: Engineers building directory plugins or Remote backends.  
> **Time**: ~1–2 days to onboard.  
> **Chinese SSOT**: [`human-docs/paths/plugin-author.md`](../human-docs/paths/plugin-author.md)  
> **Next**: Pick a slot → [modules/slots/](../modules/slots/)

---

## Suggested order

1. [00 vision](00_VISION_AND_POSITIONING.md) — six slots · `builtin` / `remote` / `directory`
2. [02 thirty-minute start](02_THIRTY_MINUTE_START.md) — run main repo
3. [03 glossary](03_GLOSSARY.md) + [04 engineering rules summary](04_ENGINEERING_RULES_SUMMARY.md) — L3 discipline
4. **Slot packs** (~30–60 min each; skip L5 if slot-only work):
   - LLM backend → [modules/slots/llm.md](../modules/slots/llm.md)
   - Agent / MCP → [modules/slots/agent.md](../modules/slots/agent.md)
   - Memory backend → [modules/slots/memory.md](../modules/slots/memory.md)
5. [PLUGIN_AUTHOR_LEARNING_PATH](../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)
6. Example [`examples/directory-plugin-minimal/`](../examples/directory-plugin-minimal/)

---

## Contract essentials

| Topic | Doc |
|-------|-----|
| manifest | [PLUGIN_V1](../creator-docs/plugin-and-architecture/PLUGIN_V1.md) |
| permissions | `process:spawn`, `network:*` require user grant |
| six-slot wiring | blueprint `slot_registry` or legacy `plugin_backends` |
| packaging | `pack_plugin` Tauri command → `.oclive-plugin` |
| module definitions SSOT | [MODULE_MAP](../handoff/MODULE_MAP_AND_HANDOFF.md) — **do not copy tables into human-docs** |

---

## Debugging

- Directory plugins: `{app_data}/distros/chat-pro/plugins/`, `high_risk_grants.json`
- Log target: `oclive_plugin` (see [05 debugging](05_DEBUGGING.md))
- Pack editor: sister repo **oclive-pack-editor**

---

## Deep links

- [modules/ picker](../modules/README.md)
- [DIRECTORY_PLUGINS](../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [REMOTE_PLUGIN_PROTOCOL](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)
