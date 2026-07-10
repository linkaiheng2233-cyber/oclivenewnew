# author.json (creator recommendations)

[中文](../../creator-docs/role-pack/AUTHOR_JSON.md)

Optional file at the role pack root alongside `manifest.json` and `settings.json` (`distros/chat-pro/roles/{id}/author.json`).

Reference in repo: `distros/chat-pro/roles/mumu/author.json` (copy + `recommended_plugins`; slot layout stays in `ui.json` in the same directory to avoid duplicating `suggested_ui`).

## Relationship to `ui.json`

- **`suggested_ui`** (optional, same JSON shape as `ui.json`): if present and **non-empty** (same as runtime `UiConfig::is_effectively_empty`), used as the seed for **plugin UI state** (`plugin_state.json`) and as the baseline for “reset to pack recommendation”.
- Otherwise baseline falls back to **`ui.json`** (legacy behavior).
- User overrides remain in app-data **`plugin_state.json`** (per role); not overwritten by `author.json`.

## Relationship to `settings.json`

- **`settings.json`** remains authoritative for engine fields such as **`plugin_backends`**; `author.json` does not replace it.
- **`suggested_plugin_backends`** (optional, same shape as `settings.json` → `plugin_backends`): suggestions only; host UI may write **session-level** backend overrides after user confirmation (not persisted to on-disk `settings.json`).

### Session-level vs future “user default backends”

- **Current behavior:** “Apply author suggested backends” in plugin management (or equivalent) writes **session-namespace** backend overrides for the current session; **not** a global default for all roles/sessions.
- **If product needs cross-session defaults:** add an app-data file (e.g. `user_plugin_backends.json`) or a global config field, inserted in the `effective_plugin_backends_for_session` resolution chain as “user default”; distinguish from session overrides here. `settings.json` stays the pack-shipped engine default.

## Field summary

| Field | Description |
|-------|-------------|
| `schema_version` | Recommended `1` |
| `summary` / `detail_markdown` | Role intro and detail (Markdown) |
| `recommended_plugins` | Recommended directory plugins: `id`, `version_range`, optional `slots`, `for_backends`, `optional`, `note` |
| `suggested_ui` | Same as `ui.json` |
| `suggested_plugin_backends` | Same as `plugin_backends` |

Implementation: `kernel/crates/oclive_kernel_types/src/models/author_pack.rs` in the oclivenewnew repo.
