# handoff · 维护者与 AI 工程入口

**SSOT 范围**：全仓文档的读者分层、状态判读、活跃 handoff 职责与维护/归档规则；不承载运行时业务契约。
**最后更新**：2026-10-30（补公开源码文档与本机过程材料的分流边界）。
**新人开发者**从 [human-docs](../human-docs/README.md) 开始；**创作者**从 [创作者黄金路径](../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md) 开始。

**六槽当前交接状态**：先读 [B1 + B2 阶段总收口](#six-slot-stage-closure)，再查 [B1 绑定](#six-slot-b1-closure) 与 [逐槽实现／验收](#six-slot-b2-adaptation)。较早的“待实施／尚未编译／Event 尚未落实”属于当时记录，不是当前开工指令。

**ChatPro 接入验收**：沿内核向外的分层工作范围、现有证据与未证边界见 [Host 接入检查单](CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md)；发行版 smoke 是下游回归，不反向定义六槽 Base。

## 文档分责

以下只分配文档职责；具体模块、格式和行为由对应专题拥有。

### 文档分层

| 层 | 读者 | 只负责 | 入口 |
|----|------|--------|------|
| 根 `README` | 所有人 | 项目定位与身份分流 | [README](../README.md) |
| `human-docs/` | 主仓开发者 | 顺序学习、调试、模块开工 | [学习阶梯](../human-docs/README.md) |
| `creator-docs/` | 用户、创作者、插件作者、集成方 | 现行使用说明与公开契约 | [文档索引](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) |
| `handoff/` | 维护者、AI Agent | 工程边界、关键路径、债务、巡检 | 本页 |
| `handoff/archive/` | 查历史的人 | 外置历史的查阅说明；**非 truth** | [归档索引](archive/ARCHIVE_PROJECT_HISTORY.md) |
| `*-en/` | 英文读者 | 对应中文 SSOT 的镜像 | [creator-docs-en](../creator-docs-en/README.md) · [human-docs-en](../human-docs-en/README.md) |

**人类 / AI 分类是阅读职责，不是两套事实库。** 人类入口解释概念、例子与学习顺序；AI 入口规定改动范围、取证与验收路径。两者都引用同一专题 SSOT。`human-docs/ai-package/` 是保留的导航入口，不再维护第二套 AI 规则；crate README 说明当前物理代码与用法，不决定逻辑 Kernel 边界。

<a id="documentation-publishing-boundary"></a>

### 公开源码与过程材料的边界

公开源码保留使用、接入、兼容、安全与贡献说明，以及维护所需的现行规范、关键决定、冻结边界和机器消费的最小计划。普通用户不必阅读维护规范，不代表这些规范应当删除。

普通会话交接、逐命令执行记录、原始测试输出、机器路径与环境快照、路演和视频筹备稿，默认保存在仓库外的本机或私有维护区；公开文档只保留必要结论、受检 SHA、验证范围及可携带的获取说明，不复制整段运行流水，也不为了交接新增一份波次长文。凭据、聊天正文和运行态目录不得作为公开证据。

现有 `archive/`、`debt-marathon/waves/` 和 `long-plans/` 分批核对引用与脚本消费者后再迁移。控制器仍读取的契约、最小波次恢复记录和当前状态继续保留；原始执行工件外置不等于取消验收或把仅在本机的证据说成他人可复现。删除当前文件不清除 Git 历史。

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
| 普通用户安装、聊天、备份与排错 | [当前 Chat Pro 用户手册](../creator-docs/getting-started/USER_MANUAL.md) | [版本与候选状态](../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md)；无需通读下方工程阶段记录 |
| 改 Chat Pro 新前端 | [前端开工包](../human-docs/modules/surfaces/frontend-chat-pro.md) | [默认壳与收起能力](MODULE_MAP_AND_HANDOFF.md#chatpro-deferred-surfaces)；旧壳源码不等于当前默认界面 |
| 接手主编排 / DB | [BUS_FACTOR_NOTES](BUS_FACTOR_NOTES.md) | [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md) |
| 改模块或槽位 | [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md) | [SLOT_BACKEND_REALITY_MATRIX](SLOT_BACKEND_REALITY_MATRIX.md) |
| 接续六槽基础契约讨论 | [已确认方向](MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension) · [公共承诺决策](MODULE_MAP_AND_HANDOFF.md#six-slot-confirmed-decisions) | [统一模板候选](MODULE_MAP_AND_HANDOFF.md#six-slot-contract-candidate) · [逐槽审阅](MODULE_MAP_AND_HANDOFF.md#six-slot-consistency-review) · [公共面组合](MODULE_MAP_AND_HANDOFF.md#small-kernel-composition) · [接口表达建议](MODULE_MAP_AND_HANDOFF.md#six-slot-interface-proposal) · [AI 阅读顺序](AI_READING_INDEX.md#kernel-slot-reading) |
| 做全仓审查 | [AI_VERIFICATION_PROTOCOL](AI_VERIFICATION_PROTOCOL.md) | [RECURRING_OPTIMIZATION_PLAYBOOK](RECURRING_OPTIMIZATION_PLAYBOOK.md) |
| 看当前债务 / 冻结 / 接手 | [TECHNICAL_DEBT_INVENTORY](TECHNICAL_DEBT_INVENTORY.md) | [变动与续跑协议](debt-marathon/DEBT_CHANGELOG.md) · [覆盖审查](debt-marathon/COVERAGE.md) |
| 改角色包边界 | [ROLE_PACK_BOUNDARY](ROLE_PACK_BOUNDARY.md) | [角色包规范](../creator-docs/role-pack/ROLE_PACK_SPEC.md) |

## 活跃 SSOT

### 架构与边界

| 文件 | 唯一职责 |
|------|----------|
| [MODULE_MAP_AND_HANDOFF.md](MODULE_MAP_AND_HANDOFF.md) | **小 Kernel / 六槽 / Host / Adapter 权责唯一 SSOT**：模块定义、候选公共语义与源码职责对照；§0.5.1 另收 Host 侧行为承诺（仅收已被真实 Host 执行过的条款，其余列为候选项） |
| [SLOT_BACKEND_REALITY_MATRIX.md](SLOT_BACKEND_REALITY_MATRIX.md) | 六槽 × backend 实现真值 |
| [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md) | 角色包、蓝图、发行版和会话分责 |
| [CHAT_STORAGE_ARCHITECTURE.md](CHAT_STORAGE_ARCHITECTURE.md) | 聊天日志、短期与长期记忆存储 |
| [ARCHITECTURE_LAYERING.md](ARCHITECTURE_LAYERING.md) | domain / infrastructure 依赖方向 |
| [BLUEPRINT_FOLDER_LAYOUT.md](BLUEPRINT_FOLDER_LAYOUT.md) | 蓝图目录和 includes 布局 |
| [KERNEL_SCHEDULER_RESCOPE.md](KERNEL_SCHEDULER_RESCOPE.md) | 参考运行时的进程/资源调度范围；不定义小 Kernel 的六槽调度权 |
| [THREE_DISTRO_KERNEL_CLOSURE.md](THREE_DISTRO_KERNEL_CLOSURE.md) | 三发行版内核结项约束；新能力以 HostProfile SSOT 为准 |
| [EVENT_RING.md](../creator-docs/plugin-and-architecture/EVENT_RING.md) | Event Ring wire、注册、权威边界与主动回合授权公开契约 |
| [RFC_RUNTIME_EVENT_STREAM.md](../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md) | 外围 Runtime Event Stream 的分层与分阶段实验边界；已实现切片、暂停项和 Production 缺口按 RFC 与技术债判断，不把 trace/shadow 当正式 Stream |
| [RFC_DEVELOPER_SDK_V0.md](../creator-docs/rfc/RFC_DEVELOPER_SDK_V0.md) | 源码接入工具包的交付形态与扩展停止线；实际配方指向独立 Memory 工程指南，不新增公共门面或重定义六槽 |

### 关键路径与验证

| 文件 | 唯一职责 |
|------|----------|
| [BUS_FACTOR_NOTES.md](BUS_FACTOR_NOTES.md) | 主编排、DB、错误码与源码锚点；§11 不变量表（违反后会发生什么） |
| [INVOKE_HOTPATH_MATRIX.md](INVOKE_HOTPATH_MATRIX.md) | Tauri invoke 热路径矩阵 |
| [BREAKING_CHANGE_PROCESS.md](BREAKING_CHANGE_PROCESS.md) | 破坏性变更流程 |
| [AI_CHANGE_BOUNDARIES.md](AI_CHANGE_BOUNDARIES.md) | AI 改动边界 G1–G17、规划/执行委派边界（含关联能力闭环） |
| [AI_READING_INDEX.md](AI_READING_INDEX.md) | AI 按任务深读导航，不承载事实 |
| [catalog/README.md](catalog/README.md) | 可选文档导航的命令、范围与维护；即时原文导航与精选记录分开，不改变原 SSOT、状态或验收规则；缺卡／旧卡可回原文 |
| [AI_VERIFICATION_PROTOCOL.md](AI_VERIFICATION_PROTOCOL.md) | 审查、带数字汇报与 Document → Code 支撑调查的证据/止点规则 |
| [dev-pipeline/SKILL.md](workflows/dev-pipeline/SKILL.md) | 通用七阶段职责、尺寸与证据状态；仓库正式来源，本机用户级 Skill 仅安装副本；OCLive 增量见 [项目 Skill](../.cursor/skills/oclive-dev-pipeline/SKILL.md) |
| [oclive-adaptive-pipeline/SKILL.md](workflows/oclive-adaptive-pipeline/SKILL.md) | 第二条自适应协作层：按风险规划、选择实施者与复核，不绑定型号（2026-10-01） |
| [RECURRING_OPTIMIZATION_PLAYBOOK.md](RECURRING_OPTIMIZATION_PLAYBOOK.md) | 多轮巡检流程 |

### 状态、性能与专项执行

| 文件 | 唯一职责 |
|------|----------|
| [TECHNICAL_DEBT_INVENTORY.md](TECHNICAL_DEBT_INVENTORY.md) | 活跃债、冻结项与下一动作 |
| [debt-marathon/DEBT_CHANGELOG.md](debt-marathon/DEBT_CHANGELOG.md) | 债务变动事件、证据关联与 AI 接手协议；不维护第二份当前状态表；执行准入仍由 [队列与计划](debt-marathon/README.md) 决定 |
| [debt-marathon/waves/WAVE-20260929-ACTIONS-PINS.md](debt-marathon/waves/WAVE-20260929-ACTIONS-PINS.md) | K-SUPPLY-10 固定引用的来源、配置差量与验证证据；当前债务状态仍由主台账维护 |
| [debt-marathon/waves/WAVE-20260929-DEBT-REFERENCES.md](debt-marathon/waves/WAVE-20260929-DEBT-REFERENCES.md) | D-DEBT-LEDGER-01 权威行/引用、历史迁移与持续结构检查合同和保全核对；不维护另一份当前债务状态表 |
| [PRODUCT_LINE_TASK_BUCKETS.md](PRODUCT_LINE_TASK_BUCKETS.md) | 产品线执行分桶 |
| [PERF_PHASES.md](PERF_PHASES.md) | 性能阶段与复现入口 |
| [TTFT_BENCHMARK.md](TTFT_BENCHMARK.md) | TTFT 基准 |
| [DEEP_PROMPT_DISTILLATION.md](DEEP_PROMPT_DISTILLATION.md) | Deep 路径与 Prompt 蒸馏专项 |
| [DUAL_CORE_CURSOR_HANDOFF.md](DUAL_CORE_CURSOR_HANDOFF.md) | 双核实验交接 |
| [GITHUB_PLUGIN_INDEX_LINE.md](GITHUB_PLUGIN_INDEX_LINE.md) | GitHub 插件索引线 |
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

### 用户与发布入口

| 文件 | 唯一职责 |
|------|----------|
| [USER_MANUAL](../creator-docs/getting-started/USER_MANUAL.md) | 当前默认 Chat Pro 的安装、聊天、设置与自助恢复；旧壳技术路径按范围另查 |
| [PROJECT_CURRENT_STATUS](../creator-docs/getting-started/PROJECT_CURRENT_STATUS.md) | 当前版本、候选基线与交付/未验状态的事实快照；其他入口不复制版本表 |
| [RELEASE_VERSIONING](../creator-docs/development/RELEASE_VERSIONING.md) | 独立版本、候选与 MSI 映射规则；不重复维护当前数值 |

## 归档与新增规则

- 已完成的阶段过程、时点审查和旧执行计划放到源码外的个人或私有归档；项目内保留必要结论和固定来源，不作为现行行为依据。
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


## 六槽与 Host 的接续入口

<a id="six-slot-b1-closure"></a>

**B1 独立调用面**：源码与职责查 [MODULE_MAP 的 Base 接入点](MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，实际用法查 [Memory 接入示例](../examples/developer-sdk-memory/README.md)。本页不复制 trait、请求或兼容规则。

<a id="six-slot-b2-adaptation"></a>

**B2 有限参考适配**：各实现的边界以 [源码职责对照](MODULE_MAP_AND_HANDOFF.md#kernel-source-map)及其实现/测试为准；Event 的组合方式不成为所有实现的必需依赖，局部适配不等于参考 Host 全链路验收。

<a id="six-slot-stage-closure"></a>

**既有阶段结论**：2026-09-19 的 B1/B2 本地收口只覆盖当时声明的独立调用面与有限实现，不代表 Stable API 发布、真实模型质量、物理打包或产品全量验证。后续 Chat Pro 的当前接线、证据与未证边界读 [Host 接入检查单](CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md)，冻结与下一动作读 [主台账](TECHNICAL_DEBT_INVENTORY.md)。

旧阶段的逐轮命令、试错、复核和机器路径已迁入维护者个人文档；原版本也可按固定源码基线 `925310c83c94d53dda66fda8eb0c0f1cd9f50de4` 查阅 Git 历史。整理没有重新执行旧验收，也没有变更其范围或债务状态。
