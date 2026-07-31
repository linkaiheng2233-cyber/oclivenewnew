# Side channel pack · chat storage (EN summary)

> Full checklist (ZH): [`human-docs/modules/side-channels/chat-storage.md`](../../../human-docs/modules/side-channels/chat-storage.md)
> Architecture SSOT: [CHAT_STORAGE_ARCHITECTURE](../../../handoff/CHAT_STORAGE_ARCHITECTURE.md) · [MODULE_MAP §11](../../../handoff/MODULE_MAP_AND_HANDOFF.md)

**You plug in**: Independent channel (**not** slot `memory`) · `HybridConversationStore` · `chat_sessions` / `chat_messages` tables.

**Three stores** (do not confuse): ① chat log (`chat_*`) ≠ ② `short_term_memory` ≠ ③ `long_term_memory`. Memory slot does not use chat rows as retrieval truth.

**Do**: Chat UI replay · export · session APIs · `HybridConversationStore` impl and migrations · `replay_memory_extraction` (①→③ merge).

**Don't**: Treat `chat_messages` as memory retrieval source · clear STM/LTM when deleting UI chat · read `{app_data}/chats/` directly from memory slot.

**Read next**: [slots/memory](../slots/memory.md) · [01 architecture §three stores (EN)](../../01_ARCHITECTURE_SIMPLE.md) · `001_init.sql` `chat_*` tables.
