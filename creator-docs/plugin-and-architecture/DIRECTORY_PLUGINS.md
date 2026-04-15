# 目录式进程插件（Directory Plugins）— 架构与契约

本文描述用户选型 **A1–C1** 下**当前实现**：`plugins/` 扫描、`manifest.json`、子进程 JSON-RPC、**整壳 UI**（`https://ocliveplugin.localhost/…`）、**统一门面命令** `directory_plugin_invoke`（等价于「动态 Tauri 命令」），以及**开发者模式**从额外根目录加载。

**Wire 格式**：与现有 Remote 侧车一致（HTTP POST JSON-RPC 2.0、请求头 `x-oclive-remote-protocol` 等），见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。

**与 `plugin_backends` 的关系**：各模块枚举值可为 **`directory`**；同包（或会话覆盖）内嵌对象 **`directory_plugins`** 为各槽位指定 **`manifest.id`**（见下节）。就绪行：子进程 stdout 打印 **`{ready_prefix} {rpc_url}`**（默认前缀 `OCLIVE_READY`，与一行 URL，空格分隔）。

---

## 1. 目录布局与扫描顺序

宿主合并以下**存在的**扫描根，每个根下的一级子目录若含 `manifest.json` 则视为一个插件包（以 manifest 内 `id` 注册；重复 `id` 时后扫描到的根覆盖并打日志）：

1. **`<roles 父目录>/plugins/`**（与 `roles/` 同级；开发时常为仓库根下 `plugins/`）
2. **`./plugins/`**（相对进程当前工作目录）
3. **`{app_data}/plugins/`**（与 `app.db` 同级的应用数据目录下的 `plugins/`）

**开发者模式（C1）**：当 `app_data/oclive_host_plugins.json` 中 **`developer_mode`: true**，或环境变量 **`OCLIVE_DEVELOPER=1`**（`true`/`yes` 亦可）时，额外扫描 **`extra_plugin_roots`** 中每一项（须为已存在目录）；行为同上。

### `oclive_host_plugins.json`（可选，位于应用数据目录根）

| 字段 | 类型 | 说明 |
|------|------|------|
| `developer_mode` | `boolean?` | 为真时启用 `extra_plugin_roots` |
| `extra_plugin_roots` | `string[]?` | 额外插件容器目录（其下一级子目录为插件根） |
| `shell_plugin_id` | `string?` | 指定用于整壳替换的插件 `manifest.id` |

环境变量 **`OCLIVE_SHELL_PLUGIN_ID`**（非空 trim）优先于文件中的 `shell_plugin_id`。

---

## 2. `manifest.json`（插件根目录）

| 字段 | 类型 | 说明 |
|------|------|------|
| `schema_version` | `number` | 当前仅接受 **`1`** |
| `id` | `string` | 全局唯一；与 `directory_plugins.*` 槽位对应 |
| `version` | `string` | 建议 SemVer 文本 |
| `shell` | `object?` | **`entry`**：相对插件根的 HTML 入口（整壳 B1，回退用）；**`vueEntry?`**：相对插件根的 `.vue`，在 **`force_iframe_mode` 关闭** 且文件可读时由宿主用 Vue 渲染整壳（与插槽 `vueComponent` 体验一致；失败则回退 `entry`） |
| `process` | `object?` | **`command`** / **`args[]`** / **`cwd?`**（`cwd` 相对插件根，可省略则默认为插件根） |
| `ready_prefix` | `string?` | 默认 **`OCLIVE_READY`**；就绪行 = 此前缀 + 空格 + **JSON-RPC 根 URL**（须 `http://` 或 `https://`） |

**懒启动**：首次需要该插件的 RPC（五模块 `directory`、`directory_plugin_invoke`、或需解析 shell manifest）时启动子进程，并缓存 **RPC URL** 与 **子进程**（当前实现不随角色切换回收子进程；应用退出时释放）。并发多次触发同一 `id` 时，宿主对单次启动加锁，避免重复子进程。

