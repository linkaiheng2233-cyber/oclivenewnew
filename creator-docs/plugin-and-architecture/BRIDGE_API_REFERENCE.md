# 整壳桥接 API 完整参考

本文档描述 **目录插件** 在 HTML 整壳或 **`shell.vueEntry` 宿主 Vue 页** 中，通过 **`OclivePluginBridge.invoke`**（或原生 Vue 插槽的 **`inject('oclive').invoke`）可调用的**命令列表、权限别名与典型参数/返回值。

**权威实现**：`src-tauri/src/api/plugin_bridge.rs`（`required_permission_token`、`dispatch_bridge_command`、`validate_bridge`）。

**前置阅读**：[DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) §4.1–4.3（整壳桥接、敏感命令门禁、事件总线）。

---

## 1. 概述

### 调用方式

- **iframe 整壳**：宿主在 `shell.entry` 对应 HTML 中注入 `window.OclivePluginBridge`，使用 **`OclivePluginBridge.invoke(command, params)`**。
- **Vue 整壳**：`shell.vueEntry` 由宿主挂载时，使用 **`const oclive = inject('oclive'); await oclive.invoke(command, params)`**，与 iframe 走同一后端 `plugin_bridge_invoke`。

底层均为 Tauri 命令 **`plugin_bridge_invoke`**，请求体包含：

- `pluginId`：插件 `manifest.id`
- `assetRel`：当前页面对应资源相对路径（整壳须为 **`shell.entry`** 或 **`shell.vueEntry`** 的规范化相对路径，与 manifest 中 `bridge` 校验一致）
- `command`：下表中的命令名字符串
- `params`：JSON 对象（各命令字段见下表）

### 权限声明机制

在 **`manifest.json`** 中：

- **`shell.bridge.invoke`**（整壳）或对应 **`ui_slots[].bridge.invoke`**（插槽页）：数组内可写 **命令名**（如 `send_message`）或 **权限别名**（如 `read:conversation`），二者命中其一即可。
- **`shell.bridge.events`** / **`ui_slots[].bridge.events`**：允许 `OclivePluginBridge.listen` / `oclive.events` 订阅的宿主事件名（见 §4）。

未在 `invoke` 中声明的命令会被拒绝（`[API_PERMISSION_DENIED]`）。

### 整壳深度集成（敏感命令）

以下命令除需 **`bridge.invoke`** 命中外，还要求：

1. manifest 顶层 **`"type": "ocliveplugin"`**
2. 调用来源为 **`shell.entry` 对应 HTML** 或 **`shell.vueEntry` 宿主 Vue 页**（**不得**从 `ui_slots` 页面调用）

标记为 **「敏感」** 的命令均属于此类（见下表「敏感」列）。

**不强制 `ocliveplugin` 的桥接命令**（仍需在 `invoke` 中声明）：如 `get_role_info`、`list_roles`、`get_time_state`、`get_directory_plugin_bootstrap` 等。

---

## 2. 命令列表

参数与返回值字段名以 **JSON（camelCase 与 snake_case 混用以匹配前端现有契约）** 为准；以下为当前实现中的常用形态。

