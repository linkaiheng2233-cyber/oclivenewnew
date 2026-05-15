# Whole‑shell bridge API — full reference

Commands callable from **directory plugins** in an HTML whole shell or a **`shell.vueEntry`** host Vue page via **`OclivePluginBridge.invoke`** (or **`inject('oclive').invoke`** in native Vue slots).

**Implementation**: `src-tauri/src/api/plugin_bridge.rs` (`required_permission_token`, `dispatch_bridge_command`, `validate_bridge`).

**Prerequisite**: [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) §4.1–4.3 (whole‑shell bridge, sensitive command gate, event bus).

[中文](../../creator-docs/plugin-and-architecture/BRIDGE_API_REFERENCE.md)

---

## 1. Overview

### How to call

- **iframe whole shell**: the host injects `window.OclivePluginBridge` into `shell.entry` HTML — use **`OclivePluginBridge.invoke(command, params)`**.
- **Vue whole shell**: for `shell.vueEntry`, use **`const oclive = inject('oclive'); await oclive.invoke(command, params)`** — same backend as **`plugin_bridge_invoke`**.

Request body always includes:

- `pluginId` — plugin `manifest.id`
- `assetRel` — resource path for the current page (whole shell must match **`shell.entry`** or **`shell.vueEntry`** normalization, consistent with manifest `bridge` checks)
- `command` — string from the table below
- `params` — JSON object (per command)

### Declaring permissions

In **`manifest.json`**:

- **`shell.bridge.invoke`** (whole shell) or **`ui_slots[].bridge.invoke`** (slot pages): array of **command names** (e.g. `send_message`) or **permission aliases** (e.g. `read:conversation`) — either match grants access.
- **`shell.bridge.events`** / **`ui_slots[].bridge.events`**: host event names allowed for `OclivePluginBridge.listen` / `oclive.events` (see §4).

Undeclared commands are rejected (`[API_PERMISSION_DENIED]`).

### Deep integration (“sensitive” commands)

Besides **`bridge.invoke`**, these also require:

1. Root **`"type": "ocliveplugin"`** in the manifest  
2. Caller is **`shell.entry` HTML** or the **`shell.vueEntry` host Vue page** (**not** a `ui_slots` page)

Commands marked **Sensitive** in the table follow this rule.

Commands **not** requiring `ocliveplugin` (still must be listed in `invoke`): e.g. `get_role_info`, `list_roles`, `get_time_state`, `get_directory_plugin_bootstrap`.

---

## 2. Command table

Parameter / field names follow **JSON** (camelCase and snake_case mixed to match existing contracts).

| Command | Purpose | Permission alias | Sensitive | Example params | Example result (excerpt) |
|---------|---------|------------------|-----------|----------------|--------------------------|
| `send_message` | Send user text through the chat engine | `send_message` | yes | `{ "role_id": "my_role", "user_message": "hi" }` or top‑level **`text`** instead of `user_message` | Serialized `SendMessageResponse` |
| `get_conversation` | Read recent turns | `read:conversation` | yes | `{ "role_id": "my_role", "session_id": null, "limit": 50, "offset": 0 }` | `{ role_id, session_namespace, total, limit, offset, items: [...] }` |
| `switch_role` | Switch active role | `switch_role` | yes | `{ "role_id": "other_role" }` | `RoleInfo` |
| `get_roles` | List role summaries | `read:roles` | yes | `{}` | `RoleSummary[]` |
| `get_current_role` | Alias of `get_role_info` | `read:current_role` | yes | `{ "role_id": "...", "session_id": null }` or `{ "req": { ... } }` | `RoleInfo` |
| `get_role_info` | Runtime info for a role | `get_role_info` | no | same as above | `RoleInfo` |
| `list_roles` | Same as `get_roles` | `list_roles` | no | `{}` | `RoleSummary[]` |
| `get_time_state` | Virtual time state | `get_time_state` | no | `{ "roleId": "..." }` or `{ "role_id": "..." }` | `TimeStateResponse` |
| `get_directory_plugin_bootstrap` | Shell URL, slots, subscribed events, … | `get_directory_plugin_bootstrap` | no | `{ "roleId": "..." }` optional | `DirectoryPluginBootstrapDto` |
| `update_memory` | Write long‑term memory | `write:memory` | yes | `{ "role_id": "...", "content": "...", "importance": 0.5 }` | `{ "memory_id": "..." }` |
| `delete_memory` | Delete memory | `write:memory` | yes | `{ "role_id": "...", "memory_id": "..." }` | `{ "ok": true }` |
| `update_emotion` | Update emotion label | `write:emotion` | yes | `{ "role_id": "...", "emotion": "happy" }` | `{ "ok": true }` |
| `update_event` | Create / record event | `write:event` | yes | `{ "role_id": "...", "event_type": "...", "description": "..." }` | same as `create_event` |
| `export_conversation` | Export chat logs | `export:conversation` | yes | `{ "role_id": "...", "format": "json", "session_id": null }` | same as `export_chat_logs` |
| `import_role` | Import a pack | `import:role` | yes | `{ "path": "C:/path.zip", "overwrite": false }` or **`src_path`** | `{ "role_id": "...", "ok": true }` |
| `delete_role` | Delete local role | `delete:role` | yes | `{ "role_id": "..." }` or `{ "roleId": "..." }` | same as `delete_role` command |
| `update_settings` | Update whitelisted app settings | `write:settings` | yes | see `update_settings_impl` | host‑specific |
| `get_conversation_list` | Session metadata list | `read:conversations` | yes | `{}` | `{ "items": [ { session_namespace, turn_count, last_at } ] }` |
| `update_prompt` | Dynamic prompt fragment (reserved) | `write:prompt` | yes | (not wired) | `{ "ok": false, "error": "not_implemented", ... }` |

**Notes**

- `get_roles` and `list_roles` both call `list_roles_impl`.
- Missing params → **`[INVALID_PARAMETER]`** string errors (see §3).

---

## 3. Error codes

Failures are formatted as **`[CODE] message`**; the frontend may parse the leading `CODE` (`src/utils/tauri-api.ts`: `parseApiErrorCode`, `toFriendlyErrorMessage`).

| Code | Meaning |
|------|---------|
| `API_PLUGIN_NOT_FOUND` | `plugin_id` not in scanned plugins |
| `API_PERMISSION_DENIED` | Missing bridge permission, wrong `type`, caller not whole‑shell entry, … |
| `API_INVALID_MANIFEST` | `manifest.json` failed to load/validate |
| `INVALID_PARAMETER` | Missing params, bad JSON, unknown `command`, … |
| `IO_ERROR` | Includes host JSON serialization failures |
| `DB_ERROR` / `ROLE_NOT_FOUND` / … | Same as other `AppError::to_frontend_error()` paths |

---

## 4. Subscribing to built‑in events

After declaring names in **`bridge.events`**, you may use `listen` / `oclive.events.on` for:

| Bus key | When | Suggested `data` (JSON‑serializable) |
|---------|------|--------------------------------------|
| `role:switched` | Active role changed | `{ "roleId": string }` |
| `message:sent` | User sent a message and the reply returned | `{ "message": string, "reply": string }` |
| `theme:changed` | Pack `ui.json` primary color applied | `{ "primaryColor": string }` |

Vue listeners using the **`oclive:`** prefix map **`oclive:role:switched`** → bus key **`role:switched`**.

---

## 5. Related docs

- [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) — directory plugins, `manifest`, `plugin_bridge_invoke`
- [../getting-started/ERROR_CODES.md](../getting-started/ERROR_CODES.md) — user‑visible errors
