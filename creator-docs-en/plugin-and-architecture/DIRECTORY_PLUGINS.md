# Directory process plugins — architecture & contract

User paths **A1–C1**: scanning `distros/chat-pro/plugins/`, `manifest.json`, child‑process JSON‑RPC, **whole‑shell UI** (`https://ocliveplugin.localhost/…`), the unified façade commands **`directory_plugin_invoke`** / **`plugin_bridge_invoke`**, and **developer mode** extra roots.

**Wire format**: same as the HTTP remote sidecar (**POST JSON‑RPC 2.0**, `x-oclive-remote-protocol` header, …) — [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md).

**`plugin_backends`**: each slot may be **`directory`**; the nested object **`directory_plugins`** maps slots to **`manifest.id`** (below). Ready line: child stdout prints **`{ready_prefix} {rpc_url}`** (default prefix `OCLIVE_READY`, one line, space before URL).

[中文](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)

---

## 1. Layout & scan order

The host merges these **existing** roots; each **first‑level** subdirectory containing `manifest.json` is one plugin (registered by manifest `id`; later roots **override** duplicates with a log line):

1. **`<parent of roles>/distros/chat-pro/plugins/`** (sibling of `distros/chat-pro/roles/`; often `./distros/chat-pro/plugins/` in dev)  
2. **`./distros/chat-pro/plugins/`** (relative to process CWD)  
3. **`{app_data}/distros/chat-pro/plugins/`** under app data next to `app.db`

**Developer mode (C1)**: when `app_data/oclive_host_plugins.json` has **`developer_mode`: true**, or env **`OCLIVE_DEVELOPER=1`** (`true`/`yes` accepted), also scan each directory in **`extra_plugin_roots`** (each entry is a **container**; its first‑level children are plugin roots).

### `oclive_host_plugins.json` (optional, app data root)

| field | type | notes |
|-------|------|-------|
| `developer_mode` | `boolean?` | enables `extra_plugin_roots` |
| `extra_plugin_roots` | `string[]?` | extra container dirs |
| `shell_plugin_id` | `string?` | manifest `id` used for whole‑shell replacement |

Env **`OCLIVE_SHELL_PLUGIN_ID`** (non‑empty trim) overrides file `shell_plugin_id`.

---

## 2. `manifest.json` (plugin root)

| field | type | notes |
|-------|------|-------|
| `schema_version` | `number` | currently **`1`** only |
| `id` | `string` | globally unique; referenced by `directory_plugins.*` |
| `version` | `string` | SemVer text recommended |
| `shell` | `object?` | **`entry`**: HTML path relative to plugin root (whole shell B1 fallback); **`vueEntry?`**: `.vue` path; when **`force_iframe_mode` is off** and file reads succeed, host mounts Vue whole shell (same UX idea as slot `vueComponent`; else fallback `entry`) |
| `process` | `object?` | **`command`**, **`args[]`**, optional **`cwd`** (relative to plugin root; default = plugin root) |
| `ready_prefix` | `string?` | default **`OCLIVE_READY`**; ready line = prefix + space + **JSON‑RPC base URL** (`http`/`https`) |
| `dependencies` | `object?` | optional map **`other plugin id` → semver range**; missing / mismatch marks plugin disabled in manager |

**Lazy start**: first RPC need (`plugin_backends` **`directory`**, `directory_plugin_invoke`, or shell manifest resolution) spawns the child and caches **RPC URL** + **process** (today children are **not** recycled per role switch; released on app exit). Concurrent starts for the same `id` are locked.

---

## 3. Six backend slots (A2)

In `settings.json` `plugin_backends`:

- `memory` / `emotion` / `event` / `prompt` **`directory`** → use **`directory_plugins.<slot>`** id, lazy‑start, then same HTTP client as env remote (`memory.rank`, …).  
- `llm` **`directory`** → **`directory_plugins.llm`** URL; must implement **`llm.generate` / `llm.generate_tag`**.  
- `agent` **`directory`** → **`directory_plugins.agent`**; wire same as other remotes (methods per host + protocol).

If id missing, scan miss, spawn/handshake fails → log + fallback: **memory/emotion/event/prompt → builtin**, **llm → Ollama**, **agent → builtin**.

**Example (LLM slot → local llama.cpp HTTP, no Ollama):** repo [`examples/directory-plugin-llamacpp/`](../../examples/directory-plugin-llamacpp/README.en.md) — Node sidecar implements `llm.generate` / `llm.generate_tag` and forwards to `OCLIVE_LLAMACPP_SERVER_URL` (default `http://127.0.0.1:8080`) on `llama-server`. Set `plugin_backends.llm` to **`directory`** and `directory_plugins.llm` to this manifest **`id`** to coexist with roles that still use Ollama. Chinese: [../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) §3.