---

## 3. 后端五模块（A2）

在 `settings.json`（或等价磁盘设置）的 `plugin_backends` 中：

- `memory` / `emotion` / `event` / `prompt` 为 **`directory`** 时，使用 **`directory_plugins.<slot>`** 中的插件 `id` 懒启动后，对该 URL 走与 env-remote 相同的 HTTP 客户端（方法名分别为 `memory.rank` 等）。
- `llm` 为 **`directory`** 时，使用 **`directory_plugins.llm`** 指向的插件 URL，须实现 **`llm.generate` / `llm.generate_tag`**（超时默认按 LLM 档读取，见环境变量）。

若对应槽位 **id 缺失**、**运行时未注入目录插件**、**spawn 或握手失败**，宿主记日志并回退：**memory/emotion/event/prompt → builtin**，**llm → Ollama**。

### `plugin_backends` 与 `directory_plugins` 示例（节选）

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

**`directory_plugins` 槽位来源**：以角色包 **`settings.json` → `plugin_backends.directory_plugins`** 为准。`PluginBackendsOverride` 在 Rust 中**支持**按槽合并 `directory_plugins`（见 `apply_to`），但当前 Tauri 命令 **`set_session_plugin_backend` 仅覆盖五模块枚举与 `local_memory_provider_id`**，**不**传入 `directory_plugins`；多会话场景下若需不同目录插件 id，请通过角色包或后续扩展的会话 API 提供。

---

## 4. 整壳 UI（B1）

当 **`shell_plugin_id`**（文件或 `OCLIVE_SHELL_PLUGIN_ID`）指向已扫描到的插件，且其 manifest 含 **`shell.entry`** 时，内置前端在挂载主应用**之前**调用 **`get_directory_plugin_bootstrap`**（可省略 `role_id`，与旧行为一致）。

- **`force_iframe_mode`**（bootstrap 与 `plugin_state` 一致）：为真时**忽略** **`shell.vueEntry`**，若存在 **`shellUrl`** 且当前文档 URL 与之不同，则 **`location.replace(shellUrl)`**（HTML 整壳）。
- 否则若 manifest 含非空的 **`shell.vueEntry`**，且 **`read_plugin_asset_text`** 能读到该 `.vue`：宿主在 **`#app`** 挂载轻量 Vue 根（**`DirectoryShellApp.vue`** + **`AsyncPluginVue`**），`inject('oclive')` 与插槽一致；**`plugin_bridge_invoke` 的 `assetRel` 应传 `vueEntry` 路径**（与敏感命令门禁「整壳页」判定一致）。
- 若未走 Vue（无 `vueEntry`、读文件失败、或用户强制 iframe）：若 **`shellUrl`** 与当前页不同，则 **`location.replace(shellUrl)`**。

**`shellUrl` 形态**：`https://ocliveplugin.localhost/<manifest.id>/<entry>`（Windows WebView2 下由 Tauri 将自定义协议映射为该 HTTPS 主机名）。

**静态资源**：由宿主 **`register_uri_scheme_protocol("ocliveplugin", …)`** 从磁盘插件根读取（路径穿越会 403）。需在 **`tauri.conf.json`** 中配置 **`dangerousRemoteDomainIpcAccess`**，使该来源页面可调用 Tauri IPC（与内置 `invoke` 一致）。条目内需 **`enableTauriAPI`: true**（字段名随 Tauri 版本以 schema 为准；勿使用无效的 `enable`）。

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

### 4.1 整壳前端桥接（`shell.bridge`）

若 **`shell`** 下声明 **`bridge`**，且 **`invoke`** / **`events`** 非空：宿主在提供 **`shell.entry` 对应 HTML** 时会在 `</body>` 前注入脚本，挂载 **`window.OclivePluginBridge`**；若走 **`shell.vueEntry`** Vue 整壳，则由 **`provide('oclive', …)`** 注入同一套 **`invoke` / `events`**（底层仍走 **`plugin_bridge_invoke`**）。

