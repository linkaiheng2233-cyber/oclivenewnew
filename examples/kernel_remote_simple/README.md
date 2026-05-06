# kernel_remote_simple — HTTP 调用内核 `/health` 与 `/chat`

演示 **无桌面壳** 的 **`oclive_kernel_server`** 试聊接口，与编写器 / 根目录 [README.md](../../README.md) 中 `POST /chat` 契约一致。

## 前置条件

1. **Rust**：本仓库已 `git clone` 且能 `cargo build -p oclive_kernel_server`。
2. **Ollama**：本机已运行 Ollama，并已 `ollama pull` 与角色 `settings.json` 中 **`model`** 一致的模型（示例角色 `roles/mumu` 默认为 `qwen2.5:7b`）。
3. **Python 3.8+** 和/或 **Node.js 18+**（二选一运行客户端脚本即可）。

### Linux 生产式启动（推荐与文档对齐）

与 **[`docs/LINUX_KERNEL_DEPLOY.md`](../../docs/LINUX_KERNEL_DEPLOY.md)** 一致，显式设置路径并打开严格模式，避免数据库落到临时目录：

```bash
export OCLIVE_ROLES_DIR=/绝对路径/oclivenewnew/roles
export OCLIVE_DB_PATH=/tmp/oclive-kernel-remote-demo.db
export OCLIVE_APP_DATA_DIR=/tmp/oclive-kernel-remote-demo-app
export OCLIVE_REQUIRE_EXPLICIT_PATHS=1
export OOCP_API_BIND=127.0.0.1
# 若已设置 OOCP_API_TOKEN，客户端需加 --header 或 CURL_OPTS；/health 仍无需鉴权

cargo run -p oclive_kernel_server
```

预期：终端出现 `HTTP API listening http://127.0.0.1:48888`（端口以 `OOCP_API_PORT` 为准）。另开终端执行下文第 2 节客户端；**`--role-path`** 须与 **`OCLIVE_ROLES_DIR`** 下某一角色目录一致（例如 `.../roles/mumu`）。

## 1. 启动内核服务

在**仓库根目录**执行：

```bash
cargo run -p oclive_kernel_server
```

默认监听 **`127.0.0.1:48888`**。若未设三路径，内核会用开发向默认（临时 DB 等）；**Linux 长期运行请务必按上一节设置**。改端口：

```bash
# Linux / macOS
export OOCP_API_PORT=48888
export OCLIVE_ROLES_DIR="/绝对路径/oclivenewnew/roles"
cargo run -p oclive_kernel_server
```

```powershell
# Windows PowerShell
$env:OOCP_API_PORT = "48888"
$env:OCLIVE_ROLES_DIR = "D:\oclivenewnew\roles"
cargo run -p oclive_kernel_server
```

说明见 [`crates/oclive_kernel_server/README.md`](../../crates/oclive_kernel_server/README.md)。

## 2. 运行示例客户端

将 **`--role-path`** 换成你本机**含 `manifest.json` 的角色目录绝对路径**（勿用相对路径，避免工作目录不一致）。

**Python（无 pip 依赖）：**

```bash
python examples/kernel_remote_simple/client.py --role-path "/绝对路径/oclivenewnew/roles/mumu"
```

**Node：**

```bash
node examples/kernel_remote_simple/client.mjs --role-path "D:/oclivenewnew/roles/mumu"
```

常用参数：

| 参数 | 含义 |
|------|------|
| `--base-url` | 默认 `http://127.0.0.1:48888` |
| `--message` | 用户消息 |
| `--timeout` | HTTP 超时秒数（默认 120；LLM 慢时可加大） |
| `--session-id` / `--scene-id` | 可选，与 HTTP API 一致 |

## 3. 预期行为

1. 先 **`GET /health`**，打印 `ok`。  
2. 再 **`POST /chat`**，打印模型返回的 **`reply`** 字段。

## 4. 常见错误

| 现象 | 处理 |
|------|------|
| 连接被拒绝 | 确认 `oclive_kernel_server` 已启动且端口与 `--base-url` 一致 |
| `超时` | 增大 `--timeout`；检查 Ollama 是否响应缓慢或未启动 |
| HTTP 400 `load_role_failed` / `invalid_role_path` | 检查 `--role-path` 是否为**目录**且内含合法 `manifest.json` |
| HTTP 500 `chat_engine_failed` | 查看终端日志；多为 LLM / 模型名 / 网络问题 |

## 5. 与嵌入式示例的关系

- **本示例**：独立进程 **HTTP 服务** + 脚本客户端。  
- **`kernel_embed_minimal`**：同进程嵌入 `KernelAppState`，无 HTTP。  

二者互补，见 [KERNEL_SDK.md](../../creator-docs/kernel/KERNEL_SDK.md)。
