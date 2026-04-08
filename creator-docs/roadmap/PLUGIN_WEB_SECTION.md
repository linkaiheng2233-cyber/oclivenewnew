# 插件区（网站）— 信息架构与内容清单

本文供 **社区站** 中 **「插件」板块** 落地使用：与 [COMMUNITY_WEB_VISION.md](./COMMUNITY_WEB_VISION.md) 第三节一致，强调 **Remote HTTP 侧车** 的发现与文档，**不是**在网页里安装宿主 `.dll` 或 exe。

**权威协议**：[REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)  
**创作者总览**：[CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md)  
**settings 枚举**：[PLUGIN_V1.md](../plugin-and-architecture/PLUGIN_V1.md)

---

## 1. 定位（对用户一句话）

**「在这里查：侧车是什么、环境变量怎么填、`plugin_backends` 怎么设；下载与源码在作者的 GitHub / Release。」**

---

## 2. 建议页面结构（路由示例）

| 路径 | 内容 |
|------|------|
| `/plugins` 或 `/plugins/` | **列表页**：卡片 = 插件/侧车项目（名称、简介、兼容 oclive 版本、作者、**外链 GitHub**、可选标签） |
| `/plugins/how-it-works` | **科普页**：Remote 与 builtin 区别；`OCLIVE_REMOTE_PLUGIN_URL` / `OCLIVE_REMOTE_LLM_URL` 表格（可链到架构文档） |
| `/plugins/examples` | **最小示例**：链到仓库 [examples/remote_plugin_minimal/README.md](../../examples/remote_plugin_minimal/README.md)（站内可写 200 字摘要 + 外链） |
| `/plugins/submit` | **投稿说明**：作者如何被收录——**仅登记信息**（见 [COMMUNITY_WEB_VISION.md](./COMMUNITY_WEB_VISION.md) 成本策略）；必填：公开仓库、协议兼容说明、测试过的 oclive 版本 |

可选：`/plugins/protocol` 放协议长文 **摘要** + 指向主仓文档的「完整版」链接（避免重复维护全文）。

---

## 3. 列表数据从哪来（推荐）

**阶段 A（无后端）**  

- 仓库内维护 **`data/plugins.json`**（或 `plugins/catalog.json`），CI 构建静态站时读入。  
- 每条字段示例：

```json
{
  "id": "my-sidecar",
  "name": "示例侧车",
  "description": "一句话说明能力",
  "repo_url": "https://github.com/...",
  "min_oclive_version": "0.2.0",
  "tags": ["memory", "llm"],
  "author": "昵称或 GitHub id"
}
```

**阶段 B**  

- 与论坛账号打通、后台表单写入数据库 → 再生成 JSON 或 API。

---

## 4. 与「角色包」板块的边界

| 板块 | 交付物 |
|------|--------|
| **角色包** | `.zip` / `.ocpak`，`manifest`/`settings` |
| **插件（本站含义）** | **HTTP 服务** + 环境变量；用户克隆/下载侧车代码或 Release，**自行运行进程** |

若将来收录 **Comfy 工作流等非 oclive Remote 品类**，需在页面 **单独标注**，避免与 `plugin_backends` 混淆。

---

## 5. 启动器联动（后续）

- 启动器「插件」入口可打开 **`/plugins`**（或设置里 `marketBaseUrl` 同源路径 `/plugins`）。  
- 与 [MARKET_LAUNCHER_INTEGRATION.md](./MARKET_LAUNCHER_INTEGRATION.md) 一致：先 `open_url`，再考虑内嵌 WebView。

---

## 6. 相关索引

- 社区站三板块总览：[COMMUNITY_WEB_VISION.md](./COMMUNITY_WEB_VISION.md)  
- 市场与启动器：[MARKET_LAUNCHER_INTEGRATION.md](./MARKET_LAUNCHER_INTEGRATION.md)
