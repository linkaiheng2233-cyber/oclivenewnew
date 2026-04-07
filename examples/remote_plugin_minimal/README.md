# 最小 JSON-RPC 侧车示例（联调用）

用于验证宿主 [`OCLIVE_REMOTE_PLUGIN_URL`](../../creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) / [`OCLIVE_REMOTE_LLM_URL`](../../creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) 与 **[REMOTE_PLUGIN_PROTOCOL.md](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)**（**完整请求/响应 JSON 以协议文档为准**）的对接。

**非生产级**：无鉴权加固、无并发压测，仅演示请求形状与占位返回。

**全库文档索引**：[../../creator-docs/getting-started/DOCUMENTATION_INDEX.md](../../creator-docs/getting-started/DOCUMENTATION_INDEX.md)

## 依赖

- Python 3.9+（仅用标准库）

## 启动

```bash
cd examples/remote_plugin_minimal
python server.py
```

默认监听 `http://127.0.0.1:8765/rpc`（单一路径 POST JSON-RPC）。

## 与 oclive 联调

在启动桌面应用**之前**设置环境变量（PowerShell 示例）：

```powershell
$env:OCLIVE_REMOTE_PLUGIN_URL = "http://127.0.0.1:8765/rpc"
$env:OCLIVE_REMOTE_LLM_URL = "http://127.0.0.1:8765/rpc"
```

角色包 `settings.json` 中把需要走侧车的项设为 `remote`（见 [CREATOR_PLUGIN_ARCHITECTURE.md](../../creator-docs/plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)）。

## 实现说明

`server.py` 对每个方法返回**最小合法**占位数据；你可逐步替换为真实模型调用或业务逻辑。
