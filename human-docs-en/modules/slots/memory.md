# Slot pack · `memory` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/memory.md`](../../../human-docs/modules/slots/memory.md)
> Definition SSOT: [MODULE_MAP §4](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: blueprint `slot_registry.type: memory` → folded `PluginBackends.memory` (legacy v1: `settings.json.plugin_backends.memory`) · trait `MemoryRetrieval` · `pre.rs` retrieval. Kernel orchestration performs STM/LTM writes in `post_llm`; those writes are not authority granted by the retrieval trait.

**Three stores** (do not confuse): chat log (`chat_*`) ≠ `short_term_memory` ≠ `long_term_memory`. Deep dive: [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md).

Memory may propose `memory.recall.candidate`; an Event decision module may produce `memory.recollection.activated`. The event carries IDs and scores, not memory text. Text is fetched outside the ring only after admission for one-turn recollection context. Contract: [EVENT_RING](../../../creator-docs-en/plugin-and-architecture/EVENT_RING.md).

**Don't**: Declare a candidate already remembered · copy memory text into event payloads · use chat messages as memory truth · clear STM/LTM when deleting UI chat.

**Read next**: [01 architecture (EN)](../../01_ARCHITECTURE_SIMPLE.md) · [chat-storage pack (ZH)](../../../human-docs/modules/side-channels/chat-storage.md).
