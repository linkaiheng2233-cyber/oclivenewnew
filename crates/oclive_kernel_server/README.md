# oclive_kernel_server

Standalone **Oclive Kernel Server** (no Tauri). It exposes:

- `GET /health` → `"ok"`
- `GET /oocp` → OOCP WebSocket endpoint (capabilities first frame)
- `POST /chat` → JSON 试聊（`process_message`；与编写器/工具链对齐，见 `oclive_kernel_runtime::http_api`）

## Run

```bash
# default: 127.0.0.1:48888
cargo run -p oclive_kernel_server
```

Optional env:

- `OOCP_API_PORT`: listening port (default `48888`)
- `OOCP_API_TOKEN`: enable Bearer auth (optional)
- `OCLIVE_DB_PATH`: sqlite db path (optional; default under app data dir)
- `OCLIVE_ROLES_DIR`: roles root dir (optional)
- `OCLIVE_APP_DATA_DIR`: app data dir (optional)

## HTTP 试聊脚本示例

仓库 **[`examples/kernel_remote_simple/`](../../examples/kernel_remote_simple/)** 提供 Python / Node 客户端，演示 **`GET /health`** 与 **`POST /chat`**（需本机 Ollama 与合法 `role_path`）。

## Status

This binary runs the **full kernel runtime** (roles, DB, plugin backends) and exposes it via OOCP. 方法参数与结果以仓库内 [`creator-docs/oocp/OOCP_SPEC_v0_1.md`](../../creator-docs/oocp/OOCP_SPEC_v0_1.md) 为准（例如 `time.get_state` 需带 `session_ns`）。

