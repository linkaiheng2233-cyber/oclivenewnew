# oclive_kernel_runtime

可嵌入的 Oclive **内核运行时**：会话编排、`KernelAppState`、SQLite 持久化、目录插件/远程插件协议面、市场索引与插件归档处理等。官方桌面（`src-tauri`）与无头 [`oclive_kernel_server`](../oclive_kernel_server/) 均依赖本 crate。

## Features

| Feature | 默认 | 说明 |
|--------|------|------|
| `full` | ✅ | 聚合当前发行版所需能力（含 `kernel-http-api`）。 |
| `kernel-http-api` | ✅（经 `full`） | Axum 本地 HTTP（`/health`、`/chat`）与 OOCP WebSocket；关闭后不编译 `http_api` 与内核 `adapters::oocp_ws`，可减小依赖树。 |
| `tauri_invoke` | ❌ | 由桌面 crate 启用：`AppError` → `tauri::InvokeError`。 |

**极简嵌入示例**（不需要内置 HTTP 服务器时）：

```toml
oclive_kernel_runtime = { path = "../crates/oclive_kernel_runtime", default-features = false }
```

`oclive_kernel_server` 需要 HTTP/WS，请保持默认 `full` 或显式 `features = ["kernel-http-api"]`。

## 文档与契约

- 内核边界：[creator-docs/kernel/KERNEL_BOUNDARY.md](../../creator-docs/kernel/KERNEL_BOUNDARY.md)
- Baseline：[creator-docs/kernel/KERNEL_BASELINE_V1.md](../../creator-docs/kernel/KERNEL_BASELINE_V1.md)
- Tauri 命令清单 ↔ 实现对照：[creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md](../../creator-docs/kernel/KERNEL_API_IMPLEMENTATION_MATRIX.md)
- OOCP：`creator-docs/oocp/OOCP_SPEC_v0_1.md`

## 开发

```bash
# 自仓库根目录
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p oclive_kernel_runtime
cargo check -p oclive_kernel_runtime --no-default-features   # 无 Axum 栈
```

## crates.io / SDK 路线

发布前请在根 `Cargo.toml` 中补齐 `repository` / `homepage` 等与.mono 仓库一致的元数据；本 crate 的 `keywords` / `categories` / `readme` 已按 Rust 惯例预留。

## I/O 与异步（路线图）

内核内仍存在若干 `reqwest::blocking` 路径（远程插件 JSON-RPC、市场索引同步、MCP HTTP 等）。在保持 Tauri 同步命令语义的前提下，将逐步改为 `reqwest::Client` + `tokio`，或在宿主侧用 `spawn_blocking` 隔离；不属于单次 PR 的破坏性变更。
