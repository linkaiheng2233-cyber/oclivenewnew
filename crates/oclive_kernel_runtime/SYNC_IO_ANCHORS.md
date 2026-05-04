# 同步磁盘 I/O 与 `block_on` 锚点（质量加固扫描）

本文档与 `infrastructure/blocking_http.rs` 模块注释互为索引：列出 **`std::fs` 同步调用**与 **`block_on` 桥接**的用途，便于审计「应保留 / 应在 `spawn_blocking` / 应 async」。

## `oclive_kernel_runtime`：`std::fs`（生产代码）

| 区域 | 说明 |
|------|------|
| `infrastructure/directory_plugins/runtime.rs` | 子进程侧车、manifest、安装元数据；多在启动/扫描路径；重 I/O 与解压保持同步或配合进程边界。 |
| `infrastructure/directory_plugins/manifest.rs` / `install_meta.rs` / `assets.rs` | 目录插件元数据读写；与 `DirectoryPluginRuntime` 生命周期绑定。 |
| `infrastructure/plugin_state.rs` | 同步 API 与 `*_async` 并存；热路径优先 `tokio::fs`（见 `blocking_http` 注释）。 |
| `infrastructure/hotkey_bindings.rs` | 提供 `load_async` / `save_async`；遗留同步入口仅兼容。 |
| `infrastructure/storage.rs` | `RoleStorage` 同步读角色包；宿主加载阶段。 |
| `infrastructure/role_pack_archive.rs` | ZIP/目录导入导出主体在 **`spawn_blocking`** 内使用 `std::fs`。 |
| `infrastructure/mcp_client.rs`（`kernel-agent`） | manifest 发现等；与进程/HTTP 调用分离，体量小。 |
| `domain/expert_models_admin.rs` | 管理侧模型路径探测；非对话热路径。 |
| `domain/role_lifecycle.rs` | 与包加载相关的同步读。 |
| `models/ui_config.rs` | UI 配置读盘。 |
| `domain/local_plugin_bridge.rs` | 测试/桥接中的同步 I/O（见该文件 `cfg(test)`）。 |

**原则**：在 **`async fn` 长时间占用 Tokio worker** 的场景，避免直接 `std::fs`；优先 `tokio::fs` 或 `spawn_blocking`（与 `blocking_http` 文档一致）。

## `block_on` / `block_in_place`

| 位置 | 用途 |
|------|------|
| `infrastructure/blocking_http.rs` | 无 Tokio runtime 时驱动 `reqwest` 等异步 HTTP。 |
| `infrastructure/remote_plugin/jsonrpc.rs` | `call_blocking`：有 runtime 时用 `block_in_place` + `Handle::block_on`；否则回退 `blocking_http::block_on`。 |
| `domain/plugin_host.rs` | `BackendRegistry::block_on`：同步权限/注册回调内桥接 `sqlx`。 |
| `domain/role_manager.rs` | 测试内嵌 `Runtime::block_on`（非生产路径）。 |

## `src-tauri`

| 位置 | 说明 |
|------|------|
| `lib.rs` | 启动时 `block_on` 初始化 `AppState` / HTTP 试聊等。 |
| `api/plugin_index.rs` / `api/plugin_update.rs` | 部分同步命令内在落库前 `tauri::async_runtime::block_on`；属「同步 API 边界」桥接，与 `plugin_update` 中先 `std::fs` 装插件再写权限表一致。 |
| `api/local_imports.rs` / `api/role_pack.rs` / `api/directory_plugin.rs` / `infrastructure/plugin_installer.rs` | 用户导入/安装路径上的 `std::fs`；与 Tauri 命令同步语义一致。 |

## 基准

性能基线见 `benches/kernel_hot_paths.rs` 与 `benches/kernel_plugins_persistence.rs`（Criterion）。
