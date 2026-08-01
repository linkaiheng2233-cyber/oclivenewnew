# 目录式进程插件（Directory Plugins）— 架构与契约

本文描述用户选型 **A1–C1** 下**当前实现**：`distros/chat-pro/plugins/` 扫描、`manifest.json`、子进程 JSON-RPC、**整壳 UI**（平台映射的 `ocliveplugin` 自定义协议）、**统一门面命令** `directory_plugin_invoke`（等价于「动态 Tauri 命令」），以及**开发者模式**从额外根目录加载。

**Wire 格式**：与现有 Remote 侧车一致（HTTP POST JSON-RPC 2.0、请求头 `x-oclive-remote-protocol` 等），见 [REMOTE_PLUGIN_PROTOCOL.md](REMOTE_PLUGIN_PROTOCOL.md)。

**与 `plugin_backends` 的关系**：各模块枚举值可为 **`directory`**；同包（或会话覆盖）内嵌对象 **`directory_plugins`** 为各槽位指定 **`manifest.id`**（见下节）。就绪行：子进程 stdout 打印 **`{ready_prefix} {rpc_url}`**（默认前缀 `OCLIVE_READY`，与一行 URL，空格分隔）。

**`manifest.id` 安全约束**：1–128 字节；首尾必须为 ASCII 字母或数字；中间仅允许 ASCII 字母、数字、`.`、`_`、`-`，且禁止 `..`。该 id 会同时进入 URL、授权表与插件目录名，宿主在扫描/安装前统一校验，路径分隔符与穿越片段会直接拒绝。

---

## 1. 目录布局与扫描顺序

宿主合并以下**存在的**扫描根，每个根下的一级子目录若含 `manifest.json` 则视为一个插件包（以 manifest 内 `id` 注册；重复 `id` 时后扫描到的根覆盖并打日志）：

1. **`<roles 父目录>/distros/chat-pro/plugins/`**（与 `distros/chat-pro/roles/` 同级；开发时常为仓库根下 `distros/chat-pro/plugins/`）
2. **`./distros/chat-pro/plugins/`**（相对进程当前工作目录）
3. **`{app_data}/distros/chat-pro/plugins/`**（与 `app.db` 同级的应用数据目录下的 `distros/chat-pro/plugins/`）

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
| `shell` | `object?` | **`entry`**：相对插件根的 HTML 入口（发行版整壳必走）；**`vueEntry?`**：仅供本地调试的 `.vue` 入口；必须同时为 Vite DEV、`VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1` 且 `force_iframe_mode` 关闭才会同进程挂载 |
| `process` | `object?` | **`command`** / **`args[]`** / **`cwd?`**（`cwd` 相对插件根，可省略则默认为插件根） |
| `ready_prefix` | `string?` | 默认 **`OCLIVE_READY`**；就绪行 = 此前缀 + 空格 + **JSON-RPC 根 URL**（须 `http://` 或 `https://`） |
| `dependencies` | `object?` | 可选：其它目录插件 **`id` → semver 范围**（如 `">=1.0.0"`、`"^2.0.0"`）；缺失或版本不符时该插件在管理面板标记为依赖不满足且不可启用 |
| `provides` | `string[]?` | 可选：声明提供的后端能力（如 `llm`、`memory`）；供简单管理列表展示与校验 |
| `slot_attachment` | `object \| object[]?` | 可选：安装时自动写入角色包 **`slot_registry`**（见 [PLUGIN_V1.md](PLUGIN_V1.md)）；需配合 **`oclive plugin install --role`** |
| `description` / `author` | `string?` | 可选：简单管理列表展开详情 |

**懒启动**：首次需要该插件的 RPC（`plugin_backends` 六模块中 **`directory`**、`directory_plugin_invoke`、或需解析 shell manifest）时启动子进程，并缓存 **RPC URL** 与 **子进程**（当前实现不随角色切换回收子进程；应用退出时释放）。并发多次触发同一 `id` 时，宿主对单次启动加锁，避免重复子进程。

### 高风险：`process` 与子进程 spawn

可选 **`permissions`** 数组声明高危能力（见 [PLUGIN_V1.md §权限规范](PLUGIN_V1.md)）。若 manifest 声明了 **`process`**（或显式 **`process:spawn`**），宿主在首次 `spawn` 前检查 **`high_risk_grants.json`** 是否已为该插件 **`id`** 授予 **`process:spawn`**。省略 `permissions` 的旧包仍按 **`process` 块存在** 触发同一授权路径。未授权时：`directory_plugin_invoke` 等经 `map_directory_rpc_url_error` 映射为 **`HIGH_RISK_CAPABILITY_NOT_GRANTED`**；`plugin_backends` 主路径在无法取得 RPC URL 时记日志并回退内置 / Ollama（见上文 §3）。

