# OpenAI 兼容侧车示例（`requests`）

在 **`llm.generate` / `llm.generate_tag`** 中把宿主发来的 `prompt`、`model` 转发到 **OpenAI 兼容** 的 `POST /v1/chat/completions`（或其它基址下的同名路径）。**API Key 由你自行在环境变量中配置**，调用费由账号承担。

协议全文仍见 **[REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**。更通用的用户向说明见 **[SIDECAR_LLM_USER_GUIDE.md](../../creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md)**。

## 依赖

```bash
cd examples/remote_plugin_openai_compat
pip install -r requirements.txt
```

## 配置

1. 复制 **`.env.example`** 为 **`.env`**，填入 **`OPENAI_API_KEY`**（勿提交 `.env`）。  
2. 可选：**`OPENAI_MODEL`**（宿主未传 `model` 时的默认）、**`OPENAI_BASE_URL`**（默认为 `https://api.openai.com/v1`）。  
   - 其它兼容服务（如 OpenRouter、部分国内代理）请查阅其文档，将 `OPENAI_BASE_URL` 设为对应 **`.../v1`** 根路径。  
3. 可选：**`SIDECAR_PORT`**（默认 `8765`）、**`OPENAI_TIMEOUT_SEC`**、**`OPENAI_TEMPERATURE`**。

## 启动

```bash
python server.py
```

默认 **`http://127.0.0.1:8765/rpc`**。在 **oclive 启动器** 的 oclive 页选择 **云端 Remote LLM**，填写该 URL；若侧车将来加鉴权，再在启动器填 **Token**（本示例未校验 Bearer，仅演示）。

## 与「最小示例」的区别

| 项目 | `remote_plugin_minimal` | 本目录 |
|------|-------------------------|--------|
| `llm.*` | 固定占位字符串 | **真实 HTTPS 调用** |
| 依赖 | 仅标准库 + **共用 `../common/`** | `requests`、`python-dotenv`（可选加载 `.env`） |

非 LLM 方法（memory / emotion 等）来自共享模块 **`../common/oclive_stub_handlers.py`**，与最小示例行为一致，便于同一 URL 联调。HTTP/JSON-RPC 骨架见 **[../common/README.md](../common/README.md)**。

## 故障排查

- **`OPENAI_API_KEY is not set`**：未配置密钥或 `.env` 未加载。  
- **`upstream HTTP 4xx/5xx`**：检查 Key、模型名、基址是否与服务商一致。  
- **网络**：访问境外 API 若失败，请在本机配置系统代理或服务商允许的访问方式（与 oclive 无关）。
