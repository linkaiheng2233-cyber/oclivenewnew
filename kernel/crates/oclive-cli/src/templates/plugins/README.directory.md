# {{PLUGIN_NAME}}（目录插件）

由 `oclive plugin create` 生成。插件 id：`{{PLUGIN_ID}}`。

## 下一步

1. 编辑 `rpc_server.mjs`：将各 `METHOD` 桩替换为真实逻辑（JSON-RPC 2.0，契约见主仓 `creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md`）。
2. 确认 `manifest.json` 中 `provides`、`rpcMethods`、`process` 与 `permissions` 与实现一致。
3. 将整个目录复制到宿主 **`{app_data}/plugins/{{PLUGIN_ID}}/`**（或内核脚手架项目的 `plugins/`）。当前蓝图角色包在 `pipeline.ocblueprint.slot_registry` 中把对应实例设为 **`backend: directory`**，并将 `plugin` 设为本 manifest 的 **`id`**；仅 legacy / 占位脚手架 `settings.json` 使用 `directory_plugins.<slot>`。

## 本地调试

```bash
cd plugins/{{PLUGIN_ID}}
node rpc_server.mjs
```

就绪后应打印 `OCLIVE_READY http://127.0.0.1:<port>/rpc` 行。

## 文档

- [DIRECTORY_PLUGINS.md](https://github.com/oclive/oclivenewnew/blob/main/creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [PLUGIN_AUTHOR_LEARNING_PATH.md](https://github.com/oclive/oclivenewnew/blob/main/creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)
