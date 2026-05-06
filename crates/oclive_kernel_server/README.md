# oclive_kernel_server

Standalone **Oclive Kernel Server** (no Tauri). It exposes:

- `GET /health` → `"ok"`（**不**要求 Bearer；用于探活）
- `GET /oocp` → OOCP WebSocket endpoint（capabilities first frame）
- `POST /chat` → JSON 试聊（`process_message`；与编写器/桌面内核对齐，见 `oclive_kernel_runtime::http_api`）
- `POST/GET /role-feedback` 等 REST（与桌面一致）

## Run

```bash
# 默认监听 127.0.0.1:48888
cargo run -p oclive_kernel_server
```

## 环境变量

| 变量 | 默认 | 说明 |
|------|------|------|
| `OOCP_API_PORT` | `48888` | 端口 |
| `OOCP_API_BIND` | `127.0.0.1` | 监听地址；Docker / 局域网常设为 `0.0.0.0`，须配合鉴权 |
| `OOCP_API_TOKEN` | （空） | 非空时：REST 与 OOCP WS 均需 `Authorization: Bearer <token>` |
| `OCLIVE_ROLES_DIR` | 启发式 | **生产请显式设置** |
| `OCLIVE_DB_PATH` | 临时目录 sqlite | 建议固定路径 |
| `OCLIVE_APP_DATA_DIR` | 派生 | 插件/MCP 等数据根 |
| `RUST_LOG` | — | 如 `info`、`debug` |

Linux 部署、Docker、systemd、多模态外挂约定：**[`docs/LINUX_KERNEL_ENGINE.md`](../../docs/LINUX_KERNEL_ENGINE.md)**  
合成模板目录：**[`delivery/`](../../delivery/)**

## HTTP 试聊脚本示例

- **[`examples/kernel_remote_simple/`](../../examples/kernel_remote_simple/)** — Python / Node，`/health` 与 `/chat`
- **[`examples/linux_kernel_multimodal_context/`](../../examples/linux_kernel_multimodal_context/)** — 将外挂感知拼入 `message` 的示例

## Status

This binary runs the **full kernel runtime** (roles, DB, plugin backends) and exposes it via HTTP + OOCP. 方法参数与结果以 [`creator-docs/oocp/OOCP_SPEC_v0_1.md`](../../creator-docs/oocp/OOCP_SPEC_v0_1.md) 为准。
