# 体验差异化 backlog · 与愿景对照

本文汇总两类内容，供排期时对照（**不替代** [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) 中的月度里程碑）：

1. **产品体验向**：从「能用」到「好用」的差异化方向（含外部讨论稿整理）。  
2. **愿景对照**：路线图中**仍在推进或待深化**的项，与上表合并决策。

重大方向变更时请更新本文日期说明，并与 `CHANGELOG.md`、契约文档同步。

---

## 一、差异化方向（三件套：运行时 / 编写器 / 启动器）

### 1. 一体化创作与测试

| 含义 | 编写器内提供与角色的**快速试聊**，调整配置后可立即对话验证，接近「所见即所得」。 |
|------|------|
| 涉及仓库 | 主要为 **oclive-pack-editor**；与 **oclivenewnew** 的 `load_role`、对话 API 契约对齐。 |
| 实现时需考虑 | 嵌入式轻量试聊 vs 调本机运行时/子进程；须与 **`load_role` 同一套校验**（参见 [EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)）。 |
| 状态 | **待产品决策与排期**。 |

### 2. 更智能的依赖管理（启动器）

| 含义 | 在「检测环境」之上，向 **一键安装/配置 Ollama**、**拉取推荐对话模型** 演进，接近「下载即聊」。 |
|------|------|
| 涉及仓库 | **oclive-launcher**；与 [CREATOR_PLUGIN_ARCHITECTURE.md](../plugin-and-architecture/CREATOR_PLUGIN_ARCHITECTURE.md) 中的环境变量、本地边界一致。 |
| 实现时需考虑 | 安装权限、磁盘与网络、模型许可证与用户提示、离线场景；不宜静默覆盖用户已有配置。 |
| 已落实（基础） | 启动器内 **环境与排障**：检测 Node / npm、Ollama（CLI 与本地 API）、编写器/oclive 项目目录与 `package.json`；**一键重置**损坏的启动器配置（备份 `.corrupt.bak`）；**打开配置目录**。详见 **oclive-launcher** 仓库 `README.md`。 |
| 状态 | **进阶**（一键装 Ollama、拉模型、整合包）仍 **待排期**。 |

### 3. 插件 / 角色市场与 UGC 生态

| 含义 | **官方或社区**角色包、插件的浏览、安装、更新，形成 UGC 与持续分发能力。 |
|------|------|
| 涉及仓库 | 三件套 + **服务端/索引策略**（若建中央仓库）；与 [PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)、`schema_version`、签名与信任模型强相关。 |
| 实现时需考虑 | 与现有 **磁盘导入 / `.ocpak`** 的关系；安全（签名、来源校验）、与 [REMOTE_PLUGIN_PROTOCOL.md](../plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) 及月 4「外接协议」的边界。 |
| 状态 | **待产品决策与排期**（通常晚于单机闭环与契约稳定）。 |
| 落地说明（发版与市场同发、启动器入口、分阶段） | **[MARKET_LAUNCHER_INTEGRATION.md](MARKET_LAUNCHER_INTEGRATION.md)** |

### 4. 开源协作

| 含义 | 社区贡献插件、角色包、文档；模板仓与示例包降低上手成本。 |
|------|------|
| 已有基础 | 根目录 [CONTRIBUTING.md](../../CONTRIBUTING.md)、扩展点 [EXTENSION_POINTS.md](../plugin-and-architecture/EXTENSION_POINTS.md)。 |
| 状态 | **持续推进**；与上表「市场/UGC」可联动，但不等价。 |

---

## 二、相较于愿景：仍在路上 / 可深化的项

下列摘自 [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) 及当前实现对照，**与第一节合并排期**。

| 类别 | 内容 |
|------|------|
| **契约与版本** | `PACK_VERSIONING`、`min_runtime_version`、未知字段策略在创作者与编写器侧的持续收紧。 |
| **编写器 MVP** | 从「能导出且可被加载」到「少手写 JSON、校验与运行时一致」；见 [EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)。 |
| **可替换性** | Memory/Emotion 等 **builtin_v2** 已在引擎侧；编写器是否暴露选项、文档是否一致可再对齐。 |
| **外接插件与安全** | Remote JSON-RPC 已有；用户确认策略、可执行路径边界可产品化。 |
| **包内知识（月 5）** | `knowledge/` 与换包版本后的行为；可做回归场景与编写器侧编辑体验。 |
| **双软件叙事 + 启动器（月 6）** | README 分工、新用户路径；与第一节「依赖管理」叠加时需统一对外说法。 |
| **远期 backlog** | WASM 插件、关系/多模式细化、动态 `.dll`/`.so`（谨慎）等，见愿景文「第 7 月及以后」。 |

---

## 三、排期时建议使用的四分法

将需求归入下列四类，避免「体验功能」与「契约地基」混在同一迭代里难以验收：

1. **创作者闭环**：编写器试聊（若做）、校验对齐、导出与 oclive 导入（含 `.ocpak`/文件夹）。  
2. **玩家上手**：启动器环境检测 → 可选进阶为 Ollama + 模型引导。  
3. **分发与生态**：市场/UGC（角色包、插件）= 新系统层，依赖版本、签名、信任模型。  
4. **愿景地基**：契约、第二套 backend、Remote、知识、测试与文档。

---

## 四、相关索引

- 按月里程碑：[VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md)  
- 包版本：[PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)  
- 创作者工作流：[../getting-started/CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)  
- 角色包导入测试清单：[../../roles/TESTING_ROLE_PACK_IMPORT.md](../../roles/TESTING_ROLE_PACK_IMPORT.md)  

---

*初版整理自产品讨论与路线图对照；实施顺序以维护者决策为准。*
