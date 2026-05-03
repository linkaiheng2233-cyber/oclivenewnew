# KERNEL API 实现对照表（Tauri invoke ↔ 代码归属）

> **基准**：[`KERNEL_ENTRY_CHECKLIST.md`](./KERNEL_ENTRY_CHECKLIST.md) 中的命令名与 DTO。  
> **原则**：下列路径为 **主实现入口**；内核逻辑在 `crates/oclive_kernel_runtime`，桌面适配在 `src-tauri/src/api/*.rs`。  
> **轻量编译 / SKU**：[`LIGHTWEIGHT_PROFILE.md`](./LIGHTWEIGHT_PROFILE.md)（`Cargo` 特性组合、OOCP、`invoke` 分组与去重拟定）。

## 列说明

| 列 | 含义 |
|----|------|
| **命令** | `generate_handler!` 注册的 Tauri 命令名 |
| **API 模块** | `src-tauri/src/api/` 下入口文件 |
| **内核归属** | `oclive_kernel_runtime` 中主要承载领域/编排的模块（无则留空表示仍以壳层或外部进程为主） |

---

## 会话

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `send_message` | `api/chat.rs` | `domain::chat_engine::process_message` |

## 角色 / 场景 / 时间 / 独白 / 导出

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `load_role`, `get_role_info`, `list_roles`, `switch_role`, … | `api/role/mod.rs` | `domain::role_*`, `state`, `models` |
| `switch_scene`, `set_user_presence_scene` | `api/scene.rs` | `domain::scene_commands` |
| `get_time_state`, `jump_time` | `api/time.rs` | `domain::virtual_time` |
| `generate_monologue` | `api/monologue.rs` | `domain::chat_engine` 等 |
| `export_chat_logs` | `api/export.rs` | `domain::export_chat_logs` |

## 角色包 / 市场

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `export_role_pack_command`, `peek_role_pack_command`, `import_role_pack_command` | `api/role_pack.rs` | `infrastructure::role_pack_archive`；展示目录见 `domain::role_paths` |
| `sync_role_market_index`, `install_role_pack_from_market` | `api/role_market.rs` | `infrastructure::role_market_index_sync` + `models::role_market_index`；直链安装见 `role_pack_archive::install_role_pack_from_direct_url`；桌面 `infrastructure/role_market.rs` 薄封装 |
| `get_cached_plugin_reviews_index`, `sync_plugin_reviews_index` | `api/plugin_reviews.rs` | `infrastructure::plugin_reviews_index_sync` + `models::plugin_reviews_index` |

## 记忆 / 事件 / 策略 / 反馈

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `query_memories` | `api/memory.rs` | `domain::memory_query`, repositories |
| `query_events`, `create_event` | `api/event.rs` | `domain::event_commands` |
| `reload_policy_plugins` | `api/policy.rs` | `domain::policy_host` |
| `create_role_feedback`, … | `api/role_feedback.rs` | DTO + DB（见 migrations） |

## 目录插件

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `get_directory_plugin_bootstrap`, `read_plugin_asset_text`, `get_directory_plugin_catalog`, … | `api/directory_plugin.rs` | `infrastructure::directory_plugins/*`, `domain::directory_plugin_commands` |
| `directory_plugin_invoke` | `api/directory_plugin.rs` | **壳层**：HTTP RPC + 进程生命周期；协议形状与内核 DTO 对齐 |

## 插件脚手架 / 调试 / 桥 / 市场 / 配置

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `create_plugin_scaffold`, `pack_plugin`, `spawn_plugin_for_test`, … | `api/plugin_scaffold.rs`, `plugin_pack.rs`, `plugin_debug.rs` | 校验 crate + 本地工具链 |
| `plugin_bridge_invoke` | `api/plugin_bridge.rs` | `domain::local_plugin_bridge` |
| `check_plugin_updates`, `extract_plugin_zip`, `sync_plugin_index_command`, … | `api/plugin_update.rs`, `plugin_index.rs`, … | `models::plugin_market_index`, `plugin_index_sync`（含 `resolve_plugin_index_url`）、`plugin_archive`, `plugin_package_verify`, `plugin_layout`, **`infrastructure::plugin_install`**；桌面 `plugin_installer.rs` 仅路径与 `rescan` / `reload_plugin_state` / `clear_plugin_process` |
| `get_plugin_settings_ui`, `set_plugin_settings_config` | `api/plugin_config.rs` | `infrastructure::plugin_config_disk` |

## 本地导入 / 快捷键 / Agent

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `list_local_import_candidates_command`, `read_local_import_text_command`, … | `api/local_imports.rs` | 扫描与路径：`domain::local_imports`；确认安装走 **`plugin_install`**（内核）；invoke 与同意流仍在 `api/local_imports.rs` |
| `get_hotkey_bindings`, `save_hotkey_bindings` | `api/hotkeys.rs` | `infrastructure::hotkey_bindings`（校验）；全局注册在壳层 |
| `list_mcp_servers`, `call_mcp_tool`, … | `api/agent.rs` | `domain::agent`, `infrastructure::mcp_client` |

## Expert Models（模块 9）

| 命令 | API 模块 | 内核归属 |
|------|----------|----------|
| `expert_models_*`, `expert_workflows_*` | `api/expert_models.rs` | `domain::expert_models_admin` |

---

## 废弃 / 模糊地带（清理标记）

| 现象 | 建议 |
|------|------|
| `preview_local_plugin_archive_command` / `install_local_plugin_archive_command` 曾未注册 | 已在 `lib.rs` 注册；若前端仍有死链需再扫一遍 |
| `notify` 曾列入 `oclive_kernel_runtime` 但未使用 | 已从依赖移除 |
| 远程插件 HTTP 与 Tauri 异步命令混用 | **已收敛**：workspace 无 `reqwest/blocking`；runtime 用 `Client` + `blocking_http::block_on`；详见 `handoff/PERF_PHASES.md` |

---

维护节奏：新增 `generate_handler!` 命令时，请同步更新 **KERNEL_ENTRY_CHECKLIST** 与本表一行。  
迁入收尾与自检命令：[**`../../handoff/KERNEL_MIGRATION_COMPLETE.md`**](../../handoff/KERNEL_MIGRATION_COMPLETE.md)。  
轻量 profile 与特性矩阵：[**`LIGHTWEIGHT_PROFILE.md`**](./LIGHTWEIGHT_PROFILE.md)。  
内核 SDK（库 / `kernel_server`）：[**`KERNEL_SDK.md`**](./KERNEL_SDK.md)。  
工程质量路线：[**`../../handoff/ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md`**](../../handoff/ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md)。
