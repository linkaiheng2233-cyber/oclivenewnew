# 本机侧车 + 云端闭源模型（用户向）

本文说明：**不依赖 oclive 官方托管服务器**，在用户自己电脑上跑一个 **HTTP 侧车**，把对话请求转成各家 **闭源 / 云端 API**（OpenAI 兼容或其它 HTTPS 接口），**API 调用费由用户自行承担**（自带 Key，BYOK）。

**协议细节**仍以 **[REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)** 为准；最小可运行示例见仓库 **[examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md)**。

---

## 1. 为什么需要侧车？

oclive 宿主向远程地址发的是 **JSON-RPC 2.0**（例如 `llm.generate`），**不是**直接把 `https://api.openai.com/v1/chat/completions` 填进配置就能用。  
因此需要一段 **适配层**：对外实现协议里的方法，对内用你自己的 Key 去调厂商 API。这段程序就叫 **侧车**（可跑在本机 `127.0.0.1`，也可将来部署到你的服务器给「小白云端」用）。

---

## 2. 你需要准备什么

| 项目 | 说明 |
|------|------|
| **Python 3.9+**（或其它语言） | 跑侧车；最小示例仅标准库。 |
| **云端模型的 API Key** | 在对应平台申请；**不要**写进角色包或公开仓库。 |
| **oclive / 启动器** | 启动器可为 oclive 注入 **`OCLIVE_REMOTE_LLM_URL`** 等环境变量（见下）。 |

**网络**：侧车若访问 **境外** API，是否需代理取决于你的网络环境；侧车与 oclive 同机时，本机回环 `127.0.0.1` **不需要** VPN。

---

## 3. 三步走（本机）

### 步骤 A：跑通最小侧车

```bash
cd examples/remote_plugin_minimal
python server.py
```

默认监听 **`http://127.0.0.1:8765/rpc`**。此时 `llm.generate` 返回占位字符串，用于确认宿主与侧车 **HTTP / JSON-RPC 链路**正常。

### 步骤 B：在角色包里把 LLM 设为 remote

在对应角色的 **`settings.json`**（或编写器里等价配置）中，将 **`plugin_backends.llm`** 设为 **`"remote"`**（与 [PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md) 一致）。

未设置环境变量时，宿主可能回退内置并打警告；因此必须配置下一步。

### 步骤 C：告诉 oclive 侧车地址（推荐：oclive 启动器）

使用 **[oclive-launcher](https://github.com/linkaiheng2233-cyber/oclive-launcher)** 时，在 **「启动 oclive」** 页：

1. 将 **推理后端（大脑）** 选为 **云端接口（remote）**。  
2. 填写 **Remote LLM URL** 为侧车完整地址，例如 **`http://127.0.0.1:8765/rpc`**。  
3. 若侧车要求 Bearer，填写 **Token**（与协议中 `Authorization: Bearer` 一致）；可选超时。  
4. 点击 **保存配置**，再启动 oclive。

启动器会把 **`OCLIVE_LLM_BACKEND=remote`**、**`OCLIVE_REMOTE_LLM_URL`** 等注入子进程（具体键名以启动器 README 与 **[CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)** 为准）。

**仅命令行调试**（不用启动器）时，可在启动 oclive **前**在 shell 里设置与 **[examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md)** 相同的变量。

---

## 4. 接上真实闭源模型

最小示例里 **`llm.generate`** / **`llm.generate_tag`** 返回固定占位符。接入真实 API 有两种常见方式：

### 4.1 直接使用现成范例（推荐）

仓库 **[examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md)**：用 **`requests`** 调用 **OpenAI 兼容** 的 `chat/completions`，支持 **`OPENAI_API_KEY`**、**`OPENAI_BASE_URL`**、**`OPENAI_MODEL`**（详见该目录 **`.env.example`**）。安装依赖后 `python server.py`，再把启动器里的 Remote LLM URL 指到 `http://127.0.0.1:8765/rpc`（端口以控制台输出为准）。

### 4.2 自己改侧车代码

在侧车内：

1. 读取 JSON-RPC **`params.prompt`**、**`params.model`**（见 REMOTE_PLUGIN_PROTOCOL **§4.6**）。  
2. 用 HTTPS 调用厂商接口（请求头带上 **你的 API Key**）。  
3. 将模型返回的正文填入 JSON-RPC **`result`**：`{"text": "..."}` 或协议允许的等价形式。

**安全**：Key 建议只用环境变量或本机配置文件读取，**不要**提交到 Git。

---

## 5. 与「将来官方云端」的关系

当前推荐路径：**开源侧车示例 + 用户本机 BYOK**。  
若日后提供 **托管侧车 URL**，用户只需把地址换成你的域名，协议不变；**调用费**仍可通过「用户自带 Key」或「套餐计费」等产品策略区分。

---

## 6. 相关链接

| 文档 | 内容 |
|------|------|
| [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) | 方法名、`params`/`result`、请求头 |
| [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) | 环境变量总览、与角色包关系 |
| [examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md) | 最小 Python 侧车启动与联调命令 |
| [examples/remote_plugin_openai_compat/README.md](../../examples/remote_plugin_openai_compat/README.md) | **OpenAI 兼容 API**（`requests` + 环境变量 Key） |
