# MCP Server 配置（robot-gateway 模板）

智能网关通过 **Agent 槽（builtin）** 调用 MCP 工具，协调智能家居设备。本目录为**项目内占位**，运行时通常需同步到宿主 **`{app_data}/mcp-servers/*.json`**（与 oclivenewnew 桌面版一致）。

## 快速开始

1. 复制 `smart_home.example.json` 为 `smart_home.json`，填写真实 `url` 或 `stdio` 命令。
2. 在角色包 `roles/gateway/settings.json` 中确认 `plugin_backends.agent` 为 **`builtin`**。
3. 启动内核前，将 JSON 放到宿主扫描目录，或保留 `agent_mcp.local_scan_dir` 指向本目录（集成方自定加载逻辑）。
4. 首次调用 **stdio** 传输须用户授权 **`mcp:stdio`**；**http** 须 **`mcp:http`**（见主仓 PLUGIN_V1）。

## 文件说明

| 文件 | 说明 |
|------|------|
| `smart_home.example.json` | HTTP 侧车示例（灯控、传感器等） |
| `README.md` | 本说明 |

## 与 settings.json 的关系

`roles/gateway/settings.json` 内 **`agent_mcp`** 块为脚手架占位，列出建议扫描的 server id；**非** PLUGIN_V1 正式契约字段，宿主可能忽略，仅供网关固件团队对照。

更多见主仓 `creator-docs/plugin-and-architecture/PLUGIN_V1.md` 与 `AGENTS.md`（MCP 目录约定）。
