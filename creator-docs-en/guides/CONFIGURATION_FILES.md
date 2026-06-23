# Oclive configuration files

Common **read/write** paths for the **oclivenewnew** desktop host: locations, purpose, important fields. Paths assume the **default desktop layout**; with a custom `roles` dir, resolve against your real **`roles` parent** and **Tauri app data dir**.

**`{app_data}`** (below): from Tauri `path_resolver().app_data_dir()` (on Windows often under `%APPDATA%` for the app id), **sibling to `app.db`**. Directory‑plugin related files live here (**not** under `app_data/oclive/` as a subfolder name).

| File | Path |
|------|------|
| SQLite DB | `{app_data}/app.db` |
| Plugin UI state (v2) | `{app_data}/plugin_state.json` |
| Host plugin options | `{app_data}/oclive_host_plugins.json` |
| Last active role id | `{app_data}/oclive_last_role_id.txt` |
| User plugin scan root | `{app_data}/distros/chat-pro/plugins/` |

**Code**: `kernel/crates/oclive_kernel_host/src/infrastructure/plugin_state.rs`, `directory_plugins/runtime.rs`, `lib.rs` (`app_data_dir`).

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

- **Path**: pack root next to **`settings.json`** / **`manifest.json`** (see [distros/chat-pro/roles/README_MANIFEST.md](../../distros/chat-pro/roles/README_MANIFEST.md)).
- **Purpose**: author **recommended front‑end layout**: whole‑shell plugin, per official slot order/visibility, theme/layout, …
- **Format**: JSON; machine schema **[role-pack/ui.json.schema.json](../role-pack/ui.json.schema.json)**.

**Areas**

| Area | Meaning |
|------|---------|
| `shell` | recommended whole‑shell plugin `manifest.id` (string) |
| `slots` | `chat_toolbar`, `settings_panel`, `role_detail`, `sidebar`, `chat_header`, … with `order`, `visible` |
| `theme` | primary color, … (when defined in schema) |
| `layout` | layout keys (when defined in schema) |

**Split with `settings.json`**: **`ui.json`** = **front‑end & plugin layout**; **`settings.json`** = **back‑end capabilities** (`plugin_backends`, `directory_plugins` ids, …). See §5 below.

---

## 3. `oclive_last_role_id.txt`

- **Path**: `{app_data}/oclive_last_role_id.txt`
- **Purpose**: single line, last **successfully switched / used role id** for **`get_directory_plugin_bootstrap`** etc. when `role_id` is omitted (works with `plugin_state`).

---

## 4. `manifest.json` (directory plugin)

- **Path**: each plugin root **`manifest.json`** (scan roots: `<roles parent>/distros/chat-pro/plugins/`, `./distros/chat-pro/plugins/`, `{app_data}/distros/chat-pro/plugins/`, … — [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1).
- **Purpose**: id, version, whole shell, child process, UI slots, bridge allowlist, dependencies, …
- **Detail**: [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §2; **versions** must be **SemVer** the host can parse (`load_from_dir`).

---

## 5. `settings.json` (pack core)

- **Path**: pack root.
- **Purpose**: runtime behavior: scenes, personality, **`plugin_backends`**, Ollama/remote, … (full keys in [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)).
- **vs `ui.json`**:
  - **`settings.json`**: **back‑end** — e.g. `plugin_backends.memory = "directory"` and **`directory_plugins`** slot → `manifest.id`.
  - **`ui.json`**: **front‑end** — which plugins appear in toolbar/settings, plus **`theme` / `layout`** when used.

---

## 6. `oclive_host_plugins.json` (optional)

- **Path**: `{app_data}/oclive_host_plugins.json`
- **Purpose**: developer mode, extra plugin roots, default whole‑shell id, … ([DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md) §1 and the `oclive_host_plugins.json` table).

---

## Links

- [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [BRIDGE_API_REFERENCE.md](../plugin-and-architecture/BRIDGE_API_REFERENCE.md)
- [../getting-started/ERROR_CODES.md](../getting-started/ERROR_CODES.md)
