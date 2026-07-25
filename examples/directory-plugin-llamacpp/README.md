# 目录插件示例：LLM 槽对接 llama.cpp（`com.oclive.example.llamacpp_llm`）

[English](README.en.md)

本示例把 **`plugin_backends.llm = directory`** 时宿主下发的 **`llm.generate` / `llm.generate_tag`**（JSON-RPC，契约见 [REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §4.6）转发到你本机已启动的 **llama.cpp HTTP server**，从而在 **不经过 Ollama** 的情况下完成主对话与低温度标签类调用。

## 依赖

- **Node.js 18+**（内置 `fetch`），用于 `rpc_server.mjs`。
- 自行编译或下载的 **llama.cpp**，并启动带 HTTP 的服务进程（见下）。宿主 **不** 内置 llama.cpp 二进制。

## 启动 llama.cpp server（示例）

以下命令随你本机 `llama-server` 路径与模型路径调整；端口需与 `OCLIVE_LLAMACPP_SERVER_URL` 一致（默认 `http://127.0.0.1:8080`）。

```bash
# 常见：OpenAI 兼容 HTTP（本插件优先走 /v1/chat/completions）
llama-server -m /path/to/model.gguf --host 127.0.0.1 --port 8080
```

若你的构建 **没有** `/v1/chat/completions`，插件会尝试回退 **`POST /completion`**（`prompt` + `n_predict`）；仍失败时请对照该构建的 HTTP 文档改 `rpc_server.mjs` 或换用带 OpenAI 兼容层的 `llama-server` 版本。

## 安装插件到宿主

将本目录整体复制为：

`<roles 父目录>/plugins/com.oclive.example.llamacpp_llm/`

（与仓库内 `roles/` 同级时，即 `plugins/com.oclive.example.llamacpp_llm/`。）

或使用 **开发者模式** 的 `extra_plugin_roots`（见 [DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) §1）。

## 高危授权

`manifest.json` 声明了 **`process:spawn`**（由宿主拉起 Node）与 **`network:*`**（Node 进程访问本机 llama HTTP）。首次启用前需在应用内完成 **高风险能力授权**（见 [PLUGIN_V1.md](../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) 权限规范、[DIRECTORY_PLUGINS.md](../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md) §2）。

自动化环境可设 **`OCLIVE_SKIP_HIGH_RISK_GRANTS=1`**（勿用于面向用户的发行场景）。

## 角色包 `settings.json` 示例

在对应角色的 **`settings.json`** 中合并（`directory_plugins.llm` 的 id 须与 manifest 中 **`id`** 一致）：

```json
{
  "plugin_backends": {
    "memory": "builtin",
    "emotion": "builtin",
    "event": "builtin",
    "prompt": "builtin",
    "llm": "directory",
    "agent": "builtin",
    "directory_plugins": {
      "llm": "com.oclive.example.llamacpp_llm"
    }
  }
}
```

宿主内 **`effective_ollama_model`** 仍会作为 **`model` 字符串**传给本插件，再原样传给 llama-server；若上游忽略该字段，可忽略；若需固定槽位名，可在 `rpc_server.mjs` 内改写映射。

## 环境变量

| 变量 | 说明 |
|------|------|
| **`OCLIVE_LLAMACPP_SERVER_URL`** | llama.cpp HTTP 根地址，默认 **`http://127.0.0.1:8080`**（无尾部 `/` 亦可）。 |

该变量由 **插件子进程（Node）** 读取；可在系统环境或启动 oclive 前的 shell 中导出。

## 与 Ollama 并存

- 未把 `llm` 设为 **`directory`** 的角色仍走默认 **`ollama`**。
- 仅将需要实验 llama.cpp 的角色包改为上表配置即可 **按角色切换**，无需改宿主 Rust。

## 文件说明

| 文件 | 作用 |
|------|------|
| `manifest.json` | 插件 id、`provides: ["llm"]`、子进程、`permissions` |
| `rpc_server.mjs` | JSON-RPC 入口，转发至 llama.cpp HTTP |

## 排障

1. 确认 **`OCLIVE_READY http://...`** 出现在插件进程 stdout（宿主才能握手）。  
2. 在浏览器或 `curl` 直接访问 llama 根地址，确认 **未** 返回连接拒绝。  
3. 看宿主日志中 **`remote_llm`** / **`oclive_plugin`** 相关行；JSON-RPC 返回 **`llamacpp proxy:`** 前缀多为上游 HTTP 非 2xx 或 JSON 形状不符。  
4. 若长期大包传输不稳定，可考虑改为 **`plugin_backends.llm = remote`**，把本逻辑挪到独立常驻侧车（同一协议），减少子进程启停。

## 作为 LoRA 专家插件使用

本插件也可承载已经由 llama.cpp、vLLM 或其它 OpenAI-compatible 服务注册的 LoRA
模型别名。插件配置中的 `adapter_model` 非空时，它会覆盖宿主传入的基础模型名；
权重加载、显存管理和 adapter 热切换仍由推理服务负责。

角色蓝图必须把插件预声明为独立的 `llm + directory` 实例：

```json
{
  "slot_registry": {
    "llm": {
      "type": "llm",
      "label": "Default LLM",
      "backend": "ollama",
      "position": 0
    },
    "mumu_lora": {
      "type": "llm",
      "label": "Mumu LoRA",
      "backend": "directory",
      "position": 10,
      "plugin": "com.oclive.example.llamacpp_llm",
      "zone": "experimental"
    }
  }
}
```

专家路由命中时选择同一个插件 ID：

```json
{
  "action": "slot.lora.apply",
  "params": {
    "plugin_id": "com.oclive.example.llamacpp_llm"
  }
}
```

运行要求：

- 宿主使用 `dual_core` feature，角色为 blueprint v3 且启用
  `runtime_config.dual_core.enabled`。
- `pipeline.experimental` 包含 `slot.expert.invoke` 和最终的
  `slot.<llm-key>.generate`。
- 插件必须声明 `provides: ["llm"]`，并获得 `process:spawn` /
  `network:*` 授权。
- `adapter_model` 应是推理服务已经注册的 LoRA 模型别名。内核不直接解释
  `.safetensors`，因此不会把某个框架的私有加载协议写死在角色包中。
- 插件把 OpenAI-compatible SSE 转换为 OCLive `llm.generate_stream` NDJSON；
  manifest 已显式声明该能力，因此双核 Stable completion 会逐 token 转发。上游不支持
  SSE 时插件兼容回退为整段生成并单次回调。
- LoRA 选择按会话保存；配置失效、插件不可用或生成失败时，宿主记录
  `LORA_ADAPTER_INVALID` / `LORA_ADAPTER_UNAVAILABLE` /
  `LORA_ADAPTER_GENERATE_FAILED` 并回退普通 LLM；流式已输出后失败则记录
  `LORA_ADAPTER_STREAM_PARTIAL` 并保留部分回复，避免拼接第二份回复。