用户可在 **设置 → 插件与后端 → Agent 调试** 中查看/授予/撤销（调用 `list_high_risk_grants`、`grant_high_risk_capability`、`revoke_high_risk_capability`）。自动化或 CI 可设 **`OCLIVE_SKIP_HIGH_RISK_GRANTS=1`** 跳过检查（勿用于面向用户的生产场景）。

**MCP**：`{app_data}/mcp-servers/*.json` 的 **`http`** / **`stdio`** 传输分别需要 **`mcp:http`** / **`mcp:stdio`** 授权项。**Remote 侧车**（`OCLIVE_REMOTE_*`）出站前需要 **`network:*`**（grant id **`remote:plugin`** / **`remote:llm`**）。详见 [`handoff/A4_CLOSURE_SUMMARY.md`](../../handoff/A4_CLOSURE_SUMMARY.md)。

---

## 3. 后端六模块（A2）

在 `settings.json`（或等价磁盘设置）的 `plugin_backends` 中：

- `memory` / `emotion` / `event` / `prompt` 为 **`directory`** 时，使用 **`directory_plugins.<slot>`** 中的插件 `id` 懒启动后，对该 URL 走与 env-remote 相同的 HTTP 客户端（方法名分别为 `memory.rank` 等）。
- `llm` 为 **`directory`** 时，使用 **`directory_plugins.llm`** 指向的插件 URL，须实现 **`llm.generate` / `llm.generate_tag`**（超时默认按 LLM 档读取，见环境变量）。
- `agent` 为 **`directory`** 时，使用 **`directory_plugins.agent`** 指向的插件 URL，须实现 **`agent.process`**（host-orchestrated MCP，见 [AGENT_REMOTE_PROTOCOL.md](AGENT_REMOTE_PROTOCOL.md)）。

若对应槽位 **id 缺失**、**运行时未注入目录插件**、**spawn 或握手失败**，宿主记日志并回退：**memory/emotion/event/prompt → builtin**，**llm → Ollama**，**agent → builtin**。

**示例（LLM 槽 → 本机 llama.cpp HTTP，不经 Ollama）**：仓库 [`examples/directory-plugin-llamacpp/`](../../examples/directory-plugin-llamacpp/README.md)（[English](../../examples/directory-plugin-llamacpp/README.en.md)）— Node 侧车实现 `llm.generate` / `llm.generate_tag`，将请求转发到 `OCLIVE_LLAMACPP_SERVER_URL`（默认 `http://127.0.0.1:8080`）上的 `llama-server`；角色包内将 `plugin_backends.llm` 设为 **`directory`** 并填写 `directory_plugins.llm` 为该 manifest **`id`** 即可与其它仍用 Ollama 的角色并存。

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

**`directory_plugins` 槽位来源**：以角色包 **`settings.json` → `plugin_backends.directory_plugins`** 为准。`PluginBackendsOverride` 在 Rust 中**支持**按槽合并 `directory_plugins`（见 `apply_to`），但当前 Tauri 命令 **`set_session_plugin_backend` 仅覆盖六模块枚举与 `local_memory_provider_id`**，**不**传入 `directory_plugins`；多会话场景下若需不同目录插件 id，请通过角色包或后续扩展的会话 API 提供。

---

## 4. 整壳 UI（B1）

当 **`shell_plugin_id`**（文件或 `OCLIVE_SHELL_PLUGIN_ID`）指向已扫描到的插件，且其 manifest 含 **`shell.entry`** 时，内置前端在挂载主应用**之前**调用 **`get_directory_plugin_bootstrap`**（可省略 `role_id`，与旧行为一致）。

- **发行构建始终忽略 `shell.vueEntry`**；若存在 **`shellUrl`**，宿主将其挂载为覆盖主界面的 `sandbox="allow-scripts"` 全屏 iframe。该 frame 为不透明源，不能访问宿主 DOM，也不会获得仅注入主 frame 的 Tauri IPC 初始化脚本。
- 只有 Vite DEV + **`VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1`** + `force_iframe_mode=false` 三项同时满足时，宿主才会挂载 **`DirectoryShellApp.vue`** + **`AsyncPluginVue`**。该模式继承主 WebView 权限，只用于审过源码的本地调试。
- HTML 入口缺失或不可达时回退内置主界面，不再为了体验自动执行不可信 Vue。

