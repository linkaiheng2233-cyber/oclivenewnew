# 安全说明

## 分发与隐私（速览三问）

1. **第三方模型与 API**：默认 **本地 Ollama**；若使用云端或 HTTP 侧车，密钥与出站网络由 **用户与侧车** 配置负责 — 见 [SIDECAR_LLM_USER_GUIDE.md](creator-docs/getting-started/SIDECAR_LLM_USER_GUIDE.md)。  
2. **插件与权限**：目录插件 / MCP 须遵守 **manifest** 与宿主授权 — 见 [LICENSE_POLICY.md](creator-docs/LICENSE_POLICY.md)、[DIRECTORY_PLUGINS.md](creator-docs/plugin-and-architecture/DIRECTORY_PLUGINS.md)。  
3. **用户数据落盘**：SQLite 与 `{app_data}` 路径见 [CONFIGURATION_FILES.md](creator-docs/guides/CONFIGURATION_FILES.md)；漏洞报告请勿粘贴含隐私的完整路径或密钥。

## 报告漏洞

若你发现安全漏洞，请**不要**在公开 issue 中披露细节。请通过以下方式联系维护者（请替换为实际邮箱或 GitHub Security 启用后的入口）：

- 优先：仓库 **Security** → **Report a vulnerability**（若已启用 GitHub 私有报告）
- 或：向维护者发送邮件（标题注明 `[oclive-security]`）

请尽量包含：复现步骤、影响范围、版本 / 提交哈希。

## 设计说明（本地应用）

- oclive 以**本地**数据与 **Ollama** 为主；请勿在配置中硬编码 API 密钥。
- **Remote** 类插件后端接入 HTTP 侧车时请注意网络暴露面与鉴权（见 [creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)）。
