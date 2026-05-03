# 内核入口清单（Kernel Entry Checklist）

> 从 `src-tauri/src/lib.rs` 的 `tauri::generate_handler!` 提取的当前对外能力清单  
> 所有 OOCP 方法映射见 `creator-docs/oocp/OOCP_SPEC_v0_1.md`  
> **命令 ↔ `api/` / 内核模块对照表**：[KERNEL_API_IMPLEMENTATION_MATRIX.md](./KERNEL_API_IMPLEMENTATION_MATRIX.md)

---

## 会话（Session）

| 命令名 | 输入 DTO | 输出 DTO | 事件 | 备注 |
|--------|----------|----------|------|------|
| `send_message` | `SendMessageRequest` | `SendMessageResponse` | — | 核心对话入口，字段 `reply` |

---

## 角色（Role）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `load_role` | — | `RoleData` |
| `get_role_info` | `GetRoleInfoRequest` | `RoleInfo` |
| `list_roles` | — | `Vec<RoleSummary>` |
| `switch_role` | — | — |
| `set_user_relation` | `SetUserRelationRequest` | — |
| `set_scene_user_relation` | `SetSceneUserRelationRequest` | — |
| `clear_scene_user_relation` | `ClearSceneUserRelationRequest` | — |
| `set_evolution_factor` | `SetEvolutionFactorRequest` | — |
| `set_remote_life_enabled` | `SetRemoteLifeEnabledRequest` | — |
| `set_role_interaction_mode` | `SetRoleInteractionModeRequest` | — |
| `set_session_plugin_backend` | `SetSessionPluginBackendRequest` | — |
| `apply_author_suggested_plugin_backends` | — | — |
| `get_plugin_resolution_debug` | `GetPluginResolutionDebugRequest` | `PluginResolutionDebugInfo` |
| `resolve_role_asset_path` | — | — |

---

## 场景（Scene）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `switch_scene` | `SwitchSceneRequest` | `SwitchSceneResponse` |
| `set_user_presence_scene` | `SetUserPresenceSceneRequest` | — |

---

## 角色包（Role Pack）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `export_role_pack_command` | — | — |
| `peek_role_pack_command` | — | `RolePackPeekResponse` |
| `import_role_pack_command` | — | `ImportProgress`（事件推送） |

---

## 时间（Time）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `get_time_state` | — | `TimeStateResponse` |
| `jump_time` | `JumpTimeRequest` | `JumpTimeResponse` |

---

## 独白（Monologue）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `generate_monologue` | `GenerateMonologueRequest` | `GenerateMonologueResponse` |

---

## 导出（Export）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `export_chat_logs` | `ExportChatLogsRequest` | `ExportChatLogsResponse` |

---

## 记忆（Memory）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `query_memories` | `QueryMemoriesRequest` | `Vec<MemoryItem>` |

---

## 事件（Event）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `query_events` | `QueryEventsRequest` | `Vec<EventItem>` |
| `create_event` | `CreateEventRequest` | `CreateEventResponse` |

---

## 策略（Policy）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `reload_policy_plugins` | — | — |

---

## 插件 - 目录（Directory Plugin）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `get_directory_plugin_bootstrap` | — | — |
| `read_plugin_asset_text` | — | — |
| `is_host_event_subscribed` | — | — |
| `get_directory_plugin_catalog` | — | — |
| `get_plugin_state` | — | — |
| `save_plugin_state` | — | — |
| `save_global_plugin_state` | — | — |
| `reset_plugin_state_to_role_default` | — | — |
| `directory_plugin_invoke` | — | — |

---

## 插件 - 脚手架/打包/调试（Plugin Dev）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `create_plugin_scaffold` | — | — |
| `pack_plugin` | — | — |
| `spawn_plugin_for_test` | — | — |
| `kill_plugin_process` | — | — |
| `list_plugin_processes` | — | — |
| `get_plugin_logs` | — | — |
| `clear_plugin_logs` | — | — |
| `test_plugin_method` | — | — |
| `discover_plugin_methods` | — | — |

---

## 插件 - 桥/更新/市场（Plugin Bridge / Market）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `plugin_bridge_invoke` | — | — |
| `check_plugin_updates` | — | — |
| `extract_plugin_zip` | — | — |
| `sync_plugin_index_command` | — | — |
| `get_cached_plugin_index` | — | — |
| `install_plugin_from_market` | — | — |
| `install_plugin_from_git` | — | — |
| `update_plugin_from_market` | — | — |
| `uninstall_plugin_from_market` | — | — |
| `batch_update_plugins` | — | — |
| `batch_uninstall_plugins` | — | — |
| `consume_pending_protocol_installs` | — | — |

---

## 插件 - 配置（Plugin Config）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `get_plugin_settings_ui` | — | — |
| `set_plugin_settings_config` | — | — |

---

## 快捷键（Hotkeys）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `get_hotkey_bindings` | — | — |
| `save_hotkey_bindings` | — | — |

---

## Agent（Agent / MCP）

| 命令名 | 输入 DTO | 输出 DTO |
|--------|----------|----------|
| `list_mcp_servers` | — | — |
| `list_mcp_tools` | — | — |
| `call_mcp_tool` | — | — |
| `get_agent_debug_traces` | — | — |
| `clear_agent_debug_traces` | — | — |

---

## 事件/Stream（现有 Tauri event 推送）

| 事件名 | Payload | 备注 |
|--------|---------|------|
| `protocol:pending_install` | `{ "reason": "deep-link" }` | 深度链接触发 |
| `plugin:*` | 取决于插件订阅 | 插件桥事件 |

---

> **OOCP 映射**：上表为 Tauri invoke 视角；所有方法在 OOCP 中的对应见 `creator-docs/oocp/OOCP_SPEC_v0_1.md`。