### Example (excerpt)

```json
{
  "plugin_backends": {
    "memory": "directory",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "directory",
    "directory_plugins": {
      "memory": "com.example.myplugin",
      "llm": "com.example.myplugin"
    }
  }
}
```

**`directory_plugins` source of truth**: pack **`settings.json`**. Rust `PluginBackendsOverride` can merge `directory_plugins` per slot, but today Tauri **`set_session_plugin_backend`** only overrides the six enums + `local_memory_provider_id` — **not** `directory_plugins`; use pack or future session APIs for per‑session ids.

---

## 4. Whole‑shell UI (B1)

When **`shell_plugin_id`** (file or `OCLIVE_SHELL_PLUGIN_ID`) points at a scanned plugin with **`shell.entry`**, the built‑in frontend calls **`get_directory_plugin_bootstrap`** before mounting the main app (`role_id` optional, legacy behavior).

- **`force_iframe_mode`** (bootstrap + `plugin_state`): if true, **ignore** **`shell.vueEntry`**; if **`shellUrl`** exists and differs from current document, **`location.replace(shellUrl)`** (HTML shell).  
- Else if non‑empty **`shell.vueEntry`** and **`read_plugin_asset_text`** succeeds: host mounts lightweight Vue root (**`DirectoryShellApp.vue`** + **`AsyncPluginVue`**); `inject('oclive')` like slots; **`plugin_bridge_invoke` `assetRel` must be the `vueEntry` path** (matches sensitive “whole shell page” checks).  
- Else if **`shellUrl`** differs: **`location.replace(shellUrl)`**.

**`shellUrl` shape**: `https://ocliveplugin.localhost/<manifest.id>/<entry>` (WebView2 maps custom scheme to that HTTPS host).

**Static assets**: host **`register_uri_scheme_protocol("ocliveplugin", …)`** reads from disk (path traversal → 403). **`tauri.conf.json`** needs **`dangerousRemoteDomainIpcAccess`** for that origin with **`enableTauriAPI`: true** (exact field names per Tauri version schema).

```json
"dangerousRemoteDomainIpcAccess": [
  {
    "domain": "https://ocliveplugin.localhost",
    "windows": ["main"],
    "enableTauriAPI": true,
    "plugins": ["*"]
  }
]
```

### 4.1 Whole‑shell bridge (`shell.bridge`)

If **`shell.bridge`** declares non‑empty **`invoke`** / **`events`**: for **`shell.entry` HTML** the host injects **`window.OclivePluginBridge`** before `</body>`; for **`shell.vueEntry`** Vue shell, **`provide('oclive', …)`** exposes the same **`invoke` / `events`** (still **`plugin_bridge_invoke`** underneath).

- **`invoke(command, params)`**: manifest **`bridge.invoke`** is the allowlist — command names or permission aliases.  
- **`listen(event, handler)`**: only **`bridge.events`** names (uses `__TAURI__.event` inside WebView).

**Deep integration**: commands in the sensitive table below also need root **`"type": "ocliveplugin"`** and caller must be **`shell.entry` HTML** or **`shell.vueEntry` page** — **not** `ui_slots` pages.

| `invoke` command | manifest token (any one in `invoke` array) | notes |
|------------------|---------------------------------------------|--------|
| `send_message` | `send_message` | `process_message`; may use `text` instead of `user_message` |
| `get_conversation` | `get_conversation` or **`read:conversation`** | `role_id`, optional `session_id` / `limit` / `offset` |
| `switch_role` | `switch_role` | `{ "role_id": "..." }` |
| `get_roles` | `get_roles` or **`read:roles`** | same as `list_roles` |
| `get_current_role` | `get_current_role` or **`read:current_role`** | alias of `get_role_info` |
| `update_memory` | **`write:memory`** or `update_memory` | `role_id`, `content`, optional `importance` |
| `delete_memory` | **`write:memory`** or `delete_memory` | `role_id`, `memory_id` |
| `update_emotion` | **`write:emotion`** or `update_emotion` | `role_id`, `emotion` |
| `update_event` | **`write:event`** or `update_event` | same as `create_event` |
| `export_conversation` | **`export:conversation`** or `export_conversation` | `format` `json`|`txt`, … |
| `import_role` | **`import:role`** or `import_role` | `path` or `src_path`, `overwrite` |
| `update_prompt` | **`write:prompt`** or `update_prompt` | reserved / `not_implemented` |
| `delete_role` | **`delete:role`** or `delete_role` | `role_id` or `roleId` |
| `update_settings` | **`write:settings`** or `update_settings` | whitelisted app fields |
| `get_conversation_list` | **`read:conversations`** or `get_conversation_list` | session metadata |