**`shellUrl` 形态**：Linux / macOS / iOS 使用 `ocliveplugin://localhost/<manifest.id>/<entry>`；Windows / Android 在宿主启用 `useHttpsScheme` 后使用 `https://ocliveplugin.localhost/<manifest.id>/<entry>`。两种形式均由同一个 `ocliveplugin` 协议处理器提供资源。

**静态资源**：由宿主自定义 `ocliveplugin` protocol 从磁盘插件根读取（路径穿越会拒绝）。协议处理器只接受 Wry 回传的 `ocliveplugin://localhost/<id>/<entry>` 与映射后的 `http(s)://ocliveplugin.localhost/<id>/<entry>`；拒绝时返回 `PLUGIN_ASSET_URI_INVALID`，并写入 `oclive_plugin` 日志。整壳 frame **没有** custom-protocol remote IPC capability；所有调用经 source-bound parent broker 转发。broker 在首次 `load` 后发放一次性随机绑定 token；同一 frame 后续导航会被撤销权限，避免跨插件页面继承旧身份。禁止为 `https://ocliveplugin.localhost/**` 恢复 remote capability 或旧式 `dangerousRemoteDomainIpcAccess`。

### 4.1 整壳前端桥接（`shell.bridge`）

若 **`shell`** 下声明 **`bridge`**，且 **`invoke`** / **`events`** 非空：宿主在提供 **`shell.entry` 对应 HTML** 时会在 `</body>` 前注入脚本，挂载 **`window.OclivePluginBridge`**；若走 **`shell.vueEntry`** Vue 整壳，则由 **`provide('oclive', …)`** 注入同一套 **`invoke` / `events`**（底层仍走 **`plugin_bridge_invoke`**）。

- **`invoke(command, params)`**：manifest 的 **`bridge.invoke`** 为**权限列表**：可写 **命令名**（如 `send_message`）或 **权限别名**（如 `read:conversation`）；与下表对应。由 **`plugin_bridge_invoke`** 二次校验。
- **`listen(event, handler)`**：隔离 frame 当前仅转发插件自身命名空间事件（`<pluginId>:*`）；宿主事件仍 fail closed，待身份绑定阶段提供逐插件声明校验。unsafe DEV Vue 可使用宿主事件总线。

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
| **`export_conversation`** | **`export:conversation`** 或 `export_conversation` | 导出当前角色聊天记录；`params`: `role_id`，可选 `format`（`json` \| `txt`，默认 `json`）、`session_id`；返回 `content`、`suggested_filename`（与 `export_chat_logs` 一致） |
| **`import_role`** | **`import:role`** 或 `import_role` | 导入角色包；`params`: `path`（或 `src_path`），可选 `overwrite`；返回 `role_id`、`ok` |
| **`update_prompt`** | **`write:prompt`** 或 `update_prompt` | 预留：当前宿主返回 `not_implemented`，待动态提示词片段契约落地 |
| **`delete_role`** | **`delete:role`** 或 `delete_role` | 删除本地角色包及相关数据；`params`: `role_id` 或 `roleId` |
| **`update_settings`** | **`write:settings`** 或 `update_settings` | 更新允许的应用设置（白名单字段，如 `theme` / `ui_theme`、`interaction_mode`） |
| **`get_conversation_list`** | **`read:conversations`** 或 `get_conversation_list` | 返回本地会话元数据列表：`items[]` 含 `session_namespace`、`turn_count`、`last_at` |

**不强制 `type: ocliveplugin` 的桥接命令**（亦需在 **`bridge.invoke`** 中声明）：`get_role_info`、`list_roles`、`get_time_state`、`get_directory_plugin_bootstrap`、**`get_plugin_settings_ui`** / **`set_plugin_settings_config`**（插件私有 `config.json`；**桌面本地** `plugin_config.rs`，经 `plugin_bridge_invoke` → `dispatch_local_bridge_command`，**不**进内核 `dispatch_bridge_command`）、**`plugin_rpc_invoke`**（仅可调用**本插件** manifest **`rpcMethods`** 声明的方法，供 `ui_slots` 侧车 RPC；例：[`com.oclive.voice.asr`](../../distros/chat-pro/plugins/com.oclive.voice.asr/)）等。未声明的调用一律拒绝。

