# Chat storage architecture (方案 C — phase 3 complete)

## Phase status

| Phase | Scope | Status |
|-------|--------|--------|
| **1** | SQLite authoritative + JSON mirror; frontend from backend API; IndexedDB retired; storage UI role→scene | **Done** |
| **2** | Export (Markdown/JSON); auto-cleanup; search; single-message delete/edit | **Done** |
| **3** | Pluggable backends (`hybrid` / `file` / `sqlite`); trait API surface; memory replay; CLI scaffold; file search/replay; capability UI | **Done — architecture complete** |

## Three stores (do not conflate)

| Store | Location | Purpose |
|-------|----------|---------|
| IndexedDB | `chatMessageDb.ts` | **Legacy** — one-time migration source only |
| Chat log (backend-dependent) | See **Backends** below | **Authoritative** chat log |
| `short_term_memory` / `long_term_memory` | SQLite | Orchestration memory for prompts |

**Do not** route `MemoryEngine` or archive LLM through chat JSON files. **Deleting chat logs never clears memory tables.**

## Pluggable backends

`AppState::conversation_store` is `Arc<dyn ConversationStore>`. Selection order:

1. Env `OCLIVE_CHAT_STORAGE_BACKEND` (`hybrid` \| `file` \| `sqlite`)
2. Role pack `config.json` → `chat_storage.backend` (scaffold / per-project default)
3. Default **`hybrid`**

| Backend | Chat data | Mirror files | Search | Auto cleanup | Memory replay | Capability UI |
|---------|-----------|--------------|--------|--------------|---------------|---------------|
| **`hybrid`** (default) | SQLite `chat_sessions` / `chat_messages` | `{app_data}/chats/` best-effort | ✅ SQLite LIKE | ✅ | ✅ | all enabled |
| **`file`** | JSON only under `{app_data}/chats/` | Same (authoritative) | ✅ directory scan (`role_id` required) | ❌ | ✅ | search + replay only |
| **`sqlite`** | SQLite only | None | ✅ SQLite LIKE | ✅ | ✅ | all enabled |

Implementation: `src-tauri/src/infrastructure/chat_storage/backends/{hybrid_store,file_store,sqlite_store}.rs`, factory in `factory.rs`.

Shared trait: `store_trait.rs` — `append_turn`, `list_sessions`, `fetch_messages`, `list_sessions_by_role`, plus optional methods with default `NotImplemented` (search, export, cleanup, delete/edit, stats, replay, `supports_*`).

Conformance tests: `store_trait_tests.rs` (all three backends).

## Capability detection (PATCH-1)

`get_chat_storage_capabilities` returns:

| Field | Purpose |
|-------|---------|
| `backend_kind` | `hybrid` \| `file` \| `sqlite` — shown in storage panel header (i18n) |
| `supports_search` | Show search box |
| `supports_replay` | Show「重新提取记忆」buttons |
| `supports_cleanup` | Show auto-cleanup settings / manual trigger |

Frontend (`ChatStorageSettingsPanel.vue`) loads capabilities on `onMounted` and uses `v-if` — no hard-coded backend checks in UI.

## Memory replay (phase 3)

Re-extract AI memories from stored chat history into `long_term_memory`:

- **Scope**: `session` \| `scene` \| `role` (Tauri `replay_memory_extraction`)
- **Strategy**: **Merge** — dedupe by keyword overlap; threshold from `ReplayTarget.similarity_threshold` or role `chat_storage.replay_similarity_threshold` (default **0.6**, clamped **0.1–1.0**); existing lines update `mention_count`; never overwrite content
- **Idempotent**: Re-running the same range does not duplicate memories
- **Execution**: `tokio::spawn` + `get_replay_progress(task_id)` polling (~800ms)
- **Reads**: Active `ConversationStore` (`fetch_messages`, `list_sessions`, `list_sessions_by_role`)
- **Writes**: Always SQLite `long_term_memory` (independent of chat backend; file backend holds `Arc<DbManager>` for writes only)
- **UI**: Settings → 存储管理 —「重新提取记忆」at role / scene / session levels

## Data flow