- **`invoke(command, params)`**：manifest 的 **`bridge.invoke`** 为**权限列表**：可写 **命令名**（如 `send_message`）或 **权限别名**（如 `read:conversation`）；与下表对应。由 **`plugin_bridge_invoke`** 二次校验。
- **`listen(event, handler)`**：仅允许 **`bridge.events`** 中的事件名（依赖 WebView 内 `__TAURI__.event`）。

**整壳深度集成**：下列命令除需 **`bridge.invoke`** 命中外，还要求 manifest 顶层 **`"type": "ocliveplugin"`**，且调用来源为 **`shell.entry` 对应 HTML** 或 **`shell.vueEntry` 宿主 Vue 页**（**`ui_slots` 页不得调用**，避免越权）：

| `OclivePluginBridge.invoke` 命令 | manifest 权限（`invoke` 数组中任写其一即可） | 说明 |
|-----------------------------------|---------------------------------------------|------|
| **`send_message`** | `send_message` | 走 `process_message`，参数同 `send_message`（可用 `text` 代替 `user_message`） |
| **`get_conversation`** | `get_conversation` 或 **`read:conversation`** | 读短期对话；`params`: `role_id`, 可选 `session_id` / `limit` / `offset` |
| **`switch_role`** | `switch_role` | `params`: `{ "role_id": "..." }`，等价于宿主 `switch_role` |
| **`get_roles`** | `get_roles` 或 **`read:roles`** | 等价于 `list_roles` |
| **`get_current_role`** | `get_current_role` 或 **`read:current_role`** | 等价于 `get_role_info`，`params` 同 `get_role_info`（`role_id` + 可选 `session_id`） |
| **`update_memory`** | **`write:memory`** 或 `update_memory` | 写入长期记忆；`params`: `role_id`, `content`, 可选 `importance`（0–1，默认 0.5） |
| **`delete_memory`** | **`write:memory`** 或 `delete_memory` | 删除长期记忆；`params`: `role_id`, `memory_id`（须属于该角色） |
| **`update_emotion`** | **`write:emotion`** 或 `update_emotion` | 更新 `role_runtime.current_emotion`；`params`: `role_id`, `emotion` |
| **`update_event`** | **`write:event`** 或 `update_event` | 与 `create_event` 等价；`params`: `role_id`, `event_type`, 可选 `description`（事件类型枚举同 `CreateEventRequest`） |
| **`update_prompt`** | **`write:prompt`** 或 `update_prompt` | 预留：当前宿主返回 `not_implemented`，待动态提示词片段契约落地 |

**不强制 `type: ocliveplugin` 的桥接命令**（亦需在 **`bridge.invoke`** 中声明）：`get_role_info`、`list_roles`、`get_time_state`、`get_directory_plugin_bootstrap` 等。未声明的调用一律拒绝。

**写入类命令**（`update_memory` / `delete_memory` / `update_emotion` / `update_event` / `update_prompt`）与上表「聊天/角色」敏感命令相同：**必须** `type: ocliveplugin` 且自 **`shell.entry` HTML** 或 **`shell.vueEntry` Vue** 调用。

### 4.2 主界面 UI 插槽（`ui_slots`）

官方支持的 **`slot`** 值：

| `slot` | 宿主位置 | 说明 |
|--------|----------|------|
| **`chat_toolbar`** | 聊天输入区上方 | 窄条工具栏，适合快捷操作 |
| **`settings.panel`** | **设置 → 插件扩展**（顶栏「更多」→「打开设置」） | 较大区域，适合插件配置表单；可用 **选项卡** 在多个 `settings.panel` 插件间切换 |
| **`role.detail`** | 左侧 **角色详情**（立绘与名称下方，好感度条上方） | 垂直 iframe 列表，适合与当前角色相关的扩展信息或快捷编辑 |

通用规则：

