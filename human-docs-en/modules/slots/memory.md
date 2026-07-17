# Slot pack · `memory` (EN summary)

> Full checklist (ZH): [`human-docs/modules/slots/memory.md`](../../../human-docs/modules/slots/memory.md)
> Definition SSOT: [MODULE_MAP §4](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: `plugin_backends` key `memory` · trait `MemoryRetrieval` · hooks `pre.rs` retrieve · `post_llm` STM/LTM writes.

**Three stores** (do not confuse): chat log (`chat_*`) ≠ `short_term_memory` ≠ `long_term_memory`. Deep dive: [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md).

**Don't**: Use chat messages as memory truth · clear STM/LTM when deleting UI chat · edit `slot_registry` in role-pack tasks (G1).

**Read next**: [01 architecture (EN)](../../01_ARCHITECTURE_SIMPLE.md) · [chat-storage pack (ZH)](../../../human-docs/modules/side-channels/chat-storage.md).