> **AI 接线纪律**：`ui_slots` 内 `oclive.invoke("…")` **一律**走 `plugin_bridge_invoke`。若命令在 `lib.rs` 另有顶层 Tauri 注册（如 `get_plugin_settings_ui`），仍须在 **`plugin_bridge.rs` 的 `dispatch_local_bridge_command`** 显式分发，否则报 `unsupported bridge command`。

**写入类命令**（`update_memory` / `delete_memory` / `update_emotion` / `update_event` / `update_prompt`）以及 **`export_conversation`** / **`import_role`** 与上表「聊天/角色」敏感命令相同：**必须** `type: ocliveplugin` 且自 **`shell.entry` HTML** 或 **`shell.vueEntry` Vue** 调用。

### 4.2 主界面 UI 插槽（`ui_slots`）

官方支持的 **`slot`** 值：

| `slot` | 宿主位置 | 说明 |
|--------|----------|------|
| **`chat_toolbar`** | 聊天输入区上方 | 窄条工具栏，适合快捷操作 |
| **`settings.panel`** | **设置 → 插件扩展**（顶栏「更多」→「打开设置」） | 较大区域，适合插件配置表单；可用 **选项卡** 在多个 `settings.panel` 插件间切换 |
| **`role.detail`** | 左侧 **角色详情**（立绘与名称下方，好感度条上方） | 垂直 iframe 列表，适合与当前角色相关的扩展信息或快捷编辑 |
| **`sidebar`** | 左侧栏 **角色块下方**（好感度条上方） | 侧栏扩展区，适合与当前角色相关的竖向信息或工具 |
| **`chat.header`** | 右侧聊天列 **消息列表上方** | 聊天页顶栏区域，适合会话级提示或快捷条 |

通用规则：

- 若 manifest **无** **`shell`** 段，可在 **`ui_slots`** 中声明嵌入 UI：**`entry`** 为相对插件根的 HTML（**iframe 回退**）。
- 可选 **`vueComponent`**：相对插件根的 **`.vue`** 路径（如 `"slots/ToolbarButton.vue"`），仅供显式 unsafe DEV 调试；发行构建始终使用平台映射的 `ocliveplugin` 自定义协议加载 `entry` HTML。
- **含 `shell` 的插件不参与插槽**（避免与整壳重复）。
- 插槽页若需调用宿主能力：在对应 **`ui_slots[]` 条目**上配置 **`bridge`**。iframe 页仅当请求资源与 **`entry`** 一致时注入 `OclivePluginBridge`；**原生 Vue 插槽**通过 `inject('oclive')` 获得 API（见下），`plugin_bridge_invoke` 校验时使用 manifest 中的 **`entry`** 作为 **`assetRel`**（与 `bridge` 白名单一致）。
- 示例：`examples/directory-plugin-ui-slot/`（仅 iframe）；**`examples/directory-plugin-ui-slot-vue/`**（`vueComponent` + 回退 HTML）。

### 4.2.1 原生 Vue 插槽（`vueComponent` · 仅 unsafe DEV）

| 字段 | 说明 |
|------|------|
| **`entry`** | 必填；iframe URL 与 bridge 权限锚点（`assetRel` 使用本字段的规范化相对路径）。 |
| **`vueComponent`** | 可选；插件根下 `.vue` 文件相对路径。组件需 `export default` 符合 Vue 3 组件；模板内使用 **`const oclive = inject('oclive')`**。 |

**`oclive` 对象（与整壳桥接能力对齐，经同一 `plugin_bridge_invoke` 后端）：**

- **`oclive.invoke(command, params?)`**：等价于 iframe 内 `OclivePluginBridge.invoke`。
- **`oclive.pluginId`** / **`oclive.bridgeAssetRel`**：当前插件 id 与桥接用 `entry` 路径。
- **`oclive.events.emit` / `on` / `off`**：宿主 **mitt** 事件总线（见 4.3）；`on` 注册的监听在**组件卸载时自动移除**。
- **`oclive.events.request(event, data?, timeoutMs?)`**：向已用 **`onRequest`** 注册的监听方发起**请求—响应**；事件名须为 **`某插件ID:名称`**（可跨插件，不要求与调用方 id 一致）；返回 **`Promise`**，超时默认 15s。多监听方时为 **`Promise.race`**（首个 fulfilled 的结果）。
- **`oclive.events.onRequest` / `offRequest`**：注册/移除请求处理器；`handler` 可同步或异步返回值。

样式可直接使用宿主 **CSS 变量**（如 `--fluent-accent`、`--bg-primary`、`--font-ui`、`--border-light` 等，见 `distros/shared/src/styles/theme.css`）。

