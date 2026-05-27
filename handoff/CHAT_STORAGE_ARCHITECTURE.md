# Chat storage architecture (方案 C — step 1)

## Three stores (do not conflate)

| Store | Location | Purpose | Deleted with chat UI? |
|-------|----------|---------|------------------------|
| Frontend IndexedDB | `chatStore` / `chatMessageDb.ts` | UI history per `roleId × sceneId`, cap 500 | User clears browser data only (step 1 unchanged) |
| `chat_sessions` / `chat_messages` + JSON mirror | SQLite + `{app_data}/chats/` | **Authoritative chat log** for export / future UI | `delete_role` / session APIs (step 2+) |
| `short_term_memory` / `long_term_memory` | SQLite | **Orchestration memory** for prompts | Role delete clears with manifest; **not** cleared when only chat log is removed |

**Do not** route `MemoryEngine` or archive LLM prompts through chat JSON files. Chat persistence is decoupled from memory extraction.

## Scheme C (step 1)

1. **SQLite** (`chat_sessions`, `chat_messages`) is the single source of truth.
2. After each successful DB append, **best-effort** JSON mirror under `{root}/{role_id}/{scene_id}/`.
3. Mirror write failure → `tracing::warn` only; API still succeeds.
4. `rebuild_chat_mirror` Tauri command rebuilds mirror from DB.

```
CoPresent post_llm (non-empty reply)
  → ConversationStore::append_turn
       → upsert chat_sessions + insert 2 rows (transaction, FIFO 500)
       → tokio::spawn sync_mirror_append
```

## Directory layout

- Root: `OCLIVE_CHAT_STORAGE_ROOT` or `{app_data_dir}/chats/`
- Session files: `{root}/{role_id}/{scene_id}/{created_at_compact}_{session_id[0..8]}.json`
- No `project` segment.

## Session keys

- `session_id` in DB = conversation namespace `srid` (`manifest_id` or `manifest_id__sess__*`).
- `role_id` column = manifest role id (`mrid`).
- `scene_id` defaults to `default` when empty.

## Write scope (step 1)

| Path | Writes chat? |
|------|----------------|
| `TurnMode::CoPresent` + non-empty `reply` in `post_llm` | Yes |
| `RemoteLife` / `RemoteStub` / `Agent` shortcuts | No |
| Fallback replies | Yes (`reply_is_fallback` in assistant metadata) |

## Tauri API (step 1 — no frontend UI)

| Command | Args | Returns |
|---------|------|---------|
| `list_chat_sessions` | `roleId`, `sceneId`, `limit?`, `offset?` | `SessionMeta[]` |
| `fetch_chat_messages` | `sessionId`, `limit?`, `offset?` | `StoredMessage[]` |
| `rebuild_chat_mirror` | `sessionId` | mirror file path string |

### JSON mirror schema (`schema_version: 1`)

```json
{
  "schema_version": 1,
  "session_id": "...",
  "role_id": "...",
  "scene_id": "default",
  "created_at": "RFC3339",
  "updated_at": "RFC3339",
  "messages": [
    {
      "id": "uuid",
      "sender": "user|assistant",
      "content": "...",
      "timestamp": "RFC3339",
      "turn_index": 0,
      "metadata": {}
    }
  ]
}
```

Assistant `metadata` may include: `model`, `response_ms`, `reply_is_fallback`, `bot_emotion`. User rows may include `user_emotion`.

## Configuration

| Variable | Default |
|----------|---------|
| `OCLIVE_CHAT_STORAGE_ROOT` | unset → `{app_data}/chats/` |

Per-session cap: **500 messages** (250 turns), aligned with IndexedDB / short-term FIFO.

## Role delete

`delete_all_data_for_manifest_role` removes `chat_*` rows for manifest id and `__sess__*` namespaces. `delete_role` also removes `{root}/{role_id}/` mirror tree (best-effort).

## Code map

- Migration: `src-tauri/migrations/027_chat_storage.sql`
- Module: `src-tauri/src/infrastructure/chat_storage/`
- Hook: `domain/chat_engine/turn_pipeline/common.rs` (`post_llm`)
- `AppState::conversation_store`: `HybridConversationStore`

## Out of scope (step 1)

- Replacing IndexedDB in the UI
- PluginHost `ConversationStore` directory backend
- Clearing `short_term_memory` when deleting chat sessions
- RemoteLife chat persistence
