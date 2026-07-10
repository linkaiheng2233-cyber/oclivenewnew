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

### 5. 微调工坊 + 专家路由（灵魂权重层）

| 含义 | **三发行版工程结项后**的创作者工具链第三阶段：独立 **微调小模型** 软件，把口癖/节奏/直播态沉淀为 **LoRA/SFT adapter**，经角色包分发；运行时由 **专家模型设施子模块**（`expert_routing.json` · `slot.lora.apply`）按场景/关键词等条件切换 adapter。 |
|------|------|
| 涉及仓库 | 新仓或 **oclive-pack-editor** 姊妹工具（推荐独立 Tauri，避免训练/GPU 拖慢编写器）；主仓 `expert_routing` + `slot.lora.apply` + `oclive_validation` 契约；可选 directory 推理插件。 |
| 与定位关系 | 补 **权重层**，不变成封闭「性格引擎」；对标 AI 主播类实践的微调投入，但产物走 **组装—契约—打包—分发** 标准层。详见 [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md)「微调工坊」小节。 |
| 实现时需考虑 | 语料隐私与授权；基座模型（优先 Ollama 生态小参）；默认 **expert 子流程** 切换 adapter，不强制替换主 `plugin_backends.llm`；评测（prompt-only vs LoRA vs LoRA+专家）。 |
| 状态 | **愿景已纳入 · 待排期**；T0 RFC 未开。`expert_routing` / `dual_core` **冻结期内**仅契约+工坊原型，不接 Stable 主链。 |
| 场景参考 | [APPLICATION_SCENARIOS.md](APPLICATION_SCENARIOS.md) **S11** |

### 6. 具身互动 · 性格驱动的「手脚」（Playroom）

| 含义 | 角色除对话外，按人设 **在宿主上动手**：**被动**（用户开口 → agent + MCP）与 **自发**（idle → 行为导演按七维/包策略选动作）。例：高好奇心「小孩」在 **playroom 沙盒** 里建/删文件夹，并反馈到通知与记忆。 |
|------|------|
| 涉及仓库 | **oclivenewnew**（内核独立通道 · agent/MCP · Tauri 沙盒目录 · 记忆写入）；可选 **oclive-vscode** 渗透侧 **虚拟工作区**（P3）。 |
| 与定位关系 | 补 **具身层**，不变成 OpenClaw 式「整台电脑通用 Agent」；聊天仍 co-present，动手走沙盒 + 授权。详见 [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md)「具身互动」专节。 |
| 实现时需考虑 | playroom 路径硬编码在宿主；delete 限频与撤销；`skip_agent` 与 Chat Pro 默认 profile；与 `autonomous_scene`（仅虚拟位移）区分。 |
| 状态 | **愿景已纳入 · 待排期**；P1 被动手脚 → P2 idle 自发 → P3 跨发行版；T0 RFC 未开。 |
| 场景参考 | [APPLICATION_SCENARIOS.md](APPLICATION_SCENARIOS.md) **S12** |

---

## 二、相较于愿景：仍在路上 / 可深化的项

下列摘自 [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) 及当前实现对照，**与第一节合并排期**。

| 类别 | 内容 |
|------|------|
| **契约与版本** | `PACK_VERSIONING`、`min_runtime_version`、未知字段策略在创作者与编写器侧的持续收紧。 |
| **编写器 MVP** | 从「能导出且可被加载」到「少手写 JSON、校验与运行时一致」；见 [EDITOR_VALIDATION_ROADMAP.md](../role-pack/EDITOR_VALIDATION_ROADMAP.md)。 |
| **可替换性** | Memory/Emotion 等槽位 `builtin_v2` 已收敛为读兼容 alias（无独立 V2 实现，D-SLOT-01）；编写器选项与文档已对齐。 |
| **外接插件与安全** | Remote JSON-RPC 已有；用户确认策略、可执行路径边界可产品化。 |
| **包内知识（月 5）** | `knowledge/` 与换包版本后的行为；可做回归场景与编写器侧编辑体验。 |
| **双软件叙事 + 启动器（月 6）** | README 分工、新用户路径；与第一节「依赖管理」叠加时需统一对外说法。 |
| **远期 backlog** | WASM 插件、关系/多模式细化、动态 `.dll`/`.so`（谨慎）等，见愿景文「第 7 月及以后」。 |
| **微调工坊（T0–T3）** | 三发行版后创作者权重层；与专家路由组合；见愿景文专节与本文 §五。 |
| **具身互动（P1–P3）** | playroom 沙盒 + 被动/自发手脚；见愿景文专节与本文 §六。 |

---

## 三、排期时建议使用的四分法

将需求归入下列四类，避免「体验功能」与「契约地基」混在同一迭代里难以验收：

1. **创作者闭环**：编写器试聊（若做）、校验对齐、导出与 oclive 导入（含 `.ocpak`/文件夹）。  
2. **玩家上手**：启动器环境检测 → 可选进阶为 Ollama + 模型引导。  
3. **分发与生态**：市场/UGC（角色包、插件）= 新系统层，依赖版本、签名、信任模型。  
4. **愿景地基**：契约、第二套 backend、Remote、知识、测试与文档。  
5. **灵魂权重层**：微调工坊产物、adapter 卫星文件、专家路由运行时切换、评测台对比（晚于三发行版 smoke，非 Theater P0 阻塞）。  
6. **具身互动**：playroom 沙盒 MCP、行为导演独立通道、idle 自发与跨发行版虚拟工作区（非 P0 阻塞）。

---

## 四、相关索引

- 按月里程碑：[VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md)  
- **产品首发门槛（P0/P1）**：[../../handoff/archive/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](../../handoff/archive/PRODUCT_AND_KERNEL_GAP_CHECKLIST.md)  
- **产品线任务分桶（按复杂度）**：[../../handoff/PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md)  
- **发版前勾选表（P0 子集）**：[../../handoff/archive/PRODUCT_RELEASE_CHECKLIST.md](../../handoff/archive/PRODUCT_RELEASE_CHECKLIST.md)  
- 包版本：[PACK_VERSIONING.md](../role-pack/PACK_VERSIONING.md)  
- 创作者工作流：[../getting-started/CREATOR_WORKFLOW.md](../getting-started/CREATOR_WORKFLOW.md)  
- 角色包导入测试清单：[../../distros/chat-pro/roles/TESTING_ROLE_PACK_IMPORT.md](../../distros/chat-pro/roles/TESTING_ROLE_PACK_IMPORT.md)  

---

*初版整理自产品讨论与路线图对照；实施顺序以维护者决策为准。*