- 若 manifest **无** **`shell`** 段，可在 **`ui_slots`** 中声明嵌入 UI：**`entry`** 为相对插件根的 HTML（**iframe 回退**）。
- 可选 **`vueComponent`**：相对插件根的 **`.vue`** 路径（如 `"slots/ToolbarButton.vue"`）。主界面优先在宿主 Vue 内用运行时编译器加载该组件；**加载失败时自动回退**到上述 iframe（`https://ocliveplugin.localhost/<id>/<entry>`）。
- **含 `shell` 的插件不参与插槽**（避免与整壳重复）。
- 插槽页若需调用宿主能力：在对应 **`ui_slots[]` 条目**上配置 **`bridge`**。iframe 页仅当请求资源与 **`entry`** 一致时注入 `OclivePluginBridge`；**原生 Vue 插槽**通过 `inject('oclive')` 获得 API（见下），`plugin_bridge_invoke` 校验时使用 manifest 中的 **`entry`** 作为 **`assetRel`**（与 `bridge` 白名单一致）。
- 示例：`examples/directory-plugin-ui-slot/`（仅 iframe）；**`examples/directory-plugin-ui-slot-vue/`**（`vueComponent` + 回退 HTML）。

### 4.2.1 原生 Vue 插槽（`vueComponent`）

| 字段 | 说明 |
|------|------|
| **`entry`** | 必填；iframe URL 与 bridge 权限锚点（`assetRel` 使用本字段的规范化相对路径）。 |
| **`vueComponent`** | 可选；插件根下 `.vue` 文件相对路径。组件需 `export default` 符合 Vue 3 组件；模板内使用 **`const oclive = inject('oclive')`**。 |

**`oclive` 对象（与整壳桥接能力对齐，经同一 `plugin_bridge_invoke` 后端）：**

- **`oclive.invoke(command, params?)`**：等价于 iframe 内 `OclivePluginBridge.invoke`。
- **`oclive.pluginId`** / **`oclive.bridgeAssetRel`**：当前插件 id 与桥接用 `entry` 路径。
- **`oclive.events.emit` / `on` / `off`**：宿主 **mitt** 事件总线（见 4.3）；`on` 注册的监听在**组件卸载时自动移除**。

样式可直接使用宿主 **CSS 变量**（如 `--fluent-accent`、`--bg-primary`、`--font-ui`、`--border-light` 等，见 `src/styles/theme.css`）。

**安全说明**：插件组件与主界面同 JS 上下文；请勿在插件中直接使用 `window.__TAURI__`，应只通过 `oclive.invoke` 访问白名单命令。宿主不对插件做完整沙箱隔离。

### 4.3 事件总线（宿主内置）

| 事件名 | 触发时机 | `data`（建议可 JSON 序列化） |
|--------|----------|-------------------------------|
| **`role:switched`** | 用户切换当前角色成功 | `{ roleId: string }` |
| **`message:sent`** | 用户发送消息且本轮回复已返回 | `{ message: string, reply: string }` |
| **`theme:changed`** | 角色包 `ui.json` 主题主色应用到界面 | `{ primaryColor: string }` |

**按需广播（隐私与性能）**：宿主仅在**当前角色下已启用**的插件中，至少有一个在 manifest 的 **`shell.bridge.events`** 或某一 **`ui_slots[].bridge.events`** 中声明了该事件名时，才会向 Vue 插槽内的 `oclive.events` / mitt 总线广播对应内置事件。未声明则**不广播**（等同未监听）。`get_directory_plugin_bootstrap` 返回的 **`subscribedHostEvents`**（camelCase）为当前应广播的内置事件名列表（去重排序）；Tauri 命令 **`is_host_event_subscribed`** 也可按事件名查询。

自定义事件建议使用 **`插件ID:事件名`**（如 `com.example.foo:refresh`），避免冲突。iframe 内页面如需订阅，需在后续版本扩展 `OclivePluginBridge`（当前以 Vue 插槽为主）。

