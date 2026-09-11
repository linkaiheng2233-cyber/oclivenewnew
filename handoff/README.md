# handoff · 维护者与 AI 工程入口

**SSOT 范围**：全仓文档的读者分层、状态判读、活跃 handoff 职责与维护/归档规则；不承载运行时业务契约。
**最后更新**：2026-09-11。
**新人开发者**从 [human-docs](../human-docs/README.md) 开始；**创作者**从 [创作者黄金路径](../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md) 开始。

## 文档分责

以下只分配文档职责；具体模块、格式和行为由对应专题拥有。

### 文档分层

| 层 | 读者 | 只负责 | 入口 |
|----|------|--------|------|
| 根 `README` | 所有人 | 项目定位与身份分流 | [README](../README.md) |
| `human-docs/` | 主仓开发者 | 顺序学习、调试、模块开工 | [学习阶梯](../human-docs/README.md) |
| `creator-docs/` | 用户、创作者、插件作者、集成方 | 现行使用说明与公开契约 | [文档索引](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| `handoff/` | 维护者、AI Agent | 工程边界、关键路径、债务、巡检 | 本页 |
| `handoff/archive/` | 查历史的人 | 阶段记录与已完成报告；**非 truth** | [归档索引](archive/ARCHIVE_PROJECT_HISTORY.md) |
| `*-en/` | 英文读者 | 对应中文 SSOT 的镜像 | [creator-docs-en](../creator-docs-en/README.md) · [human-docs-en](../human-docs-en/README.md) |

**人类 / AI 分类是阅读职责，不是两套事实库。** 人类入口解释概念、例子与学习顺序；AI 入口规定改动范围、取证与验收路径。两者都引用同一专题 SSOT。`human-docs/ai-package/` 是保留的导航入口，不再维护第二套 AI 规则；crate README 说明当前物理代码与用法，不决定逻辑 Kernel 边界。

<a id="documentation-status"></a>

## 文档状态与冲突判读

先判断句子在描述什么，不能用文件名、较新日期或“代码里已有”直接裁决：

| 信息类别 | 应怎样读 | 事实入口 |
|---|---|---|
| 已确认职责边界 | 说明什么应该属于谁；不等于物理拆分或全部落实 | [MODULE_MAP §0.1](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities) |
| 公共语义候选 | 已收敛的语义摘要；不是完整 API 定稿或实现合规证明 | [MODULE_MAP §0.2](MODULE_MAP_AND_HANDOFF.md#kernel-semantics-candidate) |
| 当前实现 / 现行接入契约 | 说明今天可调用什么；注明参考 Host、格式版本和源码范围 | [源码对照](MODULE_MAP_AND_HANDOFF.md#kernel-source-map)、专题契约、对应源码 |
| 已准备但未接线 / 部分实现 | 组件存在不代表端到端能力可用；列明断点 | [技术债](TECHNICAL_DEBT_INVENTORY.md)、对应专题 SSOT |
| 暂停 / 计划 / 待决 | 不能作为已交付能力，也不能因文档整理自动恢复工作 | [技术债](TECHNICAL_DEBT_INVENTORY.md)、RFC 的状态头与分阶段说明 |
| 历史记录 / 阶段快照 | 保留当时结论与证据；不能覆盖当前边界 | [归档索引](archive/ARCHIVE_PROJECT_HISTORY.md)、带日期的实施/审查记录 |

RFC 可以包含已实现的切片，不能把整篇一律判成“未实现”；反过来，一项测试或切片完成也不能证明整个 RFC 已交付。文档与源码冲突时，分别记录**期望契约、实际行为、证据与影响**，回到对应 owner 修正；既不让旧实现重定义新边界，也不以候选文字声称实现已符合。调查 unknown 不自动成为产品决策或运行时机制。

## 最短入口

| 任务 | 先读 | 再读 |
|------|------|------|
| AI 改代码或文档 | [AI_CHANGE_BOUNDARIES](AI_CHANGE_BOUNDARIES.md) | [AI_READING_INDEX](AI_READING_INDEX.md) |
| 接手主编排 / DB | [BUS_FACTOR_NOTES](BUS_FACTOR_NOTES.md) | [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md) |
| 改模块或槽位 | [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md) | [SLOT_BACKEND_REALITY_MATRIX](SLOT_BACKEND_REALITY_MATRIX.md) |
| 做全仓审查 | [AI_VERIFICATION_PROTOCOL](AI_VERIFICATION_PROTOCOL.md) | [RECURRING_OPTIMIZATION_PLAYBOOK](RECURRING_OPTIMIZATION_PLAYBOOK.md) |
| 看当前债务 / 冻结 | [TECHNICAL_DEBT_INVENTORY](TECHNICAL_DEBT_INVENTORY.md) | — |
| 改角色包边界 | [ROLE_PACK_BOUNDARY](ROLE_PACK_BOUNDARY.md) | [角色包规范](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |

## 活跃 SSOT

### 架构与边界

| 文件 | 唯一职责 |
|------|----------|
| [MODULE_MAP_AND_HANDOFF.md](MODULE_MAP_AND_HANDOFF.md) | **小 Kernel / 六槽 / Host / Adapter 权责唯一 SSOT**：模块定义、候选公共语义与源码职责对照 |
| [SLOT_BACKEND_REALITY_MATRIX.md](SLOT_BACKEND_REALITY_MATRIX.md) | 六槽 × backend 实现真值 |
| [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md) | 角色包、蓝图、发行版和会话分责 |
| [CHAT_STORAGE_ARCHITECTURE.md](CHAT_STORAGE_ARCHITECTURE.md) | 聊天日志、短期与长期记忆存储 |
| [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md) | domain / infrastructure 依赖方向 |
| [BLUEPRINT_FOLDER_LAYOUT.md](BLUEPRINT_FOLDER_LAYOUT.md) | 蓝图目录和 includes 布局 |
| [KERNEL_SCHEDULER_RESCOPE.md](KERNEL_SCHEDULER_RESCOPE.md) | 参考运行时的进程/资源调度范围；不定义小 Kernel 的六槽调度权 |
| [THREE_DISTRO_KERNEL_CLOSURE.md](THREE_DISTRO_KERNEL_CLOSURE.md) | 三发行版内核结项约束；新能力以 HostProfile SSOT 为准 |
| [EVENT_RING.md](../creator-docs/plugin-and-architecture/EVENT_RING.md) | Event Ring wire、注册、权威边界与主动回合授权公开契约 |
| [RFC_RUNTIME_EVENT_STREAM.md](../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md) | 外围 Runtime Event Stream 的分层与分阶段实验边界；已实现切片、暂停项和 Production 缺口按 RFC 与技术债判断，不把 trace/shadow 当正式 Stream |

### 关键路径与验证

| 文件 | 唯一职责 |
|------|----------|
| [BUS_FACTOR_NOTES.md](BUS_FACTOR_NOTES.md) | 主编排、DB、错误码与源码锚点 |
| [INVOKE_HOTPATH_MATRIX.md](INVOKE_HOTPATH_MATRIX.md) | Tauri invoke 热路径矩阵 |
| [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md) | 破坏性变更流程 |
| [AI_CHANGE_BOUNDARIES.md](AI_CHANGE_BOUNDARIES.md) | AI 改动边界 G1–G17、规划/执行委派边界（含关联能力闭环） |
| [AI_READING_INDEX.md](AI_READING_INDEX.md) | AI 按任务深读导航，不承载事实 |
| [AI_VERIFICATION_PROTOCOL.md](AI_VERIFICATION_PROTOCOL.md) | 审查与带数字汇报的核实规则 |
| [oclive-adaptive-pipeline/SKILL.md](workflows/oclive-adaptive-pipeline/SKILL.md) | 第二条模型分工层：路由、dispatch、Luna 实施与 GPT6 验收（2026-09-05） |
| [RECURRING_OPTIMIZATION_PLAYBOOK.md](RECURRING_OPTIMIZATION_PLAYBOOK.md) | 多轮巡检流程 |

### 状态、性能与专项执行

| 文件 | 唯一职责 |
|------|----------|
| [TECHNICAL_DEBT_INVENTORY.md](TECHNICAL_DEBT_INVENTORY.md) | 活跃债、冻结项与下一动作 |
| [PRODUCT_LINE_TASK_BUCKETS.md](PRODUCT_LINE_TASK_BUCKETS.md) | 产品线执行分桶 |
| [PERF_PHASES.md](PERF_PHASES.md) | 性能阶段与复现入口 |
| [TTFT_BENCHMARK.md](TTFT_BENCHMARK.md) | TTFT 基准 |
| [DEEP_PROMPT_DISTILLATION.md](DEEP_PROMPT_DISTILLATION.md) | Deep 路径与 Prompt 蒸馏专项 |
| [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) | 双核实验交接 |
| [GITHUB_PLUGIN_INDEX_LINE.md](GITHUB_PLUGIN_INDEX_LINE.md) | GitHub 插件索引线 |
| [COMMENT_ENGLISH_MIGRATION_PLAN.md](COMMENT_ENGLISH_MIGRATION_PLAN.md) | 注释英文化计划 |
| [GOOD_FIRST_ISSUES.md](GOOD_FIRST_ISSUES.md) | 新人 issue 策展 |
| [OCLIVE_POSITIONING_DIFFERENTIATION.md](OCLIVE_POSITIONING_DIFFERENTIATION.md) | 工具内核本质、产品定位与差异化 |

## 发行版工作区

| 目录 | 入口 |
|------|------|
| `theater/` | [AI Theater](theater/README.md) |
| `vscode/` | [VS Code Flash](vscode/README.md) |
| `pack-editor/` | [角色包编写器](pack-editor/README.md) |
| `launcher/` | [退役启动器](launcher/README.md) |
| `studio/` | [工作室叙事](studio/README.md) |
| `debt-marathon/` | [债务马拉松记录](debt-marathon/README.md) |

## 归档与新增规则

- 已完成的 phase、closure、时点审查和旧实施计划进入 `handoff/archive/`，不得作为现行行为依据。
- 现行行为以源码、上表 SSOT、`creator-docs/` 契约和 `TECHNICAL_DEBT_INVENTORY` 为准。
- 不新建第二份项目总览、模块表、状态页或发版清单；扩展现有 SSOT并从索引链接。
- 根级新增 `handoff/*.md` 必须满足 [AI_CHANGE_BOUNDARIES G11–G16](AI_CHANGE_BOUNDARIES.md) 并登记到本页。

<a id="documentation-maintenance"></a>

## 防漂移维护闭环

1. **定位 owner**：名称查 NAMING，模块权责查 MODULE_MAP，格式查角色包/插件契约，当前状态查技术债；导航页用链接，不复制长表或进度数字。
2. **区分层次**：先写适用 Host / 格式 / 日期，区分目标、候选和当前代码。现有 `kernel/` 路径、`OcliveKernel` 名称和固定参考主链不自动定义最小 Kernel。
3. **同步消费入口**：职责/格式变更同步相关学习页、AI 路由和已有英文镜像；老图与旧实施计划若继续展示，应标注参考实现或历史范围，不覆盖原证据。
4. **留可追溯依据**：实现结论指向源码；阶段结果保留基线、执行过的命令与未验证项。“已阅读测试”不能写成“测试通过”。
5. **验证并审差异**：检查链接、路径、注册、镜像及编码，再审“是否把领域提交、固定调度或产品字段塞回 Kernel”。静态门禁通过不等于运行语义已验证。

**2026-09-11 文档对齐范围**：根入口、命名/定位、架构导览、人类学习/术语、AI 导航与参考运行时适用范围。保留现有专题契约与历史证据；未借文档更新实现新的小 Kernel、恢复 Event Stream/R7/R1–R4、或宣称全仓源码已完成合规验证。后续实现缺口仍由 [K-CORE-BOUNDARY-01 等技术债](TECHNICAL_DEBT_INVENTORY.md) 跟踪。

纯文档门禁：`git diff --check` · `node scripts/check-doc-registry.mjs` · `node scripts/check-markdown-links.mjs --tracked` · `node scripts/check-stale-paths.mjs --docs-only` · `node scripts/check-doc-mirror.mjs`。链接脚本检查其范围内受跟踪 Markdown 的本地目标路径（排除 archive 等目录），不验证所有章节锚点或语义；新增/改动锚点须另查。镜像门禁也不证明逐字语义等价。这些检查不替代人工审查，不要求启动服务或运行 Rust/外部探测。

**本轮验证记录（2026-09-11）**：在仓库根目录执行上述五项静态门禁，均通过；另以只读脚本核对变更文件 UTF-8/无 BOM、本文与 MODULE_MAP 的显式边界锚点、源码行号不越界及仅 Markdown 写集。源码对照基线见 MODULE_MAP §0.3。未运行 Rust/产品测试、服务或真实外部探测；本记录不代表远端 CI、历史证据重放或全仓运行语义验收。
