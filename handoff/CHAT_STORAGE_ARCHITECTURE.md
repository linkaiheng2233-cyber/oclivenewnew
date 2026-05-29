# Chat storage architecture (方案 C — phase 2 complete)

## Phase status

| Phase | Scope | Status |
|-------|--------|--------|
| **1** | SQLite authoritative + JSON mirror; frontend from backend API; IndexedDB retired; storage UI role→scene | **Done** |
| **2** | Export (Markdown/JSON); auto-cleanup; search; single-message delete/edit | **Done** |

## Three stores (do not conflate)

| Store | Location | Purpose |
|-------|----------|---------|
| IndexedDB | `chatMessageDb.ts` | **Legacy** — one-time migration source only |
| `chat_sessions` / `chat_messages` + JSON mirror | SQLite + `{app_data}/chats/` | **Authoritative** chat log |
| `short_term_memory` / `long_term_memory` | SQLite | Orchestration memory for prompts |

**Do not** route `MemoryEngine` or archive LLM through chat JSON files. **Deleting chat logs never clears memory tables.**

## Data flow

```
CoPresent post_llm (non-empty reply)
  → ConversationStore::append_turn (per-role FIFO cap)
  → SendMessageResponse { user_message_id, assistant_message_id, timestamps }
  → tokio::spawn: sync_mirror_append + apply_auto_cleanup (best-effort)

Frontend hydrate
  → fetch_chat_messages(session_id) → messageMap
  → IndexedDB cache optional / legacy migrate only
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
| `migrate_indexeddb_to_backend` | Import buckets from frontend IDB (legacy) |
| `get_chat_storage_stats` | Role → scene disk + session counts |
| `delete_role_chats` | SQLite + `{root}/{role_id}/` tree |
| `delete_scene_chats` | SQLite + `{root}/{role_id}/{scene_id}/` |
| **`export_chat_session`** | Export one session (`format`: `markdown` \| `json`) |
| **`export_role_chats`** | Export all sessions for role (Markdown single file; JSON → ZIP base64) |
| **`search_chat_messages`** | LIKE search on `chat_messages.content` (not memory tables) |
| **`delete_chat_message`** | Delete one row + rebuild mirror |
| **`edit_chat_message`** | Edit **user** message only + `edited_at` in metadata |
| **`get_role_chat_storage_config`** | Read `config.json` → `chat_storage` |
| **`save_role_chat_storage_config_cmd`** | Write `chat_storage` + invalidate role cache |
| **`run_chat_auto_cleanup`** | Manual cleanup for one role |

### `SendMessageResponse` (CoPresent)

| Field | When set |
|-------|----------|
| `user_message_id` / `assistant_message_id` | After successful `append_turn` |
| `user_message_timestamp` / `assistant_message_timestamp` | RFC3339 from DB write |

## Configuration (`config.json` → `chat_storage`)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_messages_per_session` | `u32?` | host **500** | FIFO cap per session (user+assistant rows) |
| `auto_cleanup_days` | `u32?` | off | Delete sessions with `updated_at` older than N days |
| `auto_cleanup_max_sessions` | `u32?` | off | Keep at most N most-recent sessions per role |

When **both** cleanup policies are set, a session is kept only if it satisfies **both** (stricter retention).

Cleanup runs **async after `append_turn`** (failures logged only). Also available via `run_chat_auto_cleanup`.

## Export formats

### Markdown

- Per message: `**{sender}** ({timestamp}): {content}`
- Session header: role, scene, created/updated range
- Role export: one file, `## Session:` sections

### JSON

- Session: mirror document (`schema_version: 1`, `messages[]` with `id`, `sender`, `content`, `timestamp`, optional `metadata`)
- Role export: **ZIP** of `{scene_id}/{session_id}.json` files (API returns **base64** + `content_encoding: "base64"`)

## Search

- `search_chat_messages(query, role_id?, limit, offset)` — SQLite `LIKE` on `chat_messages.content`
- Max **100** results per request; ordered by `created_at DESC`
- Returns `highlight_snippet` (match context) for UI
- **FTS5** reserved for future; interface stable

## JSON mirror (`schema_version: 1`)

Under `{root}/{role_id}/{scene_id}/{created_at_compact}_{session_id_prefix}.json`. FIFO capped to same limit as SQLite.

Env: `OCLIVE_CHAT_STORAGE_ROOT` → default `{app_data}/chats/`.

## Frontend

- Settings → **存储管理** — search, export, auto-cleanup, role→scene→session→message drill-down
- `src/api/chatStorage.ts` — all invoke wrappers
- `src/stores/chatStore.ts` — loads from `fetch_chat_messages`

## Code map

- `src-tauri/migrations/027_chat_storage.sql`
- `src-tauri/src/infrastructure/chat_storage/` (`export.rs`, `cleanup.rs`, `role_config.rs`, …)
- `src/components/settings/ChatStorageSettingsPanel.vue`

## Out of scope (later)

- PluginHost `ConversationStore` directory backend
- Clearing `short_term_memory` when deleting chat only
- RemoteLife chat persistence
- FTS5 full-text index migration
