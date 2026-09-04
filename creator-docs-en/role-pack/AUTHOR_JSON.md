# author.json (creator recommendations)

[中文](../../creator-docs/role-pack/AUTHOR_JSON.md)

Optional file at the role-pack root (`distros/chat-pro/roles/{id}/author.json`): beside `pipeline.ocblueprint` in current packs, or beside `manifest.json` and `settings.json` in legacy v1 packs.

Reference in repo: `distros/chat-pro/roles/mumu/author.json` (copy + `recommended_plugins`; slot layout stays in `ui.json` in the same directory to avoid duplicating `suggested_ui`).

## Relationship to `ui.json`

- **`suggested_ui`** (optional, same JSON shape as `ui.json`): if present and **non-empty** (same as runtime `UiConfig::is_effectively_empty`), used as the seed for **plugin UI state** (`plugin_state.json`) and as the baseline for “reset to pack recommendation”.
- Otherwise baseline falls back to **`ui.json`** (legacy behavior).
- User overrides remain in app-data **`plugin_state.json`** (per role); not overwritten by `author.json`.

## Relationship to role-engine configuration

- Current packs use **`pipeline.ocblueprint`** as authority: backend instances live in `slot_registry`, and Stable v4 runtime settings live in `runtime_config`. `author.json` does not replace either. `settings.json` is authoritative only for legacy v1.
- **`suggested_plugin_backends`** retains the six-slot `PluginBackends` compatibility shape. After user confirmation, the host maps it onto default blueprint instance keys (`memory` through `agent`) as **session slot overrides** without rewriting the pack; missing default keys are skipped.

### Session-level vs future “user default backends”

- **Current behavior:** “Apply author suggested backends” in plugin management (or equivalent) writes **session-namespace** backend overrides for the current session; **not** a global default for all roles/sessions.
- **If product needs cross-session defaults:** add a separate app-data or global configuration layer in effective slot resolution. Keep it distinct from session overrides and pack `slot_registry` defaults. No such layer exists today.

## Field summary

| Field | Description |
|-------|-------------|
| `schema_version` | Recommended `1` |
| `summary` / `detail_markdown` | Role intro and detail (Markdown) |
| `recommended_plugins` | Recommended directory plugins: `id`, `version_range`, optional `slots`, `for_backends`, `optional`, `note` |
| `suggested_ui` | Same as `ui.json` |
| `suggested_plugin_backends` | Six-slot compatibility shape; session suggestions for default instance keys, not blueprint authority |

Implementation: `kernel/crates/oclive_kernel_types/src/models/author_pack.rs` in the oclivenewnew repo.
