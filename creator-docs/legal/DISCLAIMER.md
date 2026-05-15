# 免责声明（模型、插件与数据）

本文档说明 **oclive 桌面宿主**在**模型权重**、**第三方插件**与**用户数据落盘**方面的责任边界，便于用户与集成方快速回答三问：**数据在哪**、**插件安全谁负责**、**模型许可证谁遵守**。不构成法律意见；具体合规请咨询法律顾问。

---

## 1. 模型与推理服务

- **oclive 不提供、不托管、不重新分发任何大模型权重**。用户若使用本地或云端模型，须**自行取得**相应权重与 API 的授权，并遵守其许可证（含商用限制、归属要求等）。  
- **内置集成路径**包含对本机 **Ollama** 的默认调用约定（环境变量、端口、模型名等），但 **Ollama 软件及其模型库并非本仓库的一部分**；其许可与更新由 **Ollama 项目**与**用户本机安装**负责。  
- 若用户配置 **Remote** LLM、HTTP 侧车或其它云端端点，**出站流量、密钥与上游条款**由**用户与侧车运营方**负责 — 参见 [SIDECAR_LLM_USER_GUIDE.md](../getting-started/SIDECAR_LLM_USER_GUIDE.md)、[LICENSE_POLICY.md](../LICENSE_POLICY.md)。

---

## 2. 插件与扩展生态

- **插件市场 / 索引 / 社区源**体现的是**开放生态**：上架或列出的插件由**各自作者**对其代码、更新与声明负责；**oclive 维护者不保证**第三方插件无缺陷、无恶意或与你的环境兼容。  
- **安装前**：请审阅插件包内的 **`manifest.json`**（权限、进程、网络声明等）。  
- **高风险能力**（如 **`process:spawn`**、**`network:*`** 及 MCP 传输类型等）在宿主中走**显式授权**流程；未授权时功能会降级并给出可见提示 — 详见 [DIRECTORY_PLUGINS.md](../plugin-and-architecture/DIRECTORY_PLUGINS.md)、[LICENSE_POLICY.md](../LICENSE_POLICY.md) 及应用内 **Agent 调试 / 授权** 相关说明。

---

## 3. 数据存储与遥测

- **默认本地存储**：对话记录、角色包内容、记忆与运行时状态等主要落在**本机** **SQLite** 与 **`{app_data}`** 目录（路径见 [CONFIGURATION_FILES.md](../guides/CONFIGURATION_FILES.md)）。**在默认配置下，宿主不把上述内容上传到 oclive 运营方控制的云端。**  
- **例外**：若用户在角色包或设置中将某槽配置为 **Remote** HTTP 后端、或启用会主动出站的目录插件 / MCP，则数据可能经**用户配置的 URL** 离开本机 — 由**用户选择的网络对端**负责。  
- **Sentry**：仅当构建时注入 DSN 且用户未在设置中退出时，可能上报 **Vue 侧未捕获异常**（默认不采集可识别个人信息；详见 README 与设置页说明）；**用户可在设置中关闭**。

---

## 相关链接

- [LICENSE](../../LICENSE) · [LICENSE_POLICY.md](../LICENSE_POLICY.md)  
- [SECURITY.md](../../SECURITY.md) · [SECURITY_AUDIT_SCOPE.md](../security/SECURITY_AUDIT_SCOPE.md)

[English](../../creator-docs-en/legal/DISCLAIMER.md)
