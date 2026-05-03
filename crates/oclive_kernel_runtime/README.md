# oclive_kernel_runtime

可嵌入的 Oclive **内核运行时**：会话编排、`KernelAppState`、SQLite 持久化、目录插件/远程插件协议面、**角色包（`role_pack_archive`）**、**角色/插件市场索引与评价索引同步**、**插件安装（`plugin_install`）**、插件归档与验签等。官方桌面（`src-tauri`）与无头 [`oclive_kernel_server`](../oclive_kernel_server/) 均依赖本 crate。

## Features

| Feature | 默认 | 说明 |
|--------|------|------|
| `full` | ✅ | `kernel-http-api` + `role-pack-zip` + `market-sync` + `kernel-agent`（与官方桌面 / kernel_server 一致）。 |
| `kernel-http-api` | ✅（经 `full`） | Axum 本地 HTTP 与 OOCP WebSocket；关闭后不编译 `http_api`。 |
| `role-pack-zip` | ✅（经 `full`） | `zip` 与角色包 / 插件归档路径（`plugin_archive`、`role_pack_archive`）。 |
| `market-sync` | ✅（经 `full`） | 角色 / 插件市场与评价索引同步模块。 |
| `kernel-agent` | ✅（经 `full`） | MCP、Builtin ReAct Agent、远程 / 目录 Agent HTTP。 |
| `tauri_invoke` | ❌ | 由桌面 crate 启用：`AppError` → `tauri::InvokeError`。 |

**极简嵌入示例**（不需要内置 HTTP 服务器时）：

```toml
oclive_kernel_runtime = { path = "../crates/oclive_kernel_runtime", default-features = false }
```

详细矩阵见 [creator-docs/kernel/LIGHTWEIGHT_PROFILE.md](../../creator-docs/kernel/LIGHTWEIGHT_PROFILE.md)。

`oclive_kernel_server` 需要 HTTP/WS，请保持默认 `full` 或显式 `features = ["kernel-http-api"]`。

## 文档与契约

- 内核边界：[creator-docs/kernel/KERNEL_BOUNDARY.md](../../creator-docs/kernel/KERNEL_BOUNDARY.md)
- Baseline：[creator-docs/kernel/KERNEL_BASELINE_V1.md](../../creator-docs/kernel/KERNEL_BASELINE_V1.md)
- Tauri 命令清单 ↔ 实现对照：[creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md](../../creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md)
- 迁入收尾：[handoff/KERNEL_MIGRATION_COMPLETE.md](../../handoff/KERNEL_MIGRATION_COMPLETE.md)
- OOCP：`creator-docs/oocp/OOCP_SPEC_v0_1.md`

## 开发

```bash
# 自仓库根目录
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p oclive_kernel_runtime
cargo check -p oclive_kernel_runtime --no-default-features   # 无 Axum / zip / 市场同步 / Agent 栈（按需再开子 feature）
```

## crates.io / SDK 路线

发布前请在根 `Cargo.toml` 中补齐 `repository` / `homepage` 等与.mono 仓库一致的元数据；本 crate 的 `keywords` / `categories` / `readme` 已按 Rust 惯例预留。

## I/O 与异步（路线图）

Workspace **`reqwest` 已不启用 `blocking`**。仍对外暴露同步签名的 HTTP 入口（市场索引同步、目录插件 JSON-RPC、MCP HTTP、部分 `remote_plugin` trait 等）在实现内使用 **`reqwest::Client` + `.await`**，并由 **`infrastructure::blocking_http`**（专用 Tokio runtime 上的 `block_on`）桥接，以保持 Tauri 同步 `invoke` 等契约；纯 async 路径（如 `jsonrpc::call_async`、`RemoteLlmHttp`）直接 `.await` 即可。
