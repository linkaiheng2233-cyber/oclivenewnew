# {{PLUGIN_NAME}}（Remote HTTP 插件）

由 `oclive plugin create` 生成。插件 id：`{{PLUGIN_ID}}`。

## 下一步

1. 编辑 `rpc_server.py`：实现各 JSON-RPC 方法（见 `REMOTE_PLUGIN_PROTOCOL.md`）。
2. 启动侧车后，在环境或启动器中设置：
   - 通用槽：`OCLIVE_REMOTE_PLUGIN_URL=http://127.0.0.1:8765/rpc`
   - LLM 槽：`OCLIVE_REMOTE_LLM_URL=http://127.0.0.1:8765/rpc`
3. 在角色包 `settings.json` 将对应 `plugin_backends.<slot>` 设为 **`remote`**。

## 本地调试

```bash
python rpc_server.py
```

## 文档

- [REMOTE_PLUGIN_PROTOCOL.md](https://github.com/oclive/oclivenewnew/blob/main/creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)
- [PLUGIN_AUTHOR_LEARNING_PATH.md](https://github.com/oclive/oclivenewnew/blob/main/creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)
