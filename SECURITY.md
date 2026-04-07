# 安全说明

## 报告漏洞

若你发现安全漏洞，请**不要**在公开 issue 中披露细节。请通过以下方式联系维护者（请替换为实际邮箱或 GitHub Security 启用后的入口）：

- 优先：仓库 **Security** → **Report a vulnerability**（若已启用 GitHub 私有报告）
- 或：向维护者发送邮件（标题注明 `[oclive-security]`）

请尽量包含：复现步骤、影响范围、版本 / 提交哈希。

## 设计说明（本地应用）

- oclive 以**本地**数据与 **Ollama** 为主；请勿在配置中硬编码 API 密钥。
- **Remote** 类插件后端接入 HTTP 侧车时请注意网络暴露面与鉴权（见 [creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)）。