```
CoPresent post_llm (non-empty reply)
  → ConversationStore::append_turn (per-role FIFO cap)
  → SendMessageResponse { user_message_id, assistant_message_id, timestamps }
  → tokio::spawn: sync_mirror_append + apply_auto_cleanup (hybrid only; best-effort)

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
| `rebuild_chat_mirror` | Rebuild JSON from SQLite (hybrid; uses role cap when resolvable) |
| `migrate_indexeddb_to_backend` | Import buckets from frontend IDB (legacy) |
| `get_chat_storage_stats` | Role → scene disk + session counts (backend-aware) |
| `delete_role_chats` | SQLite + `{root}/{role_id}/` tree |
| `delete_scene_chats` | SQLite + `{root}/{role_id}/{scene_id}/` |
| `export_chat_session` | Export one session (`format`: `markdown` \| `json`) |
| `export_role_chats` | Export all sessions for role (Markdown single file; JSON combined document) |
| `search_chat_messages` | Search (hybrid/sqlite: LIKE; file: JSON scan under role) |
| `delete_chat_message` | Delete one row + rebuild mirror when applicable |
| `edit_chat_message` | Edit **user** message only + `edited_at` in metadata |
| `get_role_chat_storage_config` | Read `config.json` → `chat_storage` |
| `save_role_chat_storage_config_cmd` | Write `chat_storage` + invalidate role cache |
| `run_chat_auto_cleanup` | Manual cleanup for one role |
| **`get_chat_storage_capabilities`** | Backend kind + search/replay/cleanup flags |
| **`get_chat_storage_root`** / **`set_chat_storage_root`** | Effective / persisted mirror root (`app_settings`) |
| **`replay_memory_extraction`** | Start async memory replay; returns `task_id` |
| **`get_replay_progress`** | Poll replay progress / result counters |

### `SendMessageResponse` (CoPresent)

| Field | When set |
|-------|----------|
| `user_message_id` / `assistant_message_id` | After successful `append_turn` |
| `user_message_timestamp` / `assistant_message_timestamp` | RFC3339 from DB write |

## Configuration (`config.json` → `chat_storage`)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| **`backend`** | `hybrid` \| `file` \| `sqlite` | **`hybrid`** | Storage backend (see table above) |
| `max_messages_per_session` | `u32?` | host **500** | FIFO cap per session (user+assistant rows) |
| `auto_cleanup_days` | `u32?` | off | Delete sessions with `updated_at` older than N days |
| `auto_cleanup_max_sessions` | `u32?` | off | Keep at most N most-recent sessions per role |
| **`replay_similarity_threshold`** | `f64` | **0.6** | Memory replay dedupe similarity (0.1–1.0); higher = stricter, fewer duplicate memories merged |

When **both** cleanup policies are set, a session is kept only if it satisfies **both** (stricter retention).

Cleanup runs **async after `append_turn`** on hybrid/sqlite (failures logged only). Also via `run_chat_auto_cleanup` / trait `apply_auto_cleanup`. **File backend does not support auto-cleanup.**

### Scaffold (`oclive-cli init`)

Interactive step **「Chat history storage backend」** writes `chat_storage.backend` and default `replay_similarity_threshold` into generated role pack `config.json`. Env `OCLIVE_CHAT_STORAGE_BACKEND` overrides backend at runtime.

Creator guide: [STORAGE_BACKEND_GUIDE.md](../creator-docs/storage/STORAGE_BACKEND_GUIDE.md).

## Export formats

### Markdown

- Per message: `**{sender}** ({timestamp}): {content}`
- Session header: role, scene, created/updated range
- Role export: one file, `## Session:` sections

### JSON

- Session: mirror document (`schema_version: 1`, `messages[]` with `id`, `sender`, `content`, `timestamp`, optional `metadata`)
- Role export: **ZIP** of `{scene_id}/{session_id}.json` files (API returns **base64** + `content_encoding: "base64"`)
- **sqlite** backend: JSON generated from DB rows (no mirror file required)

## Search

- Trait `search_messages` — SQLite `LIKE` on hybrid/sqlite; **file backend scans JSON under `chats/{role_id}/`** (requires `role_id`; case-insensitive `contains`)
- Max **100** results per request; ordered by `created_at DESC`
- Returns `highlight_snippet` plus optional `context_before` / `context_after` (2 turns each on SQLite backends)
- **FTS5** reserved for future; interface stable

## JSON mirror (`schema_version: 1`)

Under `{root}/{role_id}/{scene_id}/{created_at_compact}_{session_id_prefix}.json`. FIFO capped to same limit as SQLite. **Not written** when backend is `sqlite`.

Env: `OCLIVE_CHAT_STORAGE_ROOT` → `app_settings.chat_storage_root` → default `{app_data}/chats/`.

**Scheduled cleanup**: on app startup and every **24h**, roles with `auto_cleanup_*` policy run via `spawn_auto_cleanup_scheduler` (in addition to per-turn and manual `run_chat_auto_cleanup`).

## Frontend

- Settings → **存储管理** — backend label, capability-gated search/export/cleanup/replay, role→scene→session→message drill-down
- `src/api/chatStorage.ts` — all invoke wrappers
- `src/stores/chatStore.ts` — loads from `fetch_chat_messages`

## Code map

- `src-tauri/migrations/027_chat_storage.sql`
- `src-tauri/src/infrastructure/chat_storage/` (`store_trait.rs`, `factory.rs`, `replay.rs`, `backends/`, …)
- `crates/oclive-cli/src/role_pack.rs` — `config.json` with `chat_storage.backend`
- `src/components/settings/ChatStorageSettingsPanel.vue`

## Out of scope (later)

- Per-role hot-swapping backend without restart
- Clearing `short_term_memory` when deleting chat only
- RemoteLife chat persistence
- FTS5 full-text index migration
- File-backend auto-cleanup (users manage JSON files directly)