**安全说明**：插件组件与主界面同 JS 上下文，静态扫描不是安全边界。发行构建一律禁用；开发者只有在审过源码后才应设置 `VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1`，并仍只通过 `oclive.invoke` 访问白名单命令。

### 4.3 事件总线（宿主内置）

| 事件名 | 触发时机 | `data`（建议可 JSON 序列化） |
|--------|----------|-------------------------------|
| **`role:switched`** | 用户切换当前角色成功 | `{ roleId: string }` |
| **`message:sent`** | 用户发送消息且本轮回复已返回 | `{ message: string, reply: string }` |
| **`theme:changed`** | 角色包 `ui.json` 主题主色应用到界面 | `{ primaryColor: string }` |

**按需广播（隐私与性能）**：宿主仅在**当前角色下已启用**的插件中，至少有一个在 manifest 的 **`shell.bridge.events`** 或某一 **`ui_slots[].bridge.events`** 中声明了该事件名时，才会向 Vue 插槽内的 `oclive.events` / mitt 总线广播对应内置事件。未声明则**不广播**（等同未监听）。`get_directory_plugin_bootstrap` 返回的 **`subscribedHostEvents`**（camelCase）为当前应广播的内置事件名列表（去重排序）；Tauri 命令 **`is_host_event_subscribed`** 也可按事件名查询。

**插件侧 `oclive.events` 命名空间（宿主校验）**

- **`emit`**：事件名须匹配 `/^[a-zA-Z0-9.-]+:/`，且**冒号前**的命名空间须**等于当前插件** `manifest.id`（例如插件 `com.a` 仅可 `emit('com.a:refresh')`）。`refresh`、`com.b:x`、无前缀等调用会被拒绝并 **`console.warn`**。
- **`on` / `off`**：允许 **`插件ID:…`** 形式（可监听其他插件发出的命名空间事件），或 **`oclive:`** 前缀的**内置事件**监听：例如 `oclive:role:switched` 对应总线上的 **`role:switched`**（与 `emitBuiltin` 使用的事件键一致）。
- **正例**：`com.my.plugin:sidebar-toggle`（emit）；`com.other.plugin:updated`（on，跨插件）；`oclive:message:sent`（on，内置）。
- **反例**：`emit('refresh')`；`emit('com.other: x')`（在 `com.a` 插件内）。

### 4.3.1 原生 Vue 安全扫描（开发者模式）

只有 unsafe inline Vue 已显式启用时该扫描才有意义；它用于提示 `fetch`、`eval`、`document.cookie`、`localStorage`、`window.__TAURI__` 等模式，**可绕过且不是沙箱**。

**编译失败提示**：开发专用 SFC 编译器报错时，插槽 UI 展示 **插件 id、组件路径、可读摘要**；可通过 **「查看详情」** 展开原始堆栈。编译器只在 Vite DEV + 显式不安全开关同时满足时动态加载，不进入发行 bundle。

**`ui_slots` 脚本纪律**：插槽 `.vue` 只允许从 **`vue`** 导入；相对/第三方脚本 import、外部 `script/template/style src` 与 CSS 预处理器会被明确拒绝。可复用逻辑请 **内联进 `.vue`**。这保持本地调试能力，同时避免宿主为目录插件再实现一套包解析器。语音侧车踩坑记录见 [TRACK_VOICE_RECOGNITION §10](../../human-docs/team/TRACK_VOICE_RECOGNITION.md)。

### 4.3.2 发行版隔离 / 强制 iframe 模式

发行构建无条件按 **`force_iframe_mode=true` 的有效语义**运行；设置页显示为锁定。磁盘字段仅在 Vite DEV + `VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1` 时可进一步控制是否允许 inline Vue。

**`get_directory_plugin_bootstrap` → `uiSlots`** 会返回上述插槽的条目（已按 `app_data/plugin_state.json` 中的 **`slot_order`** 分插槽排序）。示例：