### 4.3.1 原生 Vue 安全扫描（开发者模式）

当 **`get_directory_plugin_bootstrap.developerMode`** 为真时，宿主在编译 `.vue` 前会对脚本做静态 AST 扫描；若匹配危险模式（如 `fetch`、`eval`、`document.cookie`、`localStorage`、`window.__TAURI__` 等），会弹出确认框，用户取消则该插槽回退行为与编译失败一致（可再走 iframe）。

### 4.3.2 强制 iframe 模式

应用数据 **`plugin_state.json`** 按角色存储的 **`force_iframe_mode`**（默认 `false`）为真时，宿主**忽略** manifest 的 **`vueComponent`**，嵌入插槽一律使用 iframe（`entry` URL），以获得更强隔离。设置页可切换该项；保存后建议**重启应用**以完全生效。

**`get_directory_plugin_bootstrap` → `uiSlots`** 会返回上述插槽的条目（已按 `app_data/plugin_state.json` 中的 **`slot_order`** 分插槽排序）。示例：

```json
{
  "disabled_plugins": [],
  "slot_order": {
    "chat_toolbar": ["com.example.toolbar_a", "com.example.toolbar_b"],
    "settings.panel": ["com.example.settings_a", "com.example.settings_b"],
    "role.detail": ["com.example.role_extra"]
  },
  "disabled_slot_contributions": {
    "chat_toolbar": [],
    "settings.panel": [],
    "role.detail": []
  }
}
```

在 **插件管理**（Ctrl+Shift+F）中可为每个插槽单独拖拽排序，或勾选「隐藏 … 嵌入」仅关闭该插槽 iframe（不卸载插件进程，除非同时停用插件）。

---

## 5. 门面命令（B2）

运行时无法向 `generate_handler!` 动态注册符号；采用**固定命令名** + 插件侧方法名：

| Tauri 命令 | 作用 |
|------------|------|
| **`get_directory_plugin_bootstrap`** | 返回 `shellUrl`、`shellPluginId`、`pluginIds`、`developerMode`、`subscribedHostEvents`、`uiSlots`（嵌入插槽列表，camelCase JSON） |
| **`is_host_event_subscribed`** | `event` + 可选 `role_id`：当前角色下是否有已启用插件在 manifest `bridge.events` 中声明该事件名 |
| **`directory_plugin_invoke`** | 懒启动目标插件后，向其 RPC URL 发送一次 JSON-RPC **`method`** / **`params`** |
| **`plugin_bridge_invoke`** | 目录插件页经 **`OclivePluginBridge.invoke`** 或宿主 Vue 插槽 **`oclive.invoke`** 调用；校验 **`pluginId` + `assetRel`** 与 manifest **`bridge.invoke` 白名单** 后转发到受控宿主逻辑 |
| **`read_plugin_asset_text`** | 宿主读取插件根下文本文件（用于编译 `.vue`）；路径不得含 `..` 或越出插件目录 |

**前端 `invoke` 载荷**（与仓库其它命令一致，单结构体参数包在 **`req`** 下）：

```json
{
  "req": {
    "pluginId": "com.example.myplugin",
    "method": "my.extension",
    "params": {}
  }
}
```

**环境变量（可选）**

| 变量 | 说明 |
|------|------|
| `OCLIVE_DIRECTORY_PLUGIN_TIMEOUT_MS` | 非 LLM 类目录 RPC 调用超时（毫秒），默认 `8000` |
| `OCLIVE_DIRECTORY_LLM_TIMEOUT_MS` | `RemoteLlmHttp` 使用目录 URL 时的超时，默认 `120000` |
| `OCLIVE_DIRECTORY_PLUGIN_TOKEN` | 可选 Bearer，写入 `Authorization` |

---

## 6. 开发者模式（C1）小结

