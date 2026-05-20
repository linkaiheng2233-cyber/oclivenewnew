# {{PLUGIN_NAME}}（目录插件）

由 `oclive plugin create` 生成。插件 id：`{{PLUGIN_ID}}`。

## 下一步

1. 编辑 `rpc_server.mjs`：将各 `METHOD` 桩替换为真实逻辑（JSON-RPC 2.0，契约见主仓 `creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md`）。
2. 确认 `manifest.json` 中 `provides`、`rpcMethods`、`process` 与 `permissions` 与实现一致。
3. 将整个目录复制到宿主 **`{app_data}/plugins/{{PLUGIN_ID}}/`**（或内核脚手架项目的 `plugins/`），在角色包 `settings.json` 中将对应槽设为 **`directory`** 并填写 `directory_plugins.<slot>` 为本 manifest 的 **`id`**。

## 本地调试

```bash
cd plugins/{{PLUGIN_ID}}
node rpc_server.mjs
```

就绪后应打印 `OCLIVE_READY http://127.0.0.1:<port>/rpc` 行。

## 文档

- [DIRECTORY_PLUGINS.md](https://github.com/oclive/oclivenewnew/blob/main/creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [PLUGIN_AUTHOR_LEARNING_PATH.md](https://github.com/oclive/oclivenewnew/blob/main/creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)
