# Chat storage architecture (方案 C — phase 1 complete)

## Phase 1 status

- **Backend**: SQLite authoritative + JSON mirror; CoPresent `post_llm` append; per-role `max_messages_per_session` in `config.json`.
- **Frontend**: `chatStore` loads from `fetch_chat_messages`; IndexedDB is **offline cache only**; one-time `migrate_indexeddb_to_backend`.
- **UI**: Settings → **存储管理** — role → scene → session preview; delete at role/scene level.
- **Memory**: Deleting chat logs does **not** touch `short_term_memory` / `long_term_memory`.

## Three stores (do not conflate)

| Store | Location | Purpose |
|-------|----------|---------|
| IndexedDB | `chatMessageDb.ts` | Offline cache after successful API load; migration source |
| `chat_sessions` / `chat_messages` + JSON mirror | SQLite + `{app_data}/chats/` | **Authoritative** chat log |
| `short_term_memory` / `long_term_memory` | SQLite | Orchestration memory for prompts |

**Do not** route `MemoryEngine` or archive LLM through chat JSON files.

## Data flow

```
CoPresent post_llm (non-empty reply)
  → ConversationStore::append_turn (per-role FIFO cap)
  → SendMessageResponse { user_message_id, assistant_message_id, timestamps }
  → tokio::spawn sync_mirror_append

Frontend hydrate
  → migrate_indexeddb_to_backend (once, localStorage chat_storage_migrated)
  → fetch_chat_messages(session_id) → messageMap
  → saveBucketToIdb (cache)
  → on API failure: loadBucketFromIdb
```

## Session keys

- `session_id` = `srid` (`conversation_state_role_id(mrid, session_id)`).
- `role_id` column = manifest id (`mrid`).
- `scene_id` defaults to `default`.

## Tauri API

| Command | Purpose |
|---------|---------|
| `list_chat_sessions` | Sessions for role + scene |
| `fetch_chat_messages` | Paginated messages |
| `rebuild_chat_mirror` | Rebuild JSON from SQLite (uses role cap when resolvable) |
| `migrate_indexeddb_to_backend` | Import buckets from frontend IDB |
| `get_chat_storage_stats` | Role → scene disk + session counts |
| `delete_role_chats` | SQLite + `{root}/{role_id}/` tree |
| `delete_scene_chats` | SQLite + `{root}/{role_id}/{scene_id}/` |

### `SendMessageResponse` (CoPresent)

| Field | When set |
|-------|----------|
| `user_message_id` / `assistant_message_id` | After successful `append_turn` |
| `user_message_timestamp` / `assistant_message_timestamp` | RFC3339 from DB write |

## JSON mirror (`schema_version: 1`)

Under `{root}/{role_id}/{scene_id}/{created_at_compact}_{session_id_prefix}.json`. FIFO capped to same limit as SQLite.

## Configuration

| Source | Key | Default |
|--------|-----|---------|
| Env | `OCLIVE_CHAT_STORAGE_ROOT` | `{app_data}/chats/` |
| Role `config.json` | `chat_storage.max_messages_per_session` | `500` |

Changing the cap applies to **new writes only**; run `rebuild_chat_mirror` to realign an existing session file.

Example:

```json
{
  "chat_storage": {
    "max_messages_per_session": 200
  }
}
```

## Frontend migration

1. On first launch after upgrade, read IndexedDB / legacy Pinia map.
2. Call `migrate_indexeddb_to_backend` with `ImportChatBucket[]`.
3. Set `localStorage.chat_storage_migrated = true`.
4. On failure, retry next launch (IDB data kept).

## Role delete

`delete_role` → `delete_all_data_for_manifest_role` (chat tables) + `delete_mirror_tree_for_role`.

## Code map

- `src-tauri/migrations/027_chat_storage.sql`
- `src-tauri/src/infrastructure/chat_storage/`
- `src/stores/chatStore.ts`, `src/api/chatStorage.ts`
- `src/components/settings/ChatStorageSettingsPanel.vue`

## Out of scope (later)

- PluginHost `ConversationStore` directory backend
- Clearing `short_term_memory` when deleting chat only
- RemoteLife chat persistence
- In-app editor for `max_messages_per_session` (optional UI)