- **`developer_mode`** 或 **`OCLIVE_DEVELOPER=1`**：`extra_plugin_roots` 参与扫描。
- 未开启时忽略 `extra_plugin_roots`，降低误加载任意路径的风险。

---

## 7. 源码索引（实现）

| 区域 | 路径 |
|------|------|
| 扫描 / manifest / 懒启动 / shell URL | `src-tauri/src/infrastructure/directory_plugins/` |
| 枚举与 `directory_plugins` 槽位 | `src-tauri/src/models/plugin_backends.rs` |
| 五模块解析与 HTTP 复用 | `src-tauri/src/domain/plugin_host.rs`、`src-tauri/src/infrastructure/remote_plugin/` |
| Tauri 命令 | `src-tauri/src/api/directory_plugin.rs`、`src-tauri/src/api/plugin_bridge.rs`、`src-tauri/src/api/plugin_update.rs`（本地 zip 覆盖 / 更新检查预留） |
| 自定义协议 + 启动 | `src-tauri/src/lib.rs` |
| 内置 UI 启动引导 | `src/main.js`、`src/utils/directoryShellBootstrap.ts`、`src/DirectoryShellApp.vue` |
| 聊天工具栏插槽 | `src/components/ChatPluginToolbarSlots.vue` |
| 设置页插槽 | `src/components/PluginSettingsPanelSlots.vue`、`src/views/SettingsView.vue` |
| 角色详情插槽 | `src/components/PluginRoleDetailSlots.vue`、`src/views/RoleDetailView.vue` |
| 前端封装 | `src/utils/tauri-api.ts`（`getDirectoryPluginBootstrap`、`directoryPluginInvoke`、`pluginBridgeInvoke`） |

---

## 8. 仓库内最小示例

见 **`examples/directory-plugin-minimal/`**（含 **`Shell.vue`** + **`shell.vueEntry`** 示例）：可复制到 `plugins/<id>/` 或加入 `extra_plugin_roots` 后，配置 `shell_plugin_id` 与（可选）`plugin_backends` 做联调。  
**非整壳 + 工具栏插槽**：**`examples/directory-plugin-ui-slot/`**；**原生 Vue 工具栏 + iframe 回退**：**`examples/directory-plugin-ui-slot-vue/`**。

---

## 9. 排错（常见问题）

| 现象 | 可能原因 |
|------|----------|
| 仍走 builtin / Ollama，日志提示 directory 缺失槽位 | `plugin_backends.* = directory` 但 **`directory_plugins.<槽>`** 未填或与 manifest **`id`** 不一致 |
| 整壳未跳转 / 仍显示内置 UI | **`shell_plugin_id`** 未设或插件未扫描到；manifest 缺 **`shell.entry`**；`get_directory_plugin_bootstrap` 返回的 **`shellUrl`** 为空；或 **`force_iframe_mode`** 为真且未发生 `location.replace` |
| Vue 整壳回退到 HTML | **`shell.vueEntry`** 路径错误或文件不存在；Vue 编译失败（`AsyncPluginVue` 触发失败回退）；**`force_iframe_mode`** 开启 |
| 插件管理「从本地 zip 更新」失败 | zip 内无有效 **`manifest.json`**（根或单一顶层目录）；**`manifest.id`** 与所选插件 id 不一致；目标目录无法删除（占用中） |
| 整壳页里 **`invoke` 失败** | **`tauri.conf.json`** 未为 **`https://ocliveplugin.localhost`** 配置 **`dangerousRemoteDomainIpcAccess`**；或页面未在 Tauri WebView 内打开 |
| 子进程启动失败 / 无 RPC | **`process.command`** 在 PATH 中不可用（如未装 Node）；**`manifest.json`** 语法错误；子进程未在超时内向 stdout 打印 **`OCLIVE_READY <url>`** |
| **`directory_plugin_invoke`** 报错 | **`pluginId`** 未扫描到；目标插件缺 **`process`** 节无法懒启动 RPC |

过滤宿主日志 target：**`oclive_plugin`**。
