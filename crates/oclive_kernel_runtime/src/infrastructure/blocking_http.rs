//! 在**无法持有 async 上下文**时，用独立 Tokio runtime 驱动 `reqwest`（工作区未启用 `reqwest/blocking`）。
//!
//! ## 仍调用 [`block_on`] 的代码路径（排查清单）
//!
//! 下列路径在 **无 Tokio runtime**（如部分单测）时仍可能回退到本模块；**市场三索引**、角色包直链、MCP、目录 RPC 等已在宿主侧改为 **`async` + `.await`**：
//!
//! - `infrastructure::role_pack_archive` — 直链下载已 async；解压/导入在 **`spawn_blocking`**
//! - `infrastructure::plugin_install` — 市场 ZIP 下载 async；**`install_plugin_from_archive_bytes_impl`** 在 **`spawn_blocking`**（`install_plugin_from_download_urls_at`）
//! - `infrastructure::mcp_client` — **`async`** + `reqwest` / `tokio::process`（无 `block_on`）
//! - `infrastructure::remote_plugin::jsonrpc::call_blocking` — 在 Tokio worker 上优先 **`block_in_place` + `Handle::block_on(call_async)`**；无 runtime 时仍用本模块 `block_on`
//! - `infrastructure::remote_plugin::invoke_directory_plugin_rpc` — **仅 `call_async` + `.await`**
//!
//! ## `std::fs` 同步磁盘 I/O
//!
//! **`hotkey_bindings` / `plugin_state`** 已提供 **`load_async` / `save_async`**（`tokio::fs`）；Tauri 热键与异步卸载路径应优先使用。
//! 目录插件 manifest、安装解压等仍多为同步读盘；宜在宿主侧 `spawn_blocking` 包裹，或后续分批改为 `tokio::fs`（与 HTTP 解耦）。
//!
//! ## `plugin_host::BackendRegistry::block_on`
//!
//! 见 `domain::plugin_host::BackendRegistry`：在目录插件权限等**同步回调**中桥接 `sqlx`；与本文的 HTTP runtime **相互独立**。

use once_cell::sync::Lazy;
use std::future::Future;
use tokio::runtime::Runtime;

static HTTP_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .thread_name("oclive-http")
        .build()
        .expect("oclive http runtime")
});

#[inline]
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    HTTP_RUNTIME.block_on(future)
}
