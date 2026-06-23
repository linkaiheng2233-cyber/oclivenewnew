# Oclive plugins & role packs — FAQ

For **plugin authors** and **role pack creators**. Authoritative technical detail lives in [`plugin-and-architecture/DIRECTORY_PLUGINS.md`](plugin-and-architecture/DIRECTORY_PLUGINS.md).

[中文](../creator-docs/FAQ.md)

---

### Q: Why does my Vue slot component not show?

**A:** Check in order:

1. **`manifest.json`** — the matching **`ui_slots`** entry has a correct **`vueComponent`** (path relative to the plugin root, e.g. `slots/ToolbarButton.vue`), and **`entry`** is set (iframe fallback and bridge anchoring depend on it).
2. Settings — **“Force iframe mode”** (`force_iframe_mode` in `plugin_state`). When on, the host **ignores** `vueComponent` and only shows iframes.
3. DevTools (F12) — look for **`PluginVueCompileError`** or **`[Vue SFC]`** errors; in developer mode a static scan may show a **plugin security** dialog — cancelling skips loading the Vue component.
4. Confirm **`get_directory_plugin_bootstrap`** lists your plugin for the slot, and it is not disabled / dependency-blocked (the manager shows hints).

---

### Q: How do I debug a plugin iframe?

**A:**

1. On chat or settings, **right‑click the iframe → View frame source** (or open that frame in DevTools) to debug that document alone.
2. For **Vue SFC** debugging (HMR, component tree), turn **off “Force iframe mode”** in the manager, save, **restart the app** so the host prefers `vueComponent`.
3. Ensure **`manifest`** declares required commands under **`bridge.invoke`**, or `oclive.invoke` will report permission errors.

---

### Q: The retry button does nothing?

**A:**

1. Confirm **`manifest.json`** and entry **HTML/Vue** under the plugin directory were not moved or corrupted.
2. Check the **main terminal** (`tauri dev`) or **browser console** for `read_plugin_asset_text`, `plugin_bridge_invoke`, or compile errors.
3. In the plugin manager, **disable then enable** the plugin, or bump **`reloadNonce`** (frontend retry increments it) to force iframe/Vue reload.

---

### Q: Plugin dependencies are missing?

**A:**

1. Open the **plugin manager** and read **dependency status** (`ok` / `missing` / `mismatch`) and the missing **`manifest.id`**.
2. Install the directory plugin from the community site or local **`distros/chat-pro/plugins/`**, with **`manifest.json` `version`** satisfying the declared semver range.
3. Restart the app or wait for directory rescan before enabling.

---

### Q: How do I export `ui.json` from a role pack?

**A:**

1. In **oclive-pack-editor**, use **“Front-end design”** (or the equivalent layout panel) to configure shell, slot order, and theme.
2. When **exporting the pack**, the editor writes **`ui.json`** next to `settings.json`. If you edit by hand, validate against **[role-pack/ui.json.schema.json](../creator-docs/role-pack/ui.json.schema.json)**.

---

### Q: After switching roles, plugin config looks “lost”?

**A:**

1. **`plugin_state.json` (v2) is keyed by `role_id`**: each role can have its own shell, slot order, and disable list.
2. After a switch you see **the current role’s** state — expected.
3. For identical layout across roles, adjust each role in the manager (or use a future export/copy workflow).

---

### Q: Where do I manage plugins (enable, disable, order, update)?

**A:**

1. Press **`Ctrl+Shift+F`** in the main window to open the **plugin manager**.
2. The panel supports:
   - Enable/disable (including batch)
   - Drag‑to‑reorder per slot (`chat_toolbar`, `settings.panel`, `role.detail`, `sidebar`, `chat.header`)
   - Update from a local zip
3. Click **Save** after changes; if you stop a process plugin, restarting the app fully releases resources.
4. **Reset to pack‑recommended layout** restores defaults from that role’s `ui.json`.

---

### Q: Which default front-end modules does mumu ship?

**A:** `distros/chat-pro/roles/mumu/ui.json` enables five directory plugins by default:

- `chat.header`: `com.oclive.mumu.chat-header-status`
- `chat_toolbar`: `com.oclive.mumu.quick-actions`
- `role.detail`: `com.oclive.mumu.role-detail-card`
- `sidebar`: `com.oclive.mumu.sidebar-glance`
- `settings.panel`: `com.oclive.mumu.settings-panel`

If you do not see them, local `plugin_state.json` often overrides the pack default — use **Reset to pack‑recommended layout** in the manager.

The `sidebar` module adds **“Suggest next line”**: it only fills the input box; it does **not** auto‑send.

---

### Q: Developer hot reload does not work?

**A:**

1. Ensure **`oclive_host_plugins.json`** has **`developer_mode`: true**, or **`OCLIVE_DEVELOPER=1`**.
2. Ensure **extra plugin roots** exist on disk (only scanned in developer mode).
3. **Linux**: if file watching fails, check for **`notify`** / **`inotify`** issues per distro.
4. After **`manifest.json`** changes, some flows need a **rescan** — restart the app or refresh from the manager.

---

### Q: Whole‑shell `invoke` says no permission?

**A:**

1. Declare command names or **permission aliases** under **`shell.bridge.invoke`** (see [BRIDGE_API_REFERENCE.md](plugin-and-architecture/BRIDGE_API_REFERENCE.md)).
2. **Sensitive commands** (e.g. `send_message`) also require **`"type": "ocliveplugin"`** at the manifest root, and the caller must be **`shell.entry`** or **`shell.vueEntry`** — not a `ui_slots` page.

---

## More reading

- [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [guides/CONFIGURATION_FILES.md](../creator-docs/guides/CONFIGURATION_FILES.md) (Chinese; English stub TBD)
- [getting-started/ERROR_CODES.md](getting-started/ERROR_CODES.md)
