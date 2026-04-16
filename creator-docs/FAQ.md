# Oclive 插件与角色包常见问题（FAQ）

面向 **插件开发者** 与 **角色包创作者**。技术细节以 `creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md` 为准。

---

### Q: 为什么我的 Vue 插槽组件没有显示？

**A:** 按顺序排查：

1. **`manifest.json`** 中对应 **`ui_slots`** 条目是否包含正确的 **`vueComponent`**（相对插件根的路径，如 `slots/ToolbarButton.vue`），且 **`entry`** 已填（iframe 回退与 bridge 锚点依赖它）。
2. 设置中是否开启了 **「强制 iframe 模式」**（`plugin_state` 中 `force_iframe_mode`）。开启后宿主会 **忽略** `vueComponent`，只显示 iframe。
3. 打开开发者工具（F12），查看控制台是否有 **`PluginVueCompileError`** 或 **`[Vue SFC]`** 相关报错；若静态扫描在开发者模式下拦截，会弹出 **「插件安全警告」**，取消则不会加载 Vue 组件。
4. 确认 **`get_directory_plugin_bootstrap`** 返回的插槽列表里包含你的插件，且未被禁用/依赖未满足（管理面板中有提示）。

---

### Q: 如何调试插件的 iframe 内容？

**A:**

1. 在聊天页或设置页找到插件 iframe 区域，**右键 → 查看框架源代码**（或等价「在开发者工具中打开 iframe」），即可对该 iframe 文档单独调试。
2. 若需 **Vue 单文件调试**（热重载、组件树），在管理面板中 **关闭「强制 iframe 模式」**，保存后 **重启应用**，使宿主优先加载 `vueComponent`。
3. 确认 **`manifest`** 里 **`bridge.invoke`** 已声明所需命令，否则 `oclive.invoke` 会报权限错误。

---

### Q: 重试按钮点了没反应怎么办？

**A:**

1. 确认插件目录 **`manifest.json`** 与 **入口 HTML/Vue** 未被移动或损坏。
2. 查看 **主终端**（`tauri dev`）或 **浏览器控制台** 是否有 `read_plugin_asset_text`、`plugin_bridge_invoke` 或编译错误日志。
3. 在插件管理面板中 **禁用再启用** 该插件，或提高 **`reloadNonce`**（前端重试会递增）以强制重新加载 iframe/Vue。

---

### Q: 插件依赖缺失怎么办？

**A:**

1. 打开 **插件管理面板**，查看该插件的 **依赖状态**（`ok` / `missing` / `mismatch`）及具体缺失的 **`manifest.id`**。
2. 从社区站或本地 **`plugins/`** 安装对应目录插件，确保 **`manifest.json`** 中 **`version`** 满足依赖声明的 semver 范围。
3. 重启应用或等待目录扫描刷新后再启用。

---

### Q: 如何导出角色包里的 `ui.json`？

**A:**

1. 在 **oclive-pack-editor（编写器）** 的 **「前端设计」**（或等价布局面板）中配置整壳、插槽顺序与主题。
2. **导出角色包** 时，编写器会将 **`ui.json`** 写入包根目录（与 `settings.json` 并列）。若手动编辑，请对照 **[role-pack/ui.json.schema.json](role-pack/ui.json.schema.json)** 校验。

---

### Q: 切换角色后插件配置像「丢失」了？

**A:**

1. **`plugin_state.json`（v2）按 `role_id` 隔离**：不同角色可有不同的整壳、插槽顺序与禁用列表。
2. 切换角色后看到的是 **当前角色** 下的状态，属于预期行为。
3. 若希望多角色一致，需在 **各角色下** 分别在管理面板中调整（或在未来版本使用导出/复制工作流）。

---

### Q: 在哪里管理插件（启用、停用、排序、更新）？

**A:**

1. 在主界面按 **`Ctrl+Shift+F`** 打开 **插件管理** 弹层。  
2. 面板支持：
   - 启用/停用插件（含批量）
   - 按插槽拖拽排序（`chat_toolbar`、`settings.panel`、`role.detail`、`sidebar`、`chat.header`）
   - 从本地 zip 更新插件
3. 调整后点击 **保存**；若涉及停用进程插件，建议重启应用以完全释放。
4. 误操作可点 **重置为角色包推荐**，恢复到该角色 `ui.json` 的默认布局。

---

### Q: mumu 现在默认有哪些前端模块？

**A:** 当前 `roles/mumu/ui.json` 默认启用了 4 个前端模块（目录插件）：

- `chat.header`：`com.oclive.mumu.chat-header-status`（头部状态）
- `chat_toolbar`：`com.oclive.mumu.quick-actions`（快捷动作）
- `sidebar`：`com.oclive.mumu.sidebar-glance`（侧栏概览）
- `settings.panel`：`com.oclive.mumu.settings-panel`（设置面板）

若你看不到这些模块，通常是本地 `plugin_state.json` 覆盖了角色包默认布局；在插件管理中执行 **重置为角色包推荐** 即可恢复。

---

### Q: 开发者模式的热重载不生效？

**A:**

1. 确认 **`oclive_host_plugins.json`** 中 **`developer_mode`: true**，或环境变量 **`OCLIVE_DEVELOPER=1`**。
2. 确认 **额外插件根** 已配置且目录存在（仅开发者模式扫描 **`extra_plugin_roots`**）。
3. **Linux**：若文件监听失败，检查终端是否有 **`notify`** 相关错误；必要时安装 **`inotify`** 工具链或 `libnotify`（依发行版而定）。
4. 修改 **`manifest.json`** 后，部分场景需 **重新扫描**；可重启应用或触发管理面板的刷新。

---

### Q: 整壳插件调用 `invoke` 提示没有权限？

**A:**

1. 在 **`shell.bridge.invoke`** 中声明 **命令名** 或 **权限别名**（见 [BRIDGE_API_REFERENCE.md](plugin-and-architecture/BRIDGE_API_REFERENCE.md)）。
2. **敏感命令**（如 `send_message`）还要求 **`"type": "ocliveplugin"`**，且调用页面对应为 **`shell.entry` 或 `shell.vueEntry`**（不能从 `ui_slots` 页调用这些命令）。

---

## 更多文档

- [plugin-and-architecture/DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [guides/CONFIGURATION_FILES.md](guides/CONFIGURATION_FILES.md)
- [getting-started/ERROR_CODES.md](getting-started/ERROR_CODES.md)