**Not** requiring `ocliveplugin` (still must list in `invoke`): `get_role_info`, `list_roles`, `get_time_state`, `get_directory_plugin_bootstrap`, … Undeclared calls are denied.

**Write‑class** commands in the table + **`export_conversation` / `import_role`** share the same **`type: ocliveplugin` + whole‑shell caller** rule as chat‑class sensitive commands.

### 4.2 Main UI slots (`ui_slots`)

Supported **`slot`** values:

| `slot` | Host location |
|--------|----------------|
| **`chat_toolbar`** | Above chat input — narrow toolbar |
| **`settings.panel`** | **Settings → Plugins** (More → Open settings) |
| **`role.detail`** | Left **role detail** (below portrait/name, above favorability) |
| **`sidebar`** | Left column **below role block** (above favorability bar) |
| **`chat.header`** | Chat column **above message list** |

Rules:

- No **`shell`** segment → declare embeds in **`ui_slots`**: **`entry`** HTML (iframe fallback).  
- Optional **`vueComponent`**: `.vue` path; host compiles in‑tree, else iframe to `https://ocliveplugin.localhost/<id>/<entry>`.  
- Plugins **with `shell`** do **not** contribute slots (avoid duplicate UI).  
- Slot pages calling the host: put **`bridge`** on the matching **`ui_slots[]`** entry. iframe injection only when asset URL matches **`entry`**; native Vue slots use **`inject('oclive')`**; `plugin_bridge_invoke` uses manifest **`entry`** as **`assetRel`**.  
- Examples: `examples/directory-plugin-ui-slot/` (iframe only); **`examples/directory-plugin-ui-slot-vue/`** (Vue + HTML fallback).

### 4.2.1 Native Vue slots (`vueComponent`)

| field | meaning |
|-------|---------|
| **`entry`** | required — iframe URL + bridge anchor (`assetRel` = normalized `entry`) |
| **`vueComponent`** | optional — `.vue` under plugin root; `export default` Vue 3; use **`const oclive = inject('oclive')`** |

**`oclive` object** (aligned with whole‑shell bridge, same `plugin_bridge_invoke` backend):

- **`oclive.invoke(command, params?)`**  
- **`oclive.pluginId` / `oclive.bridgeAssetRel`**  
- **`oclive.events.emit` / `on` / `off`** — host **mitt** bus (§4.3); `on` listeners removed on unmount.  
- **`oclive.events.request(event, data?, timeoutMs?)`** — request/response; event names **`pluginId:name`**; default timeout 15s; **`Promise.race`** if multiple handlers.  
- **`oclive.events.onRequest` / `offRequest`**

You may use host CSS variables (`--fluent-accent`, `--bg-primary`, … — `distros/shared/src/styles/theme.css`).

**Security**: slot code shares JS context with the app; **do not** use raw `window.__TAURI__` — use **`oclive.invoke`** only.

### 4.3 Event bus (built‑in)

| name | when | `data` |
|------|------|--------|
| **`role:switched`** | role changed | `{ roleId: string }` |
| **`message:sent`** | user sent + reply returned | `{ message, reply }` |
| **`theme:changed`** | pack primary color applied | `{ primaryColor: string }` |

**Opt‑in broadcast**: the host only emits a built‑in event if **at least one enabled plugin** for the current role declares that name under **`shell.bridge.events`** or some **`ui_slots[].bridge.events`**. Otherwise no broadcast. Bootstrap returns **`subscribedHostEvents`** (camelCase); Tauri **`is_host_event_subscribed`** can query.

**Plugin `oclive.events` namespace rules**

- **`emit`**: name must match `/^[a-zA-Z0-9.-]+:/`; namespace **before** `:` must equal **this** plugin’s `manifest.id`.  
- **`on` / `off`**: may listen **`otherPlugin:…`** or **`oclive:`** prefixed built‑ins (`oclive:message:sent` → bus `message:sent`).

### 4.3.1 Vue static scan (developer mode)

When bootstrap **`developerMode`** is true, the host AST‑scans `.vue` scripts before compile; dangerous patterns (`fetch`, `eval`, `document.cookie`, `localStorage`, `window.__TAURI__`, …) show a confirm dialog — cancel behaves like compile failure (may fall back to iframe).

