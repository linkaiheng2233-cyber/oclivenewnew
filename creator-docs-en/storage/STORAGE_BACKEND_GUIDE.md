# Chat storage backend selection guide

> Technical SSOT: [handoff/CHAT_STORAGE_ARCHITECTURE.md](../../handoff/CHAT_STORAGE_ARCHITECTURE.md) · Config fields: [SETTINGS_REFERENCE.md §VI](../cli/SETTINGS_REFERENCE.md) · Role pack `config.json`: [ROLE_PACK_SPEC.md §9](../role-pack/ROLE_PACK_SPEC.md).

## Three backends compared

| | **hybrid** (default) | **file** | **sqlite** |
|---|---------------------|----------|------------|
| Chat source of truth | SQLite + JSON mirror | JSON files | SQLite |
| Data paths | DB + `{app_data}/chats/{role}/{scene}/*.json` | `{app_data}/chats/{role}/{scene}/*.json` | DB only |
| Search | SQLite LIKE | JSON directory scan (`role_id` required) | SQLite LIKE |
| Auto cleanup | ✅ | ❌ | ✅ |
| Memory replay | ✅ | ✅ (memories still written to SQLite) | ✅ |
| Best for | Desktop app; transparent files + DB performance | Lightweight / embedded; user-managed JSON | Performance-first; no mirror files |

## How to switch

**Priority** (high → low):

1. Env **`OCLIVE_CHAT_STORAGE_BACKEND`** = `hybrid` \| `file` \| `sqlite` (process-wide; overrides role pack)
2. Role pack **`config.json`** → `chat_storage.backend`
3. Default **`hybrid`**

**Scaffold**: `oclive-cli init` interactive step “Chat history storage backend” writes `config.json` for generated packs.

**Note**: Switching backends requires a host restart; existing chat data is **not** auto-migrated between layouts.

## File backend limits

- **No auto-cleanup** (`supports_cleanup: false`); delete JSON under `{app_data}/chats/` manually
- Search requires a **role id** (storage settings panel searches in role context)
- Memory replay reads chat JSON but **writes** `long_term_memory` via host SQLite

## Replay similarity threshold

`config.json` → `chat_storage.replay_similarity_threshold` (default **0.6**, range **0.1–1.0**):

- **Higher** (e.g. 0.9): stricter dedupe, fewer “similar” memories merged
- **Lower** (e.g. 0.3): looser merge, more content treated as the same memory (`mention_count` increments)

Settings → Storage management → “Re-extract memories” reads the current role config before starting replay.

## Recommendations

| Scenario | Backend |
|----------|---------|
| Daily desktop chat, exports / visible mirror files | **hybrid** |
| Dev/debug, embedded deploy, users copy JSON directly | **file** |
| Many sessions, no disk mirror, lowest I/O | **sqlite** |

---

[中文](../../creator-docs/storage/STORAGE_BACKEND_GUIDE.md)
