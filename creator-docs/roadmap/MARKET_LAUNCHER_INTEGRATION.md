# 角色包 / 插件市场 · 与启动器联动（发版同发）

本文描述目标体验：**发布桌面端（运行时 / 启动器 / 编写器）时，「市场」站点或索引一并可用**，用户在 **oclive-launcher** 内可一键进入市场（浏览器或内嵌页），而非仅靠口口相传的链接。

**相关契约**：包版本与字段见 [../role-pack/PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)；侧车插件协议见 [../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)。市场解决的是 **分发与发现**，不是替代 Remote 协议本身。

### 产品参照：社区发现型站点（类比 Comfy 生态常用的「C 站」体验）

目标若对齐 **Civitai 一类站点**，核心不是「一个 zip 列表」，而是 **可逛、可搜、可对比、可追更**：

| 常见能力 | 在 oclive 里的对应物 |
|----------|----------------------|
| 列表 / 筛选 / 标签 | 按题材、兼容 oclive 版本、`schema_version`、标签（自定义）筛 **角色包** |
| 详情页 | 展示 `manifest` 摘要、截图、更新说明、**多版本历史**（同一 `id` 多条 Release） |
| 创作者主页 | 账号 / 组织维度聚合作品（需账号体系） |
| 互动数据 | 下载次数、收藏、评论（可选；需后端与反作弊） |
| 主文件 | `.zip` / `.ocpak` 与 [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md) 一致 |

这意味着 **阶段 A（纯静态 `catalog.json`）** 仍可作为冷启动；要做到「像 C 站」，**阶段 C 会明显变重**：账号与上传、对象存储与 CDN、审核与举报、合规与版权策略等，需单独产品与技术设计，**与本文 §2 各阶段并行规划**，避免先做网站再发现架构扛不住 UGC。

**边界**：市场分发 **角色包**；**ComfyUI 工作流 / 模型文件** 与 oclive 角色包不同，除非你们明确扩展品类，否则不宜混在同一套「包」契约里。

---

## 1. 目标体验（验收口径）

| 维度 | 说明 |
|------|------|
| **用户** | 打开启动器 → 有明确入口「市场 / 角色包」→ 打开后能看到**与当前生态兼容**的列表（至少：名称、版本、`min_runtime_version`、下载、可选简介）。 |
| **发版** | 发 **GitHub Release**（或你们固定渠道）时，**市场索引或静态站**同步更新；旧版启动器仍指向稳定 URL，或支持在设置里改「市场根地址」。 |
| **安全** | 首版可只做 **HTTPS + 官方索引**；签名与第三方上架属后续迭代（见 §5）。 |

---

## 2. 推荐拆分：先「入口 + 静态索引」，再「安装闭环」

### 阶段 A — 最小可行（建议先做）

1. **市场 = 静态站点**（如 **GitHub Pages**、**Cloudflare Pages**、对象存储静态站），路径固定，例如：  
   `https://<org>.github.io/oclive-market/` 或自有域名。
2. **索引文件**：根路径或 `/v1/catalog.json`（版本化），列出条目，每条至少包含：  
   `id`、`name`、`version`、`download_url`（指向 `.zip` / `.ocpak` 或 GitHub Release 资产）、`min_runtime_version`（可选）、`sha256`（可选）。
3. **启动器**：增加 **「市场」入口**（侧栏或首页卡片），行为二选一或并存：  
   - **用系统浏览器打开**市场首页（实现成本最低，Tauri 已有 `open_url`）；或  
   - **内嵌 WebView 窗口**加载同一 URL（沉浸感更好，后续再做）。
4. **配置**：启动器 `launcher-config` 增加可选字段 **`marketBaseUrl`**（默认写死你们正式环境，用户可改为镜像或测试站）。

**本阶段不要求**在启动器内完成「下载并解压到 `roles/`」；用户可先从网页下载，沿用现有 **导入 zip** 路径。先把 **发现与发版同发** 跑通。

### 阶段 B — 安装闭环（后续）

- 启动器拉取 `catalog.json`，列表展示，**一键下载**到临时目录并调用已有 **`install_role_pack_zip`**（或等价）写入 `OCLIVE_ROLES_DIR`。  
- 与阶段 A 的索引格式保持一致，仅多 UI 与校验。

### 阶段 C — 生态与安全（更远）

- 上架审核、签名、`schema_version` 强制、恶意包扫描、增量更新等；与 [BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](./BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md) 第三节「市场与 UGC」对齐。

---

## 3. 「发布软件时让市场同时部署」—— CI 怎么绑

市场与桌面端 **不必** 同一 Git 仓库，但建议 **同一发版节奏**：

| 做法 | 说明 |
|------|------|
| **独立仓库 `oclive-market`** | 含静态站源码 + `catalog.json` 生成脚本；**推送 `main` 或 tag** 时 **Pages 自动部署**；桌面端发 Release 时在 **Release Checklist** 里勾「已确认市场 `catalog.json` 已更新」。 |
| **Monorepo 子目录** | `market/` 与文档同仓；workflow 里 **先 build 市场再 build 启动器**，或并行 job。 |
| **版本对齐** | `catalog.json` 带 **`index_schema_version`**；启动器只消费兼容版本。运行时版本约束用条目里的 **`min_runtime_version`** 与 [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md) 一致。 |

**建议**：第一次落地用 **「Release 说明里写一句：市场索引 commit/tag」+ 自动化 deploy 市场站**，避免手工改两台服务器。

---

## 4. 启动器侧改动清单（实现时）

- **配置**：`LauncherConfig`（Rust）与前端类型增加 `market_base_url`（或 camelCase `marketBaseUrl`），默认值为正式市场根 URL；**保存/加载**与现有 `launcher-config.json` 一致。  
- **UI**：主导航或「第一次使用」页增加 **市场** 按钮；点击 = `open_url(marketBaseUrl)`（阶段 A）。  
- **文档**：**oclive-launcher** `README.md` 功能表更新；本文件链入 [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)。

---

## 5. 风险与开放决策（需产品拍板）

- **信任模型**：首版是否 **仅官方条目**；何时开放第三方投稿。  
- **地域与镜像**：是否允许用户改 `marketBaseUrl`（建议允许）。  
- **与插件（Remote）关系**：市场分发 **角色包 zip**；侧车 **HTTP 服务**仍由用户自建或另册分发，**不要**混成「在市场里装 exe 插件」除非单独设计安全模型。

---

## 6. 相关索引

- **社区站整体形态（论坛 / 角色包 / 插件三板块、Discord 取舍）**：[COMMUNITY_WEB_VISION.md](./COMMUNITY_WEB_VISION.md)  
- 体验向总 backlog：[BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md](./BACKLOG_EXPERIENCE_AND_ECOSYSTEM.md)  
- 工具链后发事项备忘：[SOMEDAY_TOOLCHAIN_CI.md](./SOMEDAY_TOOLCHAIN_CI.md)
