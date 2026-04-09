# 侧车示例共用代码

| 模块 | 职责 |
|------|------|
| `jsonrpc_http.py` | `rpc_result` / `rpc_error`、HTTP POST 单路径分发、`run_server` |
| `oclive_stub_handlers.py` | 非 LLM 的 JSON-RPC 占位实现（memory / emotion / event / prompt） |

`remote_plugin_minimal` 与 `remote_plugin_openai_compat` 通过 `sys.path` 引用本目录，避免两份重复的 JSON-RPC 与占位逻辑。
