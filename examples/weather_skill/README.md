# weather_skill (MCP 示例)

最小天气 Skill（Node.js），提供 `get_weather(city)` 工具，便于验证 oclive Agent → MCP 调度链路。

## 运行

```bash
npm install
npm start
```

默认监听 `http://127.0.0.1:3456/mcp`。

## MCP server manifest（放到 app_data/mcp-servers/）

示例见同目录 `mcp.server.weather-demo.json`，可直接复制到：

- Windows: `%APPDATA%/../Local/<app>/mcp-servers/`（以实际 app_data_dir 为准）

## 约定

请求：

```json
{ "tool": "get_weather", "params": { "city": "深圳" } }
```

响应：

```json
{
  "result": {
    "city": "深圳",
    "temperature_c": 29,
    "condition": "多云",
    "summary": "深圳当前多云，气温 29°C。"
  }
}
```
