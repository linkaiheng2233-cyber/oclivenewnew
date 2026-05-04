# 同步磁盘 I/O 与 `block_on` 锚点（质量加固扫描）

> **异步化扫尾已完成**（2026-05）：`kernel-agent` 下 `mcp_client` 的 MCP 根目录创建与 manifest 枚举已走 **`tokio::fs`**；其余表中条目为**有意保留**的同步边界（目录插件生命周期、角色包加载、同步 API 兼容层等），见各表「说明」列。

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
| `infrastructure/mcp_client.rs`（`kernel-agent`） | **已 async**：`list_servers` 内 **`tokio::fs::create_dir_all`** + **`read_dir` / `read_to_string`**；构造 `new` 无磁盘 I/O。 |
| `domain/expert_models_admin.rs` | **P1 第四批**：本地 GGUF 列表/导入/删改与 `.oclive_gguf_repo.json` 已改为 **`tokio::fs`**；由 Tauri **`async` command** 调用，不占用 worker 同步阻塞读盘。 |
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

## P1-1 分批迁移备注（第四批收口）

- **`expert_models_admin`**：已迁移至 **`tokio::fs`**（含 `read_dir` / `metadata` / `canonicalize` / 原子写 repo JSON）；对应 **`src-tauri/src/api/expert_models.rs`** 七条命令改为 **`async`**。
- **`http_api`**：`POST /chat` 角色加载已在 **`spawn_blocking`**；`serve_api_with_options` 对 **`app_data_dir`** 使用 **`tokio::fs::create_dir_all`**；无额外同步 `std::fs` 热路径。
- **新增读盘**：须在 **`async` 上下文**使用 **`tokio::fs` 或 `spawn_blocking`**；见 `infrastructure/blocking_http.rs`。