### 4.3.2 Force iframe mode

App data **`plugin_state.json`** per role: when **`force_iframe_mode`** is true, the host **ignores** `vueComponent` and uses iframe (`entry`) for stronger isolation. Toggle in settings; **restart** recommended for full effect.

Bootstrap **`uiSlots`** reflects `slot_order` / `disabled_slot_contributions` from `plugin_state.json`. The **plugin manager** (`Ctrl+Shift+F`) reorders per slot and can hide embeds without necessarily stopping the child process unless the plugin is disabled.

---

## 5. Façade commands (B2)

| Tauri command | role |
|---------------|------|
| **`get_directory_plugin_bootstrap`** | `shellUrl`, `shellPluginId`, `pluginIds`, `developerMode`, `subscribedHostEvents`, `uiSlots`, … |
| **`is_host_event_subscribed`** | `event` + optional `role_id` |
| **`directory_plugin_invoke`** | lazy start + one JSON‑RPC `method`/`params` to plugin URL |
| **`plugin_bridge_invoke`** | bridge from iframe / Vue; validates `pluginId` + `assetRel` + allowlist |
| **`read_plugin_asset_text`** | read text under plugin root (no `..`) |

Front‑end `invoke` wraps args under **`req`** (same as other commands):

```json
{
  "req": {
    "pluginId": "com.example.myplugin",
    "method": "my.extension",
    "params": {}
  }
}
```

**Optional env**

| variable | meaning |
|----------|---------|
| `OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS` | non‑LLM directory RPC timeout (default 8000) |
| `OCLIVE_DIRECTORY_LLM_TIMEOUT_MS` | directory LLM timeout (default 120000) |
| `OCLIVE_DIRECTORY_PLUGIN_TOKEN` | optional Bearer |

---

## 6. Developer mode (C1) recap

- **`developer_mode`** or **`OCLIVE_DEVELOPER=1`**: `extra_plugin_roots` scanned.  
- Otherwise ignored — reduces accidental loading from arbitrary paths.

---

## 7. Source index

| area | path |
|------|------|
| scan / manifest / lazy / shell URL | `kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/` |
| enums + `directory_plugins` | `kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs` |
| six‑slot resolve + HTTP reuse | `kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`, `remote_plugin/` |
| Tauri APIs | `api/directory_plugin.rs`, `api/plugin_bridge.rs`, `api/plugin_update.rs`, … |
| custom protocol | `distros/desktop-tauri/src/lib.rs` |
| Vue bootstrap | `distros/shared/src/main.js`, `distros/shared/src/utils/directoryShellBootstrap.ts`, `distros/shared/src/DirectoryShellApp.vue` |
| toolbar / settings / detail slots | `ChatPluginToolbarSlots.vue`, `PluginSettingsPanelSlots.vue`, `PluginRoleDetailSlots.vue`, … |
| TS wrappers | `distros/shared/src/api/` |

---

## 8. Minimal examples

**`examples/directory-plugin-minimal/`** — includes **`Shell.vue`** + **`shell.vueEntry`**.  
**`examples/directory-plugin-llamacpp/`** — LLM slot + local **llama.cpp** HTTP ([README.en.md](../../examples/directory-plugin-llamacpp/README.en.md) · [中文 README](../../examples/directory-plugin-llamacpp/README.md)).  
**`examples/directory-plugin-ui-slot/`** — toolbar iframe.  
**`examples/directory-plugin-ui-slot-vue/`** — Vue toolbar + HTML fallback.

Scaffold:

```bash
npm run scaffold:ui-plugin -- --id com.example.my-slot --slot role.detail --title "My Slot Card"
```

---

## 9. Troubleshooting

| symptom | likely cause |
|---------|----------------|
| Still builtin / Ollama, logs say directory | **`directory_plugins.<slot>`** empty or wrong `id` |
| Shell not switching | bad **`shell_plugin_id`**, missing **`shell.entry`**, empty **`shellUrl`**, or `force_iframe_mode` edge case |
| Vue shell → HTML | bad **`shell.vueEntry`**, compile error, or forced iframe |
| zip update fails | invalid **`manifest.json`** layout, **`id`** mismatch, files locked |
| `invoke` fails in shell page | missing **`dangerousRemoteDomainIpcAccess`** for `https://ocliveplugin.localhost` |
| child never ready | **`process.command`** not on PATH, bad JSON, no **`OCLIVE_READY <url>`** line within timeout |
| `directory_plugin_invoke` errors | unknown `pluginId`, missing **`process`** |

Filter logs: **`oclive_plugin`**.
