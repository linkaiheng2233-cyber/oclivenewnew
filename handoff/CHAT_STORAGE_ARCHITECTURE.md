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

## Pluggable storage (SQLite + optional JSON mirror)

Runtime always constructs **`HybridConversationStore`** (SQLite authoritative). Legacy `chat_storage.backend` values (`hybrid` / `file` / `sqlite`) and env `OCLIVE_CHAT_STORAGE_BACKEND` map to an internal **`mirror: bool`** via [`resolve_mirror_enabled`](../../kernel/crates/oclive_kernel_host/src/infrastructure/chat_storage/factory.rs); explicit `chat_storage.mirror` in role `config.json` wins.

Selection order:

1. Env `OCLIVE_CHAT_STORAGE_BACKEND` (`hybrid`|`file`|`sqlite`) — legacy; `file`/`sqlite` only affect mirror default
2. Role pack `config.json` → `chat_storage.backend` or `chat_storage.mirror`
3. Default **`mirror: true`** (hybrid)

| Config | SQLite chat tables | JSON mirror under `{app_data}/chats/` | Search | Auto cleanup | Memory replay |
|--------|-------------------|----------------------------------------|--------|--------------|---------------|
| **`mirror: true`** (default) | ✅ authoritative | ✅ best-effort | ✅ | ✅ | ✅ |
| **`mirror: false`** (`sqlite` legacy) | ✅ authoritative | ❌ | ✅ | ✅ | ✅ |

`get_chat_storage_capabilities.backend_kind` still reports `hybrid`|`file`|`sqlite` for UI compatibility; internally only mirror differs.

Implementation: `hybrid_store.rs` + `factory.rs::resolve_mirror_enabled`.

## Legacy “three backends” note

Older docs described independent `file` / `sqlite` store implementations. Phase 3+ uses a **single hybrid store**; `file`/`sqlite` backend enum values remain deserializable for migration and only toggle mirror behavior.

## Investment boundary (2026-06-08)

| Backend / mode | Production use | Engineering policy |
|----------------|----------------|-------------------|
| **`hybrid`** (`mirror: true`, default) | **Yes — primary path** | Maintain; bugfixes and capability UI as needed |
| **`file` / `sqlite` legacy enum** | Mirror toggle only | **Keep compiling + minimal tests**; **no new features** |
| Independent file-only / sqlite-only stores | Retired | Do not revive without new RFC |

Do not extend chat storage surface area in v0.3.x; memory replay/search/cleanup ride the hybrid store + SQLite memory tables.

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
| **`location`** | `"role_pack"` \| `"global"` | **`global`** | Chat JSON mirror root: role pack `chats/` subdir or global `{app_data}/chats/`; falls back to global with warn if role pack dir is not writable |

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

`resolve_storage_root_with_role`: env `OCLIVE_CHAT_STORAGE_ROOT` → role `config.json` → `chat_storage.location` (`role_pack` uses `{role_pack_dir}/chats/`) → `app_settings.chat_storage_root` → default `{app_data}/chats/`.

**`location = "role_pack"`**: chat logs live under `{role_pack_dir}/chats/{role_id}/{scene_id}/`. If the role pack directory is not writable, the host falls back to the global path and logs a warn.

**Scheduled cleanup**: on app startup and every **24h**, roles with `auto_cleanup_*` policy run via `spawn_auto_cleanup_scheduler` (in addition to per-turn and manual `run_chat_auto_cleanup`).

## Frontend

- Settings → **存储管理** — backend label, capability-gated search/export/cleanup/replay, role→scene→session→message drill-down
- `distros/shared/src/api/chatStorage.ts` — all invoke wrappers
- `distros/shared/src/stores/chatStore.ts` — loads from `fetch_chat_messages`

### Scene bucket ↔ `uiStore.sceneId` 不变量（回归守门）

消息按 **role × scene** 分桶（`messageMap[roleId][sceneId]`）。**`hydrateFromStorage` 不再预加载桶**；冷启动/切角色统一走 `chatStore.bootstrapChatForRole`（`completeRoleBootstrap` / `onSwitchRole`）：按 `refreshRoleInfo` 后的互动模式解析场景、`await loadMessagesForRoleScene`、再 `beginNewChatSessionOnRestart` 折叠历史。`useMainShell` 的 `interactionMode` watch **不得** `immediate`（默认 `pure_chat` 会在角色信息就绪前误触发 `enterPureChatScene`）。`applySceneChange` 用 `loadedBucketKeys` 判断桶是否已拉取。守门：`chatStoreScene.test.ts`。

## Code map

- `distros/desktop-tauri/migrations/027_chat_storage.sql`
- `kernel/crates/oclive_kernel_host/src/infrastructure/chat_storage/` (`store_trait.rs`, `factory.rs`, `replay.rs`, `backends/`, …)
- `kernel/crates/oclive-cli/src/role_pack.rs` — `config.json` with `chat_storage.backend`
- `distros/shared/src/components/settings/ChatStorageSettingsPanel.vue`

## Out of scope (later)

- Per-role hot-swapping backend without restart
- Clearing `short_term_memory` when deleting chat only
- RemoteLife chat persistence
- FTS5 full-text index migration
- File-backend auto-cleanup (users manage JSON files directly)

## Related documentation

| Topic | Path |
|-------|------|
| CLI `chat_storage` keys | [`creator-docs/cli/SETTINGS_REFERENCE.md`](../creator-docs/cli/SETTINGS_REFERENCE.md) §六 |
| Backend selection guide | [`creator-docs/storage/STORAGE_BACKEND_GUIDE.md`](../creator-docs/storage/STORAGE_BACKEND_GUIDE.md) |
| Role pack `config.json` | [`distros/chat-pro/roles/README_MANIFEST.md`](../distros/chat-pro/roles/README_MANIFEST.md) · [`creator-docs/role-pack/ROLE_PACK_SPEC.md`](../creator-docs/role-pack/ROLE_PACK_SPEC.md) §9.5a |
| Conformance tests | `store_trait_tests.rs` **9** trait cases + module unit tests **27** total (`cargo test -p oclivenewnew-tauri --lib infrastructure::chat_storage`) |
