# oclive-event-builtin-directory

与 **`oclive_event_builtin`** / 运行时 **`RemoteEventEstimatorHttp`** 协议对齐的最小目录插件：实现 JSON-RPC **`event.estimate`**。

## 运行

```bash
cd examples/oclive-event-builtin-directory
node rpc_server.mjs
```

默认监听 `http://127.0.0.1:8791/rpc`（可用环境变量 `OCLIVE_EVENT_DIR_PORT` 修改）。

## 角色包

- 将 `plugin_backends.event` 设为 **`directory`**，`directory_plugins.event` 指向本 manifest 的 **`id`**：`com.oclive.builtin.event`。
- 需授予 **`process:spawn`**（Node 拉起侧车）。

返回体为简化 `EventImpactEstimate`：`event_type` 使用字符串枚举名（如 `Ignore`），与 serde 默认枚举编码一致。
