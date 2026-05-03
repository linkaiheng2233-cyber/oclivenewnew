# P1：`oclive_kernel_runtime` 阻塞 I/O 与启动分段（清单）

> 与 [`ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md`](./ENGINEERING_ROADMAP_KERNEL_DEEPSEEK.md) §二 P1、[`PERF_PHASES.md`](./PERF_PHASES.md) 对齐；本页为 **runtime crate 内** 可 grep 的锚点，便于分段计时与后续迁移。

## 1. 已使用 `tokio::task::spawn_blocking` 的位置

| 文件 | 用途 |
|------|------|
| `src/http_api.rs` | HTTP 试聊路径：角色目录阻塞读 / 探测 |
| `src/domain/adapters/runtime_oocp_handler.rs` | OOCP：`load_role_cached`、`load_all_roles` |
| `src/domain/role_lifecycle.rs` | `delete_role`：`remove_dir_all` |
| `infrastructure/role_pack_archive.rs` | `export` / `import` / `install_role_pack_from_direct_url`：磁盘段 `spawn_blocking` |
| `infrastructure/plugin_install.rs` | `install_plugin_from_download_urls_at`：解压落盘 `spawn_blocking` |

原则：**async 任务内**不直接跑长阻塞磁盘逻辑；新增路径先对照上表再决定进 `spawn_blocking` 还是独立线程。

## 2. 同步 HTTP 边界（`blocking_http::block_on`）

**角色/插件/评价市场索引** HTTP 已改为 **`async` + `.await`**（见 `*_index_sync.rs`），**不再**使用 `block_on`。  
**插件包市场 ZIP**：下载 async；解压经 **`spawn_blocking`**。**角色包归档**（`role_pack_archive`）对外均为 **`async fn`**；HTTP 下载 async，解压/导入在 **`spawn_blocking`**。**MCP**、**目录插件 JSON-RPC**（`invoke_directory_plugin_rpc`）已 **async + `.await`**。同步 trait 内的 `jsonrpc::call_blocking` 在 runtime 内走 **`block_in_place`**，无 runtime 时仍可能用 `blocking_http::block_on`。详见 `PERF_PHASES.md` P4 与 `infrastructure/blocking_http.rs`。

## 2b. `tokio::fs`（热键、插件状态、HTTP API 启动）

- `infrastructure/hotkey_bindings.rs`：`load_async` / `save_async`  
- `infrastructure/plugin_state.rs`：`load_async` / `save_async`；`plugin_install::remove_plugin_from_plugin_state_file_async`；`DirectoryPluginRuntime::reload_plugin_state_async`  
- `DirectoryPluginRuntime::set_active_role_id_async`：`oclive_last_role_id.txt` 使用 **`tokio::fs::write`**（`load_role` 路径）
- `http_api.rs`：`serve_api_with_options` 对 **`app_data_dir`** 使用 **`tokio::fs::create_dir_all`**（P1 第四批遗留锚点收口）
- Tauri：`get_hotkey_bindings` / `save_hotkey_bindings`、`uninstall_plugin` / 市场卸载命令已走上述异步路径；**目录卸载**中 `remove_dir_all` 使用 **`spawn_blocking`**。

## 3. `KernelAppState::new_in_memory_with_llm` 启动链（分段计时建议）

以下子步骤适合加 **毫秒级日志** 或 `tracing` span（不改变对外行为）：

1. 临时目录 + SQLite 文件 + `connect_with` + **`sqlx::migrate!`**  
2. `DbManager::new` + 各 `Sqlite*Repository` 构造  
3. `RoleStorage::new` + `DirectoryPluginRuntime::bootstrap`  
4. `PluginHost::new` + `bootstrap_local_plugin_providers`  
5. `PolicyRuntime` 默认构建（无 `policy.toml` 时）

无头 `kernel_server` 与桌面共用时，优先在 **冷启动** 与 **首次打开插件管理** 两条路径各采一次样，再决定「延迟初始化」是否值得（需行为评审，避免首次错误时机漂移）。

## 4. 后续工作（未在本文件实现）

- 在 `state/app_state.rs` 内落地 **可选** `tracing` feature 或 `log::info!` 分段（默认保持安静，由 env 打开）。  
- 将「非首屏」初始化（市场索引预拉、MCP 全量扫描等）与 **首屏对话** 解耦时，单独开 ADR / PR 说明 UX 与错误面变化。
