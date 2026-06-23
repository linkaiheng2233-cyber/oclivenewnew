# 目录插件安装位置

与角色包目录 **`roles/`** 同级：在本项目根下创建 **`plugins/<manifest.id>/`**（例如 `plugins/com.example.myplugin/manifest.json`）。

宿主还会扫描（若存在）：

- **`{app_data}/plugins/`**（应用数据目录）
- **`./plugins/`**（进程当前工作目录）

开发者模式可配置额外根目录，见主仓文档。

## 三种扩展方式（简要）

| 方式 | 配置 | 适用 |
|------|------|------|
| **目录插件** | `plugin_backends.* = directory` + `directory_plugins.<slot>` | 本机子进程 + JSON-RPC，可带 UI 插槽 |
| **Remote 侧车** | `plugin_backends.* = remote` + `OCLIVE_REMOTE_*` | HTTP JSON-RPC，独立服务 |
| **MCP（Agent）** | `plugin_backends.agent` + `{app_data}/mcp-servers/*.json` | 工具调用；须用户授权 |

## 文档

- 契约与模块编号：[PLUGIN_V1.md](../../../creator-docs/plugin-and-architecture/PLUGIN_V1.md) · [OCLIVE_ARCHITECTURE_OVERVIEW.md](../../../creator-docs/getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)
- 插件作者学习路径：[PLUGIN_AUTHOR_LEARNING_PATH.md](../../../creator-docs/plugin-and-architecture/PLUGIN_AUTHOR_LEARNING_PATH.md)
- 目录插件详解：[DIRECTORY_PLUGINS.md](../../../creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)

## 示例插件（可选）

`oclive init --with-example-plugin` 会复制 **`com.oclive.example.llamacpp_llm/`**（源自主仓 `examples/directory-plugin-llamacpp/`），演示 **llm** 槽目录插件 + JSON-RPC。

未加该参数时，本目录仅含本 README；也可手动从主仓 `examples/directory-plugin-minimal/` 或 `examples/directory-plugin-llamacpp/` 复制。
