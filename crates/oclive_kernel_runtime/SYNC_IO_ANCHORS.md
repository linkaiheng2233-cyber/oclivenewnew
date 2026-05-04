# 同步磁盘 I/O 与 `block_on` 锚点（质量加固扫描）

> **异步化改造终局（边缘锚点审计）**：`mcp_client` 热路径已 **`tokio::fs`**（见下表）。表中其余 **`std::fs`** 均为 **有意保留**：同步宿主入口、目录插件进程边界、`spawn_blocking` 内解压，或 API 兼容层；**不再列为「待迁移」**。新增代码须遵守文末原则。

本文档与 `infrastructure/blocking_http.rs` 模块注释互为索引。

## `oclive_kernel_runtime`：`std::fs`（生产代码）

| 区域 | 说明 | 处置 |
|------|------|------|
| `infrastructure/directory_plugins/runtime.rs` | 子进程侧车、manifest、安装元数据；扫描与同步回调路径 | **保留**：与进程生命周期、同步 `reload_plugin_state` 共存 |
| `infrastructure/directory_plugins/manifest.rs` / `install_meta.rs` / `assets.rs` | 目录插件元数据读写 | **保留**：多在同步扫描与构造路径 |
| `infrastructure/plugin_state.rs` | 同步 `load`/`save` 与 `*_async` 并存 | **保留**：兼容旧调用方；热路径用 `load_async`/`save_async` |
| `infrastructure/hotkey_bindings.rs` | `load`/`save` 同步入口 | **保留**：兼容；推荐 `load_async`/`save_async` |
| `infrastructure/storage.rs` | `RoleStorage` 同步读角色包 | **保留**：宿主 `load_role` 同步语义 |
| `infrastructure/role_pack_archive.rs` | ZIP/目录导入导出 | **`spawn_blocking`** 内 `std::fs`（见实现） |
| `infrastructure/mcp_client.rs`（`kernel-agent`） | MCP manifest 发现 | **已 async**：`list_servers` → `tokio::fs::create_dir_all` + `read_dir`/`read_to_string` |
| `domain/expert_models_admin.rs` | GGUF 仓库与列表 | **已 tokio::fs**（Tauri/async API） |
| `domain/role_lifecycle.rs` | 删除角色目录 | **`spawn_blocking` + `remove_dir_all`** |
| `models/ui_config.rs` | UI 配置读盘 | **保留**：同步加载辅助 |
| `domain/local_plugin_bridge.rs` | 同步 I/O | **`cfg(test)`** 或桥接专用 |

**原则**：在 **`async fn` 长时间占用 Tokio worker** 的场景，避免直接 `std::fs`；优先 **`tokio::fs`** 或 **`spawn_blocking`**。

## `block_on` / `block_in_place`

| 位置 | 用途 | 处置 |
|------|------|------|
| `infrastructure/blocking_http.rs` | 无 Tokio runtime 时为 `call_async` 提供独立 runtime | **保留**：JSON-RPC 同步 trait 兜底 |
| `infrastructure/remote_plugin/jsonrpc.rs` | `call_blocking`：`block_in_place` + `Handle::block_on`，否则 `blocking_http::block_on` | **保留**：同步 Provider 边界 |
| `domain/plugin_host.rs` | `BackendRegistry::block_on`：同步回调内桥接 `sqlx` | **保留**：权限/注册同步 API |
| `domain/role_manager.rs` | 测试内嵌 `Runtime::block_on` | **保留**：非生产 |

## `src-tauri`

| 位置 | 说明 |
|------|------|
| `lib.rs` | 启动时 `block_on` 初始化 `AppState` |
| `api/plugin_index.rs` / `api/plugin_update.rs` | 同步命令内桥接异步落库 |
| `api/local_imports.rs` / `api/role_pack.rs` / `api/directory_plugin.rs` / `infrastructure/plugin_installer.rs` | 安装路径 `std::fs`：与 Tauri 同步语义一致 |

## 基准

性能基线见 `benches/kernel_hot_paths.rs`、`benches/kernel_plugins_persistence.rs`（Criterion）；CI 对比见 `creator-docs/kernel/kernel_perf_baseline_v0.json` 与 `scripts/criterion_compare_baseline.py`。

## 历史备注（P1 分批迁移，已收口）

- **`expert_models_admin`**、`**http_api**` 热路径迁移见 `handoff/P1_KERNEL_RUNTIME_BLOCKING_AND_STARTUP.md`。
- **新增读盘**：须在 **async** 上下文使用 **`tokio::fs`** 或 **`spawn_blocking`**。