| 命令 | 功能 | 权限别名 | 敏感 | 参数示例 | 返回值示例（节选） |
|------|------|----------|------|----------|---------------------|
| `send_message` | 发送用户消息，触发对话引擎推理 | `send_message` | 是 | `{ "role_id": "my_role", "user_message": "你好" }`，也可用顶层 **`text`** 代替 `user_message` | `SendMessageResponse` 序列化对象（含回复、情绪、事件等，见宿主 `process_message`） |
| `get_conversation` | 读取短期对话轮次 | `read:conversation` | 是 | `{ "role_id": "my_role", "session_id": null, "limit": 50, "offset": 0 }` | `{ "role_id", "session_namespace", "total", "limit", "offset", "items": [ { "user_input", "bot_reply", "emotion", "scene", "created_at" } ] }` |
| `switch_role` | 切换到指定角色 | `switch_role` | 是 | `{ "role_id": "other_role" }` | `RoleInfo` |
| `get_roles` | 列出所有角色摘要 | `read:roles` | 是 | `{}` | `RoleSummary[]` |
| `get_current_role` | 与 `get_role_info` 等价（命名兼容） | `read:current_role` | 是 | `{ "role_id": "...", "session_id": null }` 或 `{ "req": { ... } }` | `RoleInfo` |
| `get_role_info` | 获取指定角色运行时信息 | `get_role_info` | 否 | 同上 | `RoleInfo` |
| `list_roles` | 同 `get_roles` | `list_roles` | 否 | `{}` | `RoleSummary[]` |
| `get_time_state` | 获取角色时间状态 | `get_time_state` | 否 | `{ "roleId": "..." }` 或 `{ "role_id": "..." }` | `TimeStateResponse` |
| `get_directory_plugin_bootstrap` | 获取目录插件引导（整壳 URL、插槽、订阅事件等） | `get_directory_plugin_bootstrap` | 否 | `{ "roleId": "..." }` 可选 | `DirectoryPluginBootstrapDto` |
| `update_memory` | 写入长期记忆 | `write:memory` | 是 | `{ "role_id": "...", "content": "...", "importance": 0.5 }` | `{ "memory_id": "..." }` |
| `delete_memory` | 删除长期记忆 | `write:memory` | 是 | `{ "role_id": "...", "memory_id": "..." }` | `{ "ok": true }` |
| `update_emotion` | 更新当前情绪标签 | `write:emotion` | 是 | `{ "role_id": "...", "emotion": "happy" }` | `{ "ok": true }` |
| `update_event` | 创建/记录事件 | `write:event` | 是 | `{ "role_id": "...", "event_type": "...", "description": "..." }` | 与 `create_event` 一致 |
| `export_conversation` | 导出聊天记录 | `export:conversation` | 是 | `{ "role_id": "...", "format": "json", "session_id": null }` | 与 `export_chat_logs` 一致（如 `content`、`suggested_filename`） |
| `import_role` | 导入角色包 | `import:role` | 是 | `{ "path": "C:/path/to.pack.zip", "overwrite": false }`，也可用 **`src_path`** | `{ "role_id": "...", "ok": true }` |
| `delete_role` | 删除本地角色 | `delete:role` | 是 | `{ "role_id": "..." }` 或 `{ "roleId": "..." }` | 与 `delete_role` 命令一致 |
| `update_settings` | 更新应用设置（白名单字段） | `write:settings` | 是 | 见 `update_settings_impl` | 与宿主一致 |
| `get_conversation_list` | 会话列表元数据 | `read:conversations` | 是 | `{}` | `{ "items": [ { "session_namespace", "turn_count", "last_at" } ] }` |
| `update_prompt` | 动态提示词片段（预留） | `write:prompt` | 是 | （未接线） | `{ "ok": false, "error": "not_implemented", "message": "..." }` |

**说明**：

- `get_roles` 与 `list_roles` 在实现上均调用 `list_roles_impl`，返回值相同。
- 参数缺省会返回带 **`[INVALID_PARAMETER]`** 的字符串错误（见 §3）。

---

## 3. 错误码说明

宿主与桥接层会将失败信息格式化为 **`[CODE] 说明`**，前端可解析首段 `CODE`（见 `src/utils/tauri-api.ts`：`parseApiErrorCode`、`toFriendlyErrorMessage`）。

| 代码 | 含义 |
|------|------|
| `API_PLUGIN_NOT_FOUND` | 目录中未找到对应 `plugin_id` |
| `API_PERMISSION_DENIED` | 无 bridge 权限、`type` 非 `ocliveplugin`、或调用来源非整壳入口等 |
| `API_INVALID_MANIFEST` | `manifest.json` 无法加载或校验失败 |
| `INVALID_PARAMETER` | 参数缺失、JSON 无法解析、不支持的 `command` 等 |
| `IO_ERROR` | 含宿主将响应序列化为 JSON 失败等（正文中可能出现 `host json …`） |
| `DB_ERROR` / `ROLE_NOT_FOUND` / 等 | 来自 `AppError::to_frontend_error()` 的数据库与业务错误（与主应用其它命令一致） |

---

## 4. 事件订阅（内置）

在 **`bridge.events`** 中声明后，方可使用 `listen` / `oclive.events.on` 订阅对应内置事件（按需广播，见 [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) §4.3）。

| 事件名（总线键） | 触发时机 | `data`（建议可 JSON 序列化） |
|------------------|----------|--------------------------------|
| `role:switched` | 用户切换当前角色成功 | `{ "roleId": string }` |
| `message:sent` | 用户发送消息且本轮回复已返回 | `{ "message": string, "reply": string }` |
| `theme:changed` | 角色包 `ui.json` 主题主色应用 | `{ "primaryColor": string }` |

Vue 侧监听 **`oclive:`** 前缀时，例如 **`oclive:role:switched`** 对应总线上的 **`role:switched`**。

---

## 5. 相关文档

- [DIRECTORY_PLUGINS.md](DIRECTORY_PLUGINS.md) — 目录插件总览、`manifest`、`plugin_bridge_invoke`
- [../getting-started/ERROR_CODES.md](../getting-started/ERROR_CODES.md) — 用户可见错误与排障