```json
{
  "disabled_plugins": [],
  "slot_order": {
    "chat_toolbar": ["com.example.toolbar_a", "com.example.toolbar_b"],
    "settings.panel": ["com.example.settings_a", "com.example.settings_b"],
    "role.detail": ["com.example.role_extra"],
    "sidebar": ["com.example.sidebar_a"],
    "chat.header": ["com.example.chat_header_a"]
  },
  "disabled_slot_contributions": {
    "chat_toolbar": [],
    "settings.panel": [],
    "role.detail": [],
    "sidebar": [],
    "chat.header": []
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
| 扫描 / manifest / 懒启动 / shell URL | `kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/` |
| 枚举与 `directory_plugins` 槽位 | `kernel/crates/oclive_kernel_types/src/models/plugin_backends.rs` |
| 六模块解析与 HTTP 复用 | `kernel/crates/oclive_kernel_host/src/domain/ports/plugin_host.rs`、`kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/` |
| Tauri 命令 | `distros/desktop-tauri/src/api/directory_plugin.rs`、`distros/desktop-tauri/src/api/plugin_bridge.rs`、`distros/desktop-tauri/src/api/plugin_update.rs`（本地 zip 覆盖 / 更新检查预留） |
| 自定义协议 + 启动 | `distros/desktop-tauri/src/lib.rs` |
| 内置 UI 启动引导 | `distros/shared/src/main.js`、`distros/shared/src/utils/directoryShellBootstrap.ts`、`distros/shared/src/DirectoryShellApp.vue` |
| 聊天工具栏插槽 | `distros/shared/src/components/ChatPluginToolbarSlots.vue` |
| 设置页插槽 | `distros/shared/src/components/PluginSettingsPanelSlots.vue`、`distros/chat-pro/src/views/SettingsView.vue` |
| 角色详情插槽 | `distros/shared/src/components/PluginRoleDetailSlots.vue`、`distros/chat-pro/src/views/RoleDetailView.vue` |
| 前端封装 | `distros/shared/src/api/`（`getDirectoryPluginBootstrap`、`directoryPluginInvoke`、`pluginBridgeInvoke`） |

---

## 8. 仓库内最小示例

见 **`examples/directory-plugin-minimal/`**（含 **`Shell.vue`** + **`shell.vueEntry`** 示例）：可复制到 `distros/chat-pro/plugins/<id>/` 或加入 `extra_plugin_roots` 后，配置 `shell_plugin_id` 与（可选）`plugin_backends` 做联调。  
**LLM 槽 + 本机 llama.cpp**：**[`examples/directory-plugin-llamacpp/`](../../examples/directory-plugin-llamacpp/README.md)**（[English](../../examples/directory-plugin-llamacpp/README.en.md)）。  
**非整壳 + 工具栏插槽**：**`examples/directory-plugin-ui-slot/`**；**原生 Vue 工具栏 + iframe 回退**：**`examples/directory-plugin-ui-slot-vue/`**。

快速起手可用脚手架命令：

```bash
npm run scaffold:ui-plugin -- --id com.example.my-slot --slot role.detail --title "My Slot Card"
```

命令会生成 `distros/chat-pro/plugins/<id>/manifest.json`、`slots/slot.html`、`slots/SlotCard.vue` 三个文件；再把该插件 id 加入目标角色的 `ui.json` 对应 `slots.<slot>.order/visible` 即可。

---

## 9. 排错（常见问题）

| 现象 | 可能原因 |
|------|----------|
| 仍走 builtin / Ollama，日志提示 directory 缺失槽位 | `plugin_backends.* = directory` 但 **`directory_plugins.<槽>`** 未填或与 manifest **`id`** 不一致 |
| 整壳未显示 / 仍显示内置 UI | **`shell_plugin_id`** 未设或插件未扫描到；manifest 缺 **`shell.entry`**；`get_directory_plugin_bootstrap` 返回的 **`shellUrl`** 为空；URL 身份校验失败；或入口文件不可读 |
| Vue 整壳使用 HTML | 发行构建的安全默认；只有 unsafe DEV 双重 opt-in 后才检查 `shell.vueEntry` / Vue 编译 |
| 插件管理「从本地 zip 更新」失败 | zip 内无有效 **`manifest.json`**（根或单一顶层目录）；**`manifest.id`** 与所选插件 id 不一致；目标目录无法删除（占用中） |
| 整壳页里 **`invoke` 失败** | frame 未由宿主 broker 注册、bridge manifest 未声明命令，或插件/入口身份与 `shellUrl` 不一致 |
| 子进程启动失败 / 无 RPC | **`process.command`** 在 PATH 中不可用（如未装 Node）；**`manifest.json`** 语法错误；子进程未在超时内向 stdout 打印 **`OCLIVE_READY <url>`** |
| **`directory_plugin_invoke`** 报错 | **`pluginId`** 未扫描到；目标插件缺 **`process`** 节无法懒启动 RPC |

过滤宿主日志 target：**`oclive_plugin`**。
