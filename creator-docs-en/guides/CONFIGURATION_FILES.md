# Oclive configuration files

Common **read/write** paths for the **oclivenewnew** desktop host: locations, purpose, important fields. Paths assume the **default desktop layout**; with a custom `roles` dir, resolve against your real **`roles` parent** and **Tauri app data dir**.

**`{app_data}`** (below) means the effective `OCLIVE_APP_DATA`. Desktop/shared-kernel runs normally use the canonical brand directory (`%LOCALAPPDATA%/OCLive/data` on Windows); an explicit environment value may override it. The old Tauri `%APPDATA%/com.oclivenewnew.app` path is only a one-time migration source. `app.db`, user plugins, and host-plugin configuration are rooted in this effective directory; see [OCLIVE_APP_DATA.md](../kernel/OCLIVE_APP_DATA.md).

| File | Path |
|------|------|
| SQLite DB | `{app_data}/app.db` |
| Plugin UI state (v2) | `{app_data}/plugin_state.json` |
| Host plugin options | `{app_data}/oclive_host_plugins.json` |
| Last active role id | `{app_data}/oclive_last_role_id.txt` |
| User plugin scan root | `{app_data}/plugins/` |

**Code**: `kernel/crates/oclive_kernel_host/src/infrastructure/plugin_state.rs`, `directory_plugins/runtime/mod.rs`, `lib.rs` (`app_data_dir`).

[中文](../../creator-docs/guides/CONFIGURATION_FILES.md)

---

## 1. `plugin_state.json`

- **Path**: `{app_data}/plugin_state.json`
- **Purpose**: persist **directory plugin UI** tweaks **per `role_id`**: whole‑shell choice, per‑slot plugin order, hide a plugin’s contribution inside a slot, global disable list, **force iframe mode**, …
- **Format**: JSON; when `schema_version` is **`2`**, data lives under **`roles`**; legacy global blob migrates to **`legacy_v1`**.

**Key fields (v2)**

| Field | Meaning |
|-------|---------|
| `schema_version` | **`2`** = per‑role storage |
| `roles` | `role_id` → **`RolePluginState`**: `shell_plugin_id`, flattened `slots` (`PluginStateFile`) |
| `roles[...].slots.disabled_plugins` | globally disabled plugin ids |
| `roles[...].slots.slot_order` | e.g. `chat_toolbar` → ordered plugin ids |
| `roles[...].slots.disabled_slot_contributions` | plugin ids not rendered inside a slot |
| `roles[...].slots.force_iframe_mode` | when true, host **ignores** manifest **`vueComponent`** — slots + whole shell use iframe |
| `legacy_v1` | migrated legacy global state |

First load of a role without a record can seed from pack **`ui.json`** (`RolePluginState::from_ui_config`).

**UI**: **`Ctrl+Shift+F`** opens the plugin manager (enable/disable, slot order, per‑slot hide, reset to pack defaults).

---

## 2. `ui.json` (role pack)

- **Path**: pack root next to **`pipeline.ocblueprint`** (see [ROLE_PACK_SPEC.md](../../creator-docs/role-pack/ROLE_PACK_SPEC.md)).
- **Purpose**: author **recommended front‑end layout**: whole‑shell plugin, per official slot order/visibility, theme/layout, …
- **Format**: JSON; machine schema **[role-pack/ui.json.schema.json](../../creator-docs/role-pack/ui.json.schema.json)**.

**Areas**

| Area | Meaning |
|------|---------|
| `shell` | recommended whole‑shell plugin `manifest.id` (string) |
| `slots` | `chat_toolbar`, `settings_panel`, `role_detail`, `sidebar`, `chat_header`, … with `order`, `visible` |
| `theme` | primary color, … (when defined in schema) |
| `layout` | layout keys (when defined in schema) |

**Split with backend configuration**: **`ui.json`** owns front-end/plugin layout; **`pipeline.ocblueprint.slot_registry`** owns backend capability instances. Legacy `settings.json` is migration-only. See §5 below.

---

## 3. `oclive_last_role_id.txt`

- **Path**: `{app_data}/oclive_last_role_id.txt`
- **Purpose**: single line, last **successfully switched / used role id** for **`get_directory_plugin_bootstrap`** etc. when `role_id` is omitted (works with `plugin_state`).

---

## 4. `manifest.json` (directory plugin)

- **Path**: each plugin root **`manifest.json`** (scan roots: `<roles parent>/plugins/`, `./plugins/`, `{app_data}/plugins/`, …; this monorepo uses `distros/chat-pro/plugins/` — [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1).
- **Purpose**: id, version, whole shell, child process, UI slots, bridge allowlist, dependencies, …
- **Detail**: [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §2; **versions** must be **SemVer** the host can parse (`load_from_dir`).

---

## 5. `pipeline.ocblueprint` (pack core)

- **Path**: pack root. New packs must not keep legacy `manifest.json` / `settings.json` beside it.
- **Purpose**: `meta` stores role metadata, personality, and scenes; **`slot_registry`** stores open instances and their `type`, `backend`, `plugin` / `plugins`, model, and related fields. See [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md) and [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md).
- **vs `ui.json`**:
  - **`pipeline.ocblueprint.slot_registry`**: **back-end** — e.g. a `type: memory` instance with `backend: directory` and `plugin` pointing to the directory-plugin `manifest.id`.
  - **`ui.json`**: **front‑end** — which plugins appear in toolbar/settings, plus **`theme` / `layout`** when used.

Migration tooling can still read legacy `settings.json` `plugin_backends.directory_plugins`, but it is not the disk authority for v2/v3/v4 packs.

---

## 6. `adult_extension.json` (optional Chat Pro role-pack extension)

- **Path**: role-pack root next to `pipeline.ocblueprint`.
- **Purpose**: adult-state persona, dialogue guidance, base-scene adult directions, and creator pacing recommendations, loaded only after local adult confirmation plus the Chat Pro global and per-role gates.
- **Compatibility**: not part of the universal base pack; other distros may ignore it. The base persona, identities, and scenes must work without this file.
- **Validation**: current `schema_version` is `1`, `character_is_adult` must be `true`, and scene keys must reference base scenes. See [ROLE_PACK_SPEC.md](../role-pack/ROLE_PACK_SPEC.md#chat-pro-adult-role-extension-adult_extensionjson-optional).

---

## 7. `oclive_host_plugins.json` (optional)

- **Path**: `{app_data}/oclive_host_plugins.json`
- **Purpose**: developer mode, extra plugin roots, default whole‑shell id, … ([DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1 and the `oclive_host_plugins.json` table).

---

## Links

- [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)
- [../getting-started/ERROR_CODES.md](../getting-started/ERROR_CODES.md)
