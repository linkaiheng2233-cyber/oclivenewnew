# handoff · 维护者与 AI 工程入口

**SSOT 范围**：全仓文档的读者分层、状态判读、活跃 handoff 职责与维护/归档规则；不承载运行时业务契约。
**最后更新**：2026-10-02。
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
| 接续六槽基础契约讨论 | [已确认方向](MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension) · [公共承诺决策](MODULE_MAP_AND_HANDOFF.md#six-slot-confirmed-decisions) | [统一模板候选](MODULE_MAP_AND_HANDOFF.md#six-slot-contract-candidate) · [逐槽审阅](MODULE_MAP_AND_HANDOFF.md#six-slot-consistency-review) · [公共面组合](MODULE_MAP_AND_HANDOFF.md#small-kernel-composition) · [接口表达建议](MODULE_MAP_AND_HANDOFF.md#six-slot-interface-proposal) · [AI 阅读顺序](AI_READING_INDEX.md#kernel-slot-reading) |
| 做全仓审查 | [AI_VERIFICATION_PROTOCOL](AI_VERIFICATION_PROTOCOL.md) | [RECURRING_OPTIMIZATION_PLAYBOOK](RECURRING_OPTIMIZATION_PLAYBOOK.md) |
| 看当前债务 / 冻结 / 接手 | [TECHNICAL_DEBT_INVENTORY](TECHNICAL_DEBT_INVENTORY.md) | [变动与续跑协议](debt-marathon/DEBT_CHANGELOG.md) · [覆盖审查](debt-marathon/COVERAGE.md) |
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
| [archive/TECHNICAL_DEBT_VERIFICATIONS_202607.md](archive/TECHNICAL_DEBT_VERIFICATIONS_202607.md) | 七月历史 Verification 的旧 SHA/CI 与限定范围；非当前 truth，不定义现行状态或开工权限 |
| [archive/TECHNICAL_DEBT_CLOSEOUT_SNAPSHOTS_202608.md](archive/TECHNICAL_DEBT_CLOSEOUT_SNAPSHOTS_202608.md) | 八月工程与产品收口快照、旧评分和旧排期；非当前 truth，不定义现行状态或开工权限 |
| [archive/TECHNICAL_DEBT_ROUNDS_202606.md](archive/TECHNICAL_DEBT_ROUNDS_202606.md) | 六月轮次 16–19 的旧 Done 表；非当前 truth，当前状态仍以主台账为准 |
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

### 六槽讨论的文档收尾与接续（2026-09-14）

**上一轮记录，保留其范围与证据；当时的待决状态及接续计划由下节更新。**

- **基线与依据**：仓库 HEAD 为 `34c2783071c49e4061bc796ac4bf73a2870a7925`；本次整理的是维护者在当前对话确认的六槽方向，不是新一轮源码测绘。MODULE_MAP §0.3 的历史证据仍绑定其原 SHA，未替换为本轮证据。
- **文档分工**：定义与未决项只在 [MODULE_MAP §0.4–§0.5](MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension) 维护；本页、[AI 接续路径](AI_READING_INDEX.md#kernel-slot-reading) 与人类 [学习页](../human-docs/06_KERNEL_LEARNING_PATH.md) / [英文镜像](../human-docs-en/06_KERNEL_LEARNING_PATH.md) 只导航和解释阅读范围。桌面 DeepSeek 交接文档未改，不另建第二份定义。
- **验证范围**：仅本地文档静态检查；在仓库根执行 `git diff --check`、`node scripts/check-doc-registry.mjs`、`node scripts/check-stale-paths.mjs --docs-only`、`node scripts/check-doc-mirror.mjs`、`node scripts/check-markdown-links.mjs`，以及用链接脚本显式检查本次五份文档，均通过。编码与新增显式锚点另查；中英新增段落人工对照，不将镜像门禁当作翻译或语义证明。
- **保护与未运行**：开工前已有的技术债台账两行改动原样保留，不代为提交或改状态。本轮不改源码、配置、数据库、证据或现行 wire；未运行构建、行为测试、服务、真实模型/外部流量或历史请求重放，未 commit / push。Event Stream/R7、R1–R4 和外围 Host 恢复/context 修复继续暂停。
- **当时的接续计划（现由下节替代）**：先确认 §0.5 两项输出语义，再按统一模板补齐候选；旧锚点 [six-slot-open-decisions](MODULE_MAP_AND_HANDOFF.md#six-slot-open-decisions) 保留，当前结论以该节的已确认状态为准，不再作为活跃待决入口。

### 六槽统一候选整理（2026-09-14 · 后续确认后）

**概念稿 1 的整理记录；后续逐槽审阅修订见下节，原证据不升级为新稿验收。**

- **依据与交付范围**：维护者先后确认两项结果承诺后，授权整理完整六槽候选。定义只在 [MODULE_MAP §0.4–§0.7](MODULE_MAP_AND_HANDOFF.md#six-slot-contract-candidate) 维护；主文档用完整七项模板取代原摘要表，记录决策变更和概念反例检查，未另建六份平行 SSOT。
- **证据级别**：概念稿 1 是文档候选，不是整份契约已批准、API 已发布或现有实现已通过验证。对照依据为当前对话与既有边界；本轮不重跑源码测绘，也不把假设案例记为执行过的测试。
- **写集与保护**：仅主文档、AI 索引、本页及中英学习页；保留前轮修改，技术债台账原有两行不动。HEAD 仍为 `34c2783071c49e4061bc796ac4bf73a2870a7925`；未 commit / push，未改桌面交接文档、源码、配置、数据库或现行 wire。
- **验收口径**：只进行文档静态门禁、编码/锚点/模板完整性检查与文字自审；不以其替代 Rust/产品测试、独立模型复核或远端 CI。具体门禁仍用本页已有命令，不运行构建、行为测试、服务、模型、外部流量或历史请求重放。
- **本轮文档结果**：`git diff --check`、doc registry、stale-paths（docs-only）、doc mirror、默认范围及本轮五文件的 Markdown 链接检查均通过；另核对六槽各七项模板齐备、新增显式锚点与 UTF-8/无 BOM/统一 LF。原权责/候选/历史源码对照段未改，技术债台账哈希与开工时一致；中英新增导航按语义人工对照。以上仅为文档与结构检查证据。
- **接续入口**：审阅 [统一候选](MODULE_MAP_AND_HANDOFF.md#six-slot-contract-candidate) 与 [未定稿部分](MODULE_MAP_AND_HANDOFF.md#six-slot-consistency-review)，再决定是否进入公共数据形状/适配计划；不重复询问两项已决语义。Event Stream/R7、R1–R4 和外围 Host 恢复/context 修复继续暂停。

### 六槽逐槽审阅与组合（2026-09-14 · 概念稿 1.1）

- **授权与分工**：维护者要求先逐槽审阅，再组合 Kernel，最后由 DeepSeek 落实。本轮由主控完成语义审阅与文档整理；沿用 OCLive 工程门禁，模型分工按本次用户指定覆盖第二流水线的默认派发安排。未自动派发子 Agent，未声称独立模型复核；本机通用 `dev-pipeline` 技能缺失，以本仓已存在规则为准。
- **唯一正文**：[MODULE_MAP §0.6–§0.8](MODULE_MAP_AND_HANDOFF.md#six-slot-contract-candidate) 保存审阅稿、删除见证与公共面组合；AI/人类中英入口仅链接。§0.1–§0.3 原权责与历史源码证据保持不变。本轮是语义自审，不新增源码全量合规结论，也不把概念稿记为 API 发布或物理 Kernel 完工。
- **交接止点**：对话中交付 DeepSeek 的下一阶段任务书；先做六槽候选到现行接口的有限映射和实施备料，由主控据此确定公共表示与精确写集后才进入代码实施。不要求 DeepSeek 再重建全部 Kernel/Host，也不让它自行以 DTO、兼容桥或默认值填平未决语义。桌面文档仍只用于 DeepSeek 回传。
- **写集与冻结**：只更新此前在改的五份 Markdown，技术债台账原有改动原样保留；不改 Rust、配置、数据库、现行 wire，不 commit / push。Event Stream/R7、R1–R4 及 Host 恢复/context 修复保持暂停。
- **验收范围**：本次修订需重新进行文档静态门禁、显式锚点、编码、受保护段与写集检查；不运行构建、行为测试、服务、模型或外部探测。概念案例不是运行测试，后续公开接口实施仍须按 applicable 工程门禁验证。
- **本次静态结果**：仓库根的 `git diff --check`、doc registry、stale-paths（docs-only）、doc mirror、默认范围及五文件 Markdown 链接检查均通过；另核对六槽七项模板、40 处目标锚点引用、UTF-8/无 BOM/统一 LF、旧职责/历史证据与参考接口区未改，台账 SHA256 与开工时一致。Git 的 LF→CRLF 提示保留说明，不报告为编码错误；中英导航人工对照。本结果仅覆盖文档，不是可执行契约验收。

### 六槽接口方案准备（2026-09-14 · 概念稿 1.2 / 表示稿 0.1）

- **依据与正文**：维护者确认 Agent 的机器可读任务完成声明留在增强契约，并确认同槽多实现与单实现多增强的区别；已决语义归 [MODULE_MAP §0.4–§0.5](MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension)，模板更新在 §0.6，主控接口表达建议在 [§0.9](MODULE_MAP_AND_HANDOFF.md#six-slot-interface-proposal)。表示建议仍是草案，不把对方向的确认扩大为对 Rust 签名、错误载体或迁移写集的批准。
- **证据与接续**：此前 DeepSeek §45–§46 的有限对照只作实施备料，不从“字段缺失”推导语义缺失，也不从工具被调用/`handled=false` 推导已发生外部效果或安全重放承诺。下一包审查表示稿能否兑现最低承诺、哪些旧接口只能有限适配，并回传具体签名建议、精确写集与验证计划；主控复核后才进入代码实施，不恢复旧 S1–S5 方案。
- **分工与保护**：按用户指定由主控规划/审查、DeepSeek 执行有限对照与后续获准实施；任务书在对话交付，桌面文档仅供回传。仓库 HEAD 为 `34c2783071c49e4061bc796ac4bf73a2870a7925`，已有五份导航/定义文档在本轮继续编辑；技术债台账既有改动原样保留。未改 Rust、配置、数据库或现行 wire，未 commit / push；Event Stream/R7、R1–R4、外围 Host 恢复/context 修复与真实流量继续暂停。
- **验收界限**：只运行本页列出的文档静态检查及编码、锚点、保护范围核对；未运行 Rust/产品测试、构建、服务、模型、数据库实验或外部探测。未来改公开 API 仍须 `cargo test --workspace --doc`，不能以本轮文档门禁或单个 crate 的 doctest 代替。表示稿审查不等于实现已验证。
- **本次文档结果**：`git diff --check`、doc registry、stale-paths（docs-only）、doc mirror、默认范围及本次五文件的 Markdown 链接检查通过；另核对 UTF-8/无 BOM/统一 LF、显式锚点、六槽七项模板以及受保护区域。§0.1–§0.3 与参考说明 §1 以后的原文未改，台账哈希与开工一致；中英导航逐句对照。Git 的 LF→CRLF 提示不计为编码错误；上述结果只证明文档静态检查，不是代码验收。

### B1 接口实施准备（2026-09-14 · 表示稿 0.2）

**历史实施准备记录；当前交付与未验证范围以 [B1 本地收口](#six-slot-b1-closure) 为准，以下原始证据保留。**

- **已决与建议分开**：[MODULE_MAP §0.5](MODULE_MAP_AND_HANDOFF.md#six-slot-confirmed-decisions) 补入维护者确认的异步、已知取消/超时原因和线程亲和边界；[§0.9](MODULE_MAP_AND_HANDOFF.md#six-slot-interface-proposal) 同步为表示稿 0.2。它们不批准某个宏、Future 装箱或错误枚举；旧语义/历史源码区不重开。
- **当前交接物**：本机 `.cursor/plans/six-slot-base-b1.plan.md` 保存主控的完整 B1 Rust 绑定建议、五文件写集、测试矩阵和止点，待用户转交 DeepSeek 4.1 Flash 落实。该目录被 Git 忽略，计划不是已提交文档或第二份语义 SSOT；跨机器交接须附全文，不能只转不存在的本机路径。桌面文档仍仅供 worker 回传。
- **本轮边界**：仅同步 MODULE_MAP、本页和本地计划；不改 Rust、Cargo/现行 wire、技术债或已有中英/AI 导航改动，不 commit/push。Rust 建议尚未编译；不能称 B1 已实现、参考适配已完成或小 Kernel 已完工。文档门禁须对本轮文字重跑，前节 PASS 不升级为本轮证据。
- **后续验收**：沿第一流水线，由主控复核实际公共签名、独立 Base-only 正负例及 G8 workspace doctest 等适用证据后再建本地回滚点。已决线程边界不再重复提问；新增公共承诺/兼容责任分叉仍须裁决。Event Stream/R7、R1–R4、Host 恢复/context 与真实流量继续暂停。
- **本轮静态证据**：`git diff --check`、doc registry、stale-paths（docs-only）、doc mirror、默认 Markdown 范围和上述两份文档/本地计划的显式链接检查通过；编码为 UTF-8 无 BOM/统一 LF，显式锚点无重复。台账、AI/中英导航及 MODULE_MAP 原职责/历史段哈希保持不变。只验证文档与保护范围，不表示 Rust 建议编译通过；Git 的 LF→CRLF 提示单独保留。

<a id="six-slot-b1-closure"></a>

### B1 本地收口（2026-09-15）

**本节是 B1 当时的范围与验收记录；后续 B2 及当前止点见 [阶段总收口](#six-slot-stage-closure)。** 不用下面的“须另行授权”重复开启已经完成的 B2 切片。

**状态：B1 独立 Base Rust 绑定与 Base-only 夹具已实现、主控复核并本地验收（Locally verified）；生产适配与旧 Host 接线未实施。** 这是 B1 切片收口，不是小 Kernel 全部完工、稳定 API 发布或参考 Host 全量合规证明。语义定义仍由 [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension) 拥有，本节只记录实现与验收范围。

- **代码回滚点**：`b19622f5496bb8931cfc0459f6206b2e21ee0176`，`feat(kernel): add independent six-slot Base bindings`；父基线 `0547c5b10fa008e328f5ca883743cb51fba0b41f`。本地提交，未 push；本节不提供远端 CI 通过结论。
- **精确代码范围**：[types 请求／错误](../kernel/crates/oclive_kernel_types/src/slot_base.rs)、[contracts 六个 trait／BaseCallFuture](../kernel/crates/oclive_kernel_contracts/src/slot_base.rs)、[独立夹具](../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs)，以及两个 crate 的 `lib.rs` 显式导出。共五文件；旧接口、现行 wire、Cargo 配置／锁文件和 Host 行为未改。
- **验证证明到哪里**：受控内存夹具覆盖六槽独立接入、显式材料选择、空结果／错误、Prompt 逐字直通、线程亲和与栈借用、Pending 唤醒及可选增强；不由此推导真实 I/O 调度、跨线程适配、取消传播、远端停止或领域／外部效果恢复。公开 rustdoc 的共同绑定说明及六 trait 链接已检查。

以下为主控在上述代码提交前实际重跑并验收的证据；本次文档收尾未把它们重标为新一轮 Rust 测试：

| 验证 | 命令与结果 |
|---|---|
| 局部测试 | `cargo test --locked --offline -p oclive_kernel_types --lib -j 1`：46 passed；`-p oclive_kernel_contracts --lib -j 1`：3 passed；`-p oclive_kernel_contracts --test base_only_fixture -j 1`：16 passed；均 exit 0 |
| 公共文档示例 | `cargo test --locked --offline --workspace --doc -j 1`：exit 0；contracts 11 passed（含两项 compile-fail）。`oclive_validation_wasm` 的 cdylib doctest 不支持警告保留，不计为已执行测试 |
| 静态／边界门禁 | `cargo clippy --locked --offline -p oclive_kernel_types -p oclive_kernel_contracts --all-targets -j 1 -- -D warnings`、`cargo fmt --all -- --check`、`check-domain-layering.mjs`、`npm run check:module-compat`、`check-stale-paths.mjs`、`git diff --check` 均 exit 0 |
| 公开文档 | `cargo doc --locked --offline -p oclive_kernel_contracts -p oclive_kernel_types --no-deps -j 1`：exit 0；另检查公开页面与链接，不仅以构建成功作为可见性证据 |

worker §49 的去掉唤醒后失败／还原后通过属于 worker 的负向验证；主控检查断言并重跑最终版本，未声称亲自重复该临时变异实验。桌面交接文档保存原回传，本页不复制全部实验记录。

**本次收尾文档检查**：`git diff --check`、doc registry、stale-paths（docs-only）、doc mirror，以及四份已改导航文档／本地历史计划的显式 Markdown 链接检查均通过。中英新增导航人工对照；镜像／链接门禁不替代语义审查。本次不重跑 Rust，也不改 MODULE_MAP、技术债状态或代码；Git 的 LF→CRLF 提示不计为编码损坏。

**接续止点**：B1 不再重复实施。本机忽略目录中的旧计划已标为历史执行稿；跨机器交接以本节、代码提交和夹具为准，不依赖该本机文件。B2／生产 Adapter／旧 Host 接线须另行确定范围并授权；不因 B1 完成自动删除旧方法、改产品 fallback、承诺全部增强或把 `K-CORE-BOUNDARY-01` 标为 Done。未运行 Host 全量回归、服务、真实模型、数据库实验或外部流量；Event Stream/R7、R1–R4 及 Host 恢复/context 修复继续暂停。

<a id="six-slot-b2-adaptation"></a>

### 六槽 Base 逐槽参考适配状态（2026-09-19）

**范围**：本节点只登记逐槽 Base 参考适配的实施与验收状态、源码入口和接线止点。六槽语义定义仍由 [MODULE_MAP](MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension) 拥有，本节不复制槽位定义，也不把任一片的完成写成六槽完工、Stable 发布或参考 Host 已迁移。

| 片 | 状态 | 源码入口 | 接线止点 |
|---|---|---|---|
| B2-C1 LLM | 主控本地验收并已提交（`62072037a7d82996721e94463960e32851dba341`，`feat(kernel): add independently verified Ollama Base adapter`，982+/65−） | [具体 Ollama 非流式适配](../kernel/crates/oclive_kernel_host/src/infrastructure/base_llm.rs) | 具体类型 `OllamaBaseAdapter`，未注册到任何 Host；生产装配、观察/资源包装与远端授权链均未接入 |
| B2-C2 Memory | 主控本地验收通过（Locally verified；回滚点见本节 Git 历史） | [原生 Base 实现](../kernel/crates/oclive_kernel_runtime/src/domain/base_memory.rs)、[共享字面匹配 rule](../kernel/crates/oclive_kernel_runtime/src/domain/memory_engine.rs)、[外部调用闭环测试](../kernel/crates/oclive_kernel_runtime/tests/base_memory.rs) | `KeywordMemoryBase` 只处理本次显式材料与字面查询；未接 `slot_runner`／`AppState`／数据库／插件路径，旧 `rank_memories` 与 `MemoryRetrieval` 未改 |
| B2-C3 Prompt | 主控本地验收通过（Locally verified；回滚点见本节 Git 历史） | [原生 Base 实现](../kernel/crates/oclive_kernel_runtime/src/domain/base_prompt.rs)、[外部调用闭环测试](../kernel/crates/oclive_kernel_runtime/tests/base_prompt.rs) | `LiteralMaterialAssembler` **只接受严格空字符串 `requirements`**（非空一律 `Unsupported`，不做包含判定）；空要求时按输入顺序逐字连接材料。未接 `slot_runner`／`AppState`／ChatPro；旧 `PromptBuilder`、`PromptAssembler` 端口与 Host 调用链未改 |
| B2-C4 Emotion | 主控本地验收通过（Locally verified；共享 UTF-8 修复与 C4 实现分开提交，回滚点见本节 Git 历史） | [原生 Base 实现](../kernel/crates/oclive_kernel_runtime/src/domain/base_emotion.rs)、[外部调用闭环测试](../kernel/crates/oclive_kernel_runtime/tests/base_emotion.rs)、[共享边界回归](../kernel/crates/oclive_kernel_runtime/src/domain/lexicon/mod.rs) | `KeywordEmotionBase` 只报告本次材料的词表线索；无命中为 `Ok(None)`，中立条目命中仍有报告。非空 `context` 在分析前返回 `Unsupported`；材料内指令仍是数据。未接 Host／ChatPro，不把词表类别或否定标记升级为已确认的人物情绪；共享修复只改变分隔字符之后的 UTF-8 切片边界，未改词库、匹配／否定规则与旧七维映射 |
| B2-C5 Event | 主控本地验收通过（**限本片离线结构 Locally verified；真实分析质量未验证**；回滚点见本节 Git 历史） | [组合 LlmBase 的有限 Event 实现](../kernel/crates/oclive_kernel_runtime/src/domain/base_event.rs)、[外部调用闭环测试](../kernel/crates/oclive_kernel_runtime/tests/base_event.rs) | `LlmEventAnalyzer<'g>` 只**借用**装配方给定的 `&'g dyn LlmBase`（可为栈上、非 `'static`、非 `Send/Sync`），把本次 `material`/`context` 组织为一个只含这两项成员的 JSON 载荷并**每次调用至多生成一次**（无重试、无规则 fallback）；响应按**私有首行约定**投影：`ANALYSIS`+非空正文 ⇒ `Ok(Some(正文))`，严格 `NO_ANALYSIS`（可带一个行终止符）⇒ `Ok(None)` 只表示后端自报无适用分析，其余形状 ⇒ `Failed` 格式违约，后端 `Err` **原样传播**。不接 `slot_runner`／`AppState`／ChatPro／工具；**未绑定/未调用具体真实模型、未接 Host 产品链**（本实现确实委托 `LlmBase::generate`，生成调用本身存在）；未改 types/contracts、旧 `EventEstimator`/`event_impact_ai`/C1 与冻结项。**历史事实保留**：此前关键词路线（`StateChangeCueAnalyzer`）未获准入、未实施（见桌面 §63 与 C6 实施任务书 §2） |
| B2-C6 Agent | 主控本地验收通过（Locally verified；回滚点见本节 Git 历史） | [有限纯计算 Base 实现](../kernel/crates/oclive_kernel_runtime/src/domain/base_agent.rs)、[外部调用闭环测试](../kernel/crates/oclive_kernel_runtime/tests/base_agent.rs) | `ScalarCountAgent` **只承接唯一任务文本** `请统计材料中 Unicode 标量值的个数`（`trim` 后整句完全相等才匹配）；`context` 是**被计数的材料**（非空不拒绝），`None` 报「未提供计数材料」、`Some("")` 正确计为 0；`Unsupported` 是**唯一**错误来源（其余四类 N/A）。不接 `slot_runner`／`AppState`／ChatPro／工具／模型／MCP；旧 `AgentProvider`、`handled`、MCP 与权限链未改，也**未**因此证明兼容 |

**推进口径**：各片已按独立本地提交推进，当前统一止于 [阶段总收口](#six-slot-stage-closure)，不再自动进入“下一槽”。该顺序是工程推进顺序，不是 Kernel 执行流水线。生产接线、旧接口整体迁移、ChatPro 接入与远端 CI 结论都不由本节点推断。

**B2-C2 主控验收（2026-09-17）**：实际重跑 runtime 的 `b2_c2_` 单测 10 项、`base_memory` 集成测试 3 项、`domain::memory_engine::tests` 14 项（与前述 10 项有重叠，不相加作独立用例数）、`domain::memory_retrieval::tests` 2 项，以及 B1 `base_only_fixture` 16 项，均通过；`cargo test --locked --offline --workspace --doc -j 1`、runtime `clippy --all-targets -D warnings`、fmt、分层/module-compat/stale-paths/doc-registry/Markdown 链接与 diff 检查通过。runtime rustdoc 构建通过并实查公开模块/类型页面、边界正文和链接；保留 5 条未改文件上的既有警告，不记为零警告。生产匹配 helper 的共享调用、原文/顺序/重复保留和受保护文件哈希经源码复核。worker 的 runtime 全量 211 项记录仅作补充，未冒充主控重跑；未运行真实 I/O、模型、Host 产品链或网络探测。本片不证明语义检索、持久化恢复或旧 Host 已迁移。

**B2-C3 主控验收（2026-09-18）**：复核 §58 返修版本并实际重跑 runtime `b2_c3_` 8 项、`base_prompt` 外部测试 6 项、旧 `domain::prompt_builder::tests` 42 项、`base_memory` 外部测试 3 项与 B1 `base_only_fixture` 16 项，均通过；显式复用同一实例的成功→Unsupported→再成功测试已落实。`cargo test --locked --offline --workspace --doc -j 1` 通过：contracts 11、host 2、runtime 6、types 16、validation 3；本片模块示例实际运行并严格断言，类型示例仅编译，不能把 runtime 6 项都算成本片运行证据。保留 validation wasm 的 cdylib 不支持 doctest 提示，不计为已执行测试。

runtime `clippy --all-targets -D warnings`、fmt、分层/module-compat/stale-paths/doc-registry、本页 Markdown 链接与 diff 检查均通过；runtime rustdoc 构建及公开模块/类型页面的示例、边界正文和互链实查通过，5 条警告均在未改文件。主控核对生产逻辑与公共面没有随返修扩大，§58 对此前编译/执行及调用序列证据的更正成立；桌面旧全文前缀核对一致。本片只证明有限参考实现与独立调用，不证明任意自然语言组装要求、抗提示注入、Host 产品链或真实模型效果；未运行 Host 全量、宽 CI、模型/服务/网络/数据库实验，未 push，冻结项不变。

**B2-C4 主控验收（2026-09-19）**：复核 §61 返修版本、实际五文件与受保护哈希。初次验收发现的共享 UTF-8 panic 是本片阻塞，换成安全材料不能替代修复；本次已恢复原混排、引述与条件反例。`is_negated` 在中文窗口规则未短路时调用英文 helper，该路径不按 `MatchMode` 排除中文命中；修复把同一分隔字符的起始字节位置加一，改为加该字符的 UTF-8 长度。共享修复独立提交为 `dd430a15a4af53b1defaebf1bbedf4072eb0290d`，不依赖新 Base 类型；词库、分隔谓词、否定 token、中文窗口及旧七维映射未改。

**主控本轮实跑**（Cargo 测试均为 `--locked --offline -j 1`）：runtime `--lib b2_c4_` **17 项**（含共享 UTF-8 的 4 项）、`--test base_emotion` **7 项**、`--lib domain::lexicon::tests` **18 项**（与前项有重叠，不相加）、`--lib domain::emotion_analyzer::tests` **10 项**、`--test base_memory --test base_prompt` **3 + 6 项**、contracts `--test base_only_fixture` **16 项**全部通过。另外，将上一轮仓库外的同一份 8 项诊断重新链接到本轮 Cargo 生成的真实 runtime/contracts/types 库后运行，结果由上一轮 **4 passed / 4 failed** 变为 **8 passed / 0 failed**；不是重用旧 exe，也不是复制 helper 的模拟。worker 的修前红测单列为 §61 历史证据，不倒填成主控本轮执行。

`cargo test --locked --offline --workspace --doc -j 1` 通过：runtime **8**、contracts **11**、host **2**、types **16**、validation **3**；本片模块示例（line 83）实际运行并严格断言，类型示例（line 177）仅编译，保留 validation wasm 的 cdylib 不支持 doctest 提示。runtime `clippy --all-targets -D warnings`、fmt、分层/module-compat/stale-paths/doc-registry、本页 Markdown 链接与 diff 检查均通过。rustdoc 构建及公开模块／类型页的正文、示例和链接实查通过，**5 条既有警告**均在未改文件，不记为零警告。

**证据校正与止点**：桌面 §61 终态及其前 1124849 字节与 §60 的哈希均已核对；历史材料保留，源码／实际命令优先于回执中的简写。§61 中“B1 fixture 未在本轮重跑”与其命令 #8 冲突，主控本轮已独立跑过 16 项；额外 Host target 不属于 R1 指定验证集合，不作为扩大覆盖范围的依据。本片证明的是有限词表参考实现、新旧公开入口的定点回归及局部借用，不证明完整情绪理解、主体／引述／条件判断、真实模型或 Host 产品链。未接 ChatPro、未运行 runtime／Host 全量、宽 CI、模型／服务／网络／数据库实验，未 push、未删除任何目录或恢复件；B1、C1–C3、Host、MODULE_MAP 与冻结项不变。Event／Agent 尚未启动。

（以上为 **C4 验收当时**的记录，其中“Event／Agent 尚未启动”是当时的真实状态；C6 之后的当前状态见本节逐槽表与之下的 C6 记录。历史段落不因后续片而改写。）

**B2-C6 Agent 限定实现与自检（2026-09-19，worker 记录 · 待主控验收）**：本片按主控准入实现 `ScalarCountAgent`（`runtime/src/domain/base_agent.rs`），并把原「后续 Event / Agent」行拆为 C5（候选未准入、未实施、本轮停止该线）与 C6 两行。实现只承接**唯一任务句** `请统计材料中 Unicode 标量值的个数`（`str::trim` 后整句完全相等；内部改写、句号、其它单位、附加要求、引述/前后缀一律 `Unsupported`）；`context` 是**被计数的材料**，`Some("")` 计为 0 并与 `context=None` 的「未提供计数材料」报告**分开**；空任务返回「任务为空」且不做替代计算；计数为 `chars().count()`，**不 trim、不规范化、不按字节/UTF-16/字素/词**。

- **本片实测（返修后，cwd `E:/OCLive/oclivenewnew`，Cargo 均 `--locked --offline -j 1`）**：`--lib b2_c6_` **6 项**通过（含单位与不规范化、精确任务匹配、空任务/缺材料/空材料三分、材料作为数据、报告措辞与 `detail` 不回显，以及改名后的 `b2_c6_repeated_independent_calls_do_not_carry_state`）；`--test base_agent` **6 项**通过（外部 `&dyn AgentBase`、**成功路径的局部 task 与材料 `String`/`format!` 借用**、真实 poll、同实例五步序列）；`--workspace --doc` 的 runtime **11 项**（本片 2 个运行示例 + 1 个仅编译类型示例）、contracts **11 项**、host **2 项**、types **16 项**、validation **3 项**通过；`clippy --all-targets -D warnings`、`fmt --check`、`git diff --check`、`cargo doc`、doc-registry 与本页 Markdown 链接通过。相邻 Base 与 B1 夹具在 C6 原片跑过，本轮**未重跑**，不登记为返修后 PASS。
- **证据归属更正（R2）**：内测**不再**声称「同一实例被复用」——`drive` 每次绑定新的实现值，内测只证明**独立多次调用不串用前次结果**；**同实例证据由外部五步测试**（`b2_c6_public_path_same_instance_across_five_calls`）承担。外测顶部原写「task/material 从不使用 static literal」**不成立**（成功计数原本传常量 task），已改为「包含局部 task/material 借用验证」，并在成功计数用例中用局部 `String`/`format!` 构造 task 与材料真实 `execute/poll`。
- **执行范围偏差（如实登记）**：本片在授权清单之外**额外执行过** `cargo test -p oclive_kernel_runtime`（runtime **整 crate**：242 单测 + 6 个集成 target），而原 C6 任务 §7 **禁止 runtime 全量**。该行为**已发生、不抹去**，但**不计作本片授权验证覆盖**，也不作为扩大验收的理由；详见桌面 §64 之后的返修记录。
- 首轮运行有 **2 项断言失败**：`"aé😀"` 的字节数（期望写 8，实际 7）与支持任务句的标量值数（期望写 33，实际 21）。其中**字节数一处源自原计划 §6 的 8 字节记载本身写错**（实际 1+2+4=7），worker 改正期望正确；两处均按**实际值**改正期望，**未**改实现、**未**放宽断言。

**唯一错误来源**：`Unsupported`（trim 后非空且不等于支持句）；`Failed`/`Unavailable`/`Cancelled`/`TimedOut` 本实现**无来源，N/A**，不加自检故障、不加 cancellation 协议、不用 `catch_unwind` 包装。**未接线**：`slot_runner`／`AppState`／插件路径／ChatPro／MCP／模型／工具均未接；旧 `AgentProvider`、`handled`、MCP 与权限链未改，也**未**因此证明其兼容。本片只证明**一个有限纯计算委托可被独立兑现**，不证明工具型 Agent、Hermes、远端 Agent 或六槽完工；Event 仍为未落实项。

**B2-C6 主控验收（2026-09-19）**：R1–R4 的实现／测试／公开文档修正通过；C4 被误删的 352 字符历史段已与基线逐字比对恢复。主控独立运行 C6 单测 **6**、外测 **6**、workspace doctest（runtime **11**／contracts **11**／host **2**／types **16**／validation **3**，C6 为 **2 运行 + 1 compile**）、runtime clippy `--all-targets -D warnings`、fmt、rustdoc、doc-registry、本页链接与 diff 检查，均退出 0。公开页新边界、示例及模块链接已实查，**5 条既有 rustdoc 警告**均在未改文件。B1 夹具、相邻 Base 和 layering/module-compat/stale-paths 沿用 worker 原片记录，未登记为主控本轮重跑；主控未运行 runtime／Host 全量或外部探测。

**验收证据更正与止点**：不采纳桌面 §65.5 由 `domain/mod.rs` 注释推断 runtime 全部单测“纯内存、无网络/服务/子进程”的结论；`src/app_data_migration.rs` 的单测已有临时文件写入，整 crate 的副作用不能由 domain 注释证明。此前越界全量运行继续如实登记，legacy 复制是否当时发生仍未验证，不以验收消除该未知。§65.6 包含 rustdoc 的“生产块逐字不变”及单个终态哈希不足以证明前后相同：主控在内存中逆向还原返修前文件，完整 SHA256 命中 `389167BE…092`，再比对确认**非注释生产代码逐字相同**；计数实现未变，formatter 的 rustdoc 确实按 R3 修改。桌面 §65 及其前 **1243883 字节**已核对；不要求再为报告措辞返修实现。本片只验收有限纯计算实现，不接 Host／ChatPro、不发布 Stable API，C5 Event 保持未准入；未 push，未删除目录或恢复件。

**以上 C6 记录中的 Event 未落实/未准入为 C6 验收当时状态；当前 C5 状态见逐槽表及下文。**

**B2-C5 Event 组合实现与离线结构自检（2026-09-19，worker 记录 · 待主控验收；真实分析质量未验证）**：本片按主控准入实现 `LlmEventAnalyzer<'g>`（`runtime/src/domain/base_event.rs`），把 `EventBaseRequest` 的 `material`/`context` 组织为固定分析指令 + 固定引导行 + **一个只含 `material`、`context` 两成员的 JSON 对象**（`serde_json` 构造，`None`→`null`、`Some("")`→空串、其余原样），调用所借用的 `LlmBase` **一次**并 await，再按私有首行约定投影。公共面仅模块、类型、`new(generator: &'g dyn LlmBase)` 与 `impl EventBase`；**无**访问器/Default/Clone/builder/泛型策略/线程或注册机制，构造器不调用后端、不读环境与资源。

- **可兑现的承诺（离线结构层）**：输入保真（对完整生成输入有**独立手写的整串预期**，另按固定引导行后的**整个余串**解码并断言 `material`/`context` 逐值相等、键集合恰为两者）、每次调用至多一次生成且 Pending 后**不重复发起**、未 poll 的 Future 不触发调用、drop 未 poll 不增次数；`ANALYSIS` 的 LF/CRLF 与正文**原样保留**、`NO_ANALYSIS` 三种合法完整表示 ⇒ `None`；其余形状 ⇒ `Failed` 且说明为固定常量、不回显请求或回复；后端五种 `BaseCallErrorKind` 经真实 await 路径**原样传播**（含 `detail` 为 `None`/空串/误导性 timeout 文本）。
- **借用与异步证据**：外测替身**持有测试函数局部值的真实引用**（`&Cell<usize>` 调用计数、`&RefCell<Vec<String>>` 记录、`&str` 预备回复），其类型因此**非 `'static`、非 `Send`/`Sync`**；该后端的受控 Future **在 Pending 之后再次读取 `LlmBaseRequest.input`**，两次读取一致且解码值对应原材料/context——这才是「生成输入跨 await 存活」的证据。内测替身是**拥有式**脚本后端，只证明调用路径/组织/投影/错误，不冒充生命周期证据；`waker` 仍用 `Arc`+`AtomicUsize`（`Waker::from` 要求 `Send + Sync`，这是对 waker 的约束）。
- **本片实测（返修后，cwd `E:/OCLive/oclivenewnew`，Cargo 均 `--locked --offline -j 1`）**：`--lib b2_c5_` **7 项**通过；`--test base_event` **4 项**通过；`--workspace --doc`：runtime **13**（= 既有 **11** + Event **运行 1**（模块示例）+ Event **compile 1**（类型示例））、contracts 11、host 2、types 16、validation 3；`clippy -D warnings`、`fmt --check`、`cargo doc`、doc-registry、本页链接与 `git diff --check` 见本轮命令结果。B1 夹具、相邻 Base、layering/module-compat/stale-paths 沿用原片记录，**未在本返修轮重跑**。
- **范围表述更正（避免与生产实现矛盾）**：说“**未绑定/未调用具体真实模型、未接 Host 产品链**”。本实现**确实**委托 `LlmBase::generate`，生成调用本身存在；未验证的是**具体真实模型**的质量与稳定性、具体绑定是否胜任该分析任务。
- **局限与未验证分开陈述**：**已确认局限**——响应解析**不含任何语义校验**，结构合法但颠倒否定/替换主体/把计划写成事实的回复仍会被投影（测试中有意保留该反例，记为未检测风险，不是“负例通过”）；**未验证**——真实后端产生这类错误回复的频率、真实分析质量与稳定性、`NO_ANALYSIS` 判断是否诚实。替身预写回复只给 A 类证据。

**B2-C5 主控验收（2026-09-19，限离线结构）**：R1 的真实局部借用、跨 Pending 输入读取、独立完整输入预期、JSON 边界及错误传播已实读复核；主控在该 Rust 版本独立运行 `b2_c5_` 单测 **7**、`base_event` 外测 **4**、workspace doctest（runtime **13**／contracts **11**／host **2**／types **16**／validation **3**，Event 为 **1 运行 + 1 compile**）、runtime Clippy、fmt 检查、rustdoc 与文档检查，均通过。公开页面与模块链接已实查，**5 条既有 rustdoc 警告**均在未改文件。随后 R2 只修文档；主控最终复算三份 Rust 哈希不变，核对 C6 **2991 字符**连续历史块及 C4 完整历史块与基线一致，并重跑文档注册、本页链接与 diff 检查通过；**没有把前轮 Rust 测试记成本次文档收尾重新执行**。B1／相邻 Base／layering／module-compat／stale-paths 沿用原片记录，不扩大为主控新证据。

**C5 收口范围与止点**：桌面 §70 及其前 **1309624 字节**身份已核对；§69 的格式化命令范围偏差与备份轮次更正保留，事后内容相符不独立证明历史动作时序。验收只覆盖该参考实现的输入组织、委托、语法投影、错误透明与独立借用调用；**真实模型分析质量、具体后端胜任性及 Host 产品链均未验证**，不宣布 Event 语义质量合格、Stable API 已发布或小 Kernel 全部完工。未接 ChatPro，未运行 runtime／Host 全量、宽 CI、模型／服务／网络／数据库实验，未 push；不改变 MODULE_MAP、公共契约或冻结项。

<a id="six-slot-stage-closure"></a>

### B1 + B2 阶段总收口（2026-09-19）

**结论：本轮“边界收敛 → 独立 Base 绑定 → 六槽有限参考实现”阶段已本地收口，状态为 Locally verified。** 源码核对基线为 `e8b74b01b4080fddd51d0d003bf36df3247c70e7`；本次总收口只整理文档与复验既有独立调用，不修改 Rust、现行 wire 或产品行为。不以这一结论宣布全局最小性、全部实现合规、Stable 发布或物理拆分完成。

| 层次 | 本阶段交付与证据入口 | 收口限度 |
|---|---|---|
| 职责与最低语义 | [MODULE_MAP §0.1–§0.9](MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)；已决项、候选与历史源码证据保持各自效力 | 没有发现需重开既有权责的新反例；不将所有候选文字升级为已发布兼容规范 |
| 独立 Rust 公共调用面 | [B1 代码、夹具与验收](#six-slot-b1-closure) | 已有可编译调用面；不是旧端口／wire 的替换，不等于独立最小 crate 或零外围依赖发行包 |
| 六槽有限参考实现 | [逐槽表及各片验收](#six-slot-b2-adaptation) | 六槽均有具体实现与对应本地证据；只在各自声明范围内验收，不表示六个全功能模块，Event 特别限于离线结构 |
| 旧 Host 与外围系统 | [源码职责对照](MODULE_MAP_AND_HANDOFF.md#kernel-source-map)、[技术债](TECHNICAL_DEBT_INVENTORY.md) | 本阶段没有完成旧 Host 迁移、产品接线、恢复治理或真实后端质量验收；不因总收口解冻 |

**跨槽复核结果**（本次读源码和既有断言，不新增公共规则）：

- 六个请求类型与六个单方法 trait 继续分开；共同绑定／错误以 B1 源码为准。无完整 Role、领域状态或统一权限句柄被加入基础请求；也不禁止 Host 显式提供领域文本。未调用、正常空内容与失败的已有区别未被参考实现的默认值覆盖。
- `BaseCallFuture` 的借用、动态调用与非 `Send` 返回约束仍如实公开；不把它写成跨线程保证、取消协议、invocation 终态或新的 Authority 系统。已知错误保留与实际副作用、重试安全分别判断。
- Event 借用 LLM 是 **`LlmEventAnalyzer` 的具体组合**，不是所有 Event 实现必须依赖 LLM，也没有引入 Emotion 前置要求。Prompt 的空 requirements、Emotion 的非空 context 拒绝、Memory 的字面匹配、Agent 的单一计数任务，以及 C1 的 Ollama 参数／完成判据，均为各自实现约定，不提升为 Kernel Base 限制。
- C1 是独立具体适配器而非生产装配；C2–C6 不因拥有 `impl ...Base` 就证明旧 Host 已迁移。公开类型可调用、任务语义兑现、产品可用与外部效果是不同验收问题。旧方法未因新 Base 增加而被删除，未把参考实现的私有输出约定设为统一协议。

**后续事项与重新开工条件**：

| 事项 | 当前处理 | 何时另开工作 |
|---|---|---|
| 真实后端／分析质量 | 未验证，尤其 Event 的格式投影不能证明主体、否定、条件及 `NO_ANALYSIS` 判断正确 | 选定具体绑定、质量目标、数据和调用授权后做限定验证；不因此重开六槽定义 |
| 旧 Host 适配、资源／权限包装和产品回归 | 未接线；不默认迁移，也不以源码能编译替代产品验收 | 维护者选定具体消费路径与允许的行为变化后单独立项 |
| 公共发布、兼容版本和可选增强互通 | B1 已有实现；完整版本／协商／发布规则仍未定稿，未发布 Stable | 实际第三方发布或扩展互通需要时明确兼容责任；不预建注册表／协商框架 |
| 物理打包／拆分及外围债务 | `K-CORE-BOUNDARY-01` 继续 OPEN；Host 恢复/context、Event Stream/R7、R1–R4 继续冻结 | 维护者独立授权；不把这些工作自动塞进本阶段，也不把延期当作风险消失 |

**本轮同基线复验**（cwd 为仓库根；源码基线同上；以下是本轮新执行，不沿用 worker 数字）：

| 命令 | 结果与证明范围 |
|---|---|
| `cargo test --locked --offline -p oclive_kernel_contracts --test base_only_fixture -j 1` | **16 passed**；B1 独立调用、错误、借用与增强边界的受控夹具 |
| `cargo test --locked --offline -p oclive_kernel_runtime --test base_memory --test base_prompt --test base_emotion --test base_event --test base_agent -j 1` | **3 / 6 / 7 / 4 / 6 passed**，按命令中 target 顺序列出；五个有限实现的独立外部调用，Event 使用替身，不证明真实模型质量 |
| `cargo test --locked --offline -p oclive_kernel_host --test base_llm_adapter -j 1` | **1 passed**；仅构造与公共借用调用，Future 不 poll、不发请求，不证明真实生成、超时或服务可达 |
| 文档门禁 | `check-doc-registry.mjs`、`check-markdown-links.mjs`（仅本轮六份文档）、`check-stale-paths.mjs --docs-only`、`check-doc-mirror.mjs`、`git diff --check` 均 exit 0；新增中英文导航另做人工对应核对，不以镜像门禁替代语义审查 |

合计 **43 项**来自上述互不重叠的 target，只说明这组定向回归通过，不是六槽全部能力、整个 workspace 或产品的覆盖率。各片单测、doctest、Clippy、rustdoc 与更广回归仍按原验收段的日期和执行者读取；本次纯文档未重新运行这些项目，也未运行 runtime／Host 全量、宽 CI、真实模型／网络／数据库实验。未变更公共 API，故不以文档收尾触发 G8 的新一轮 API 验收。

**止点与防漂移**：本阶段不再无边界找样本或重复测绘。只有新的源码证据或具体使用需求足以挑战既有公共承诺时才局部重开；普通实现限制留在实现说明，外围问题追到足以划界即止。历史验收段保留原文与证据归属，当前状态以本节及逐槽表为入口。跨机器接手使用已跟踪源码／文档／Git 历史；忽略的本机计划和桌面交接是补充记录，不是调用或理解 Base 的必备输入。未 push，不提供当前远端 CI 结论。

### CP-B3-C1：ChatPro 非流式主生成的受检完成语义绑定（2026-09-20 · 主控验收通过 / Locally verified）

**范围**：把 **builtin 非流式文本生成**的具体承接对象从裸 `SharedOllamaClient` 换成 Host 私有绑定 `LlmWithCheckedCompletion`（`kernel/crates/oclive_kernel_host/src/infrastructure/base_llm_binding.rs`，**新增、`pub(crate)`**），仍位于原 `CoordinatedExternalLlm(OLLAMA_ADAPTER_ID)` 观察包装之内。`Some(self.llm)` 外部注入分支、观察 guard、Performance primary/gate/controller 与 fallback 层级、`slot_runner`/`post` 编排、流式入口、wire、types/contracts、B1 与 MODULE_MAP **均未改**。

**真实写入集（CP-B3-C1-R1 校正）**：本片累计 **7 个文件**——6 个已跟踪修改（`base_llm_binding` 之外的五处源码 + 本页）加 **1 个未跟踪新文件** `base_llm_binding.rs`（新文件的行数单独计，不并入 `diff --numstat`）。以本片收尾时实测为准：已跟踪 `git diff --numstat` 合计 **112 增 / 49 减**（`base_llm.rs` 63/45、`README.md` 22/0、`ollama_client.rs` 15/0、`app_state_builder.rs` 10/3、`llm.rs` 1/1、`mod.rs` 1/0；其中 `llm.rs` 仅 `log_ollama_metrics` 私有→`pub(crate)`，`mod.rs` 仅一行模块声明）；桌面 §76 记录的是同一未提交写入集在更早测量点的 **95/44**，两者都以各自时刻的实测为准，不互相折算。新文件 `base_llm_binding.rs` **914 行 / 38384 字节**（含测试模块），不计入上述 112/49。

**范围的准确边界（CP-B3-C1-R1 校正）**：新绑定**只改 `generate_with_opts`**。`generate`、`generate_tag`、`generate_stream`、`generate_stream_with_opts`、`startup_probe`、`supports_prefix_cache` **全部按原具体实现委托**——最初把 `generate` 也套上新检查属于超出准入，已恢复（`generate` 还有其他调用者，例如协调 wrapper 的 fallback generate，以及 Agent 等直接 generate 面）。此处 `generate_stream` 与 `generate_stream_with_opts` 是**两个独立入口**（各自对应带回调的流式具体方法），不是一组「四条方法」，两者的委托都逐项核对过、未合并计数。

**改动前的事实（CP-B3-C1-R2 校正）**：改前的生产对象 `SharedOllamaClient` **自己就覆写了** `generate_with_opts`（`infrastructure/llm.rs:230`），转发到 `<OllamaClient as LlmClient>::generate_with_opts`（`llm.rs:123`）：该方法**每次调用**读 `llm_params::main_chat_options()`、用 `b2_c1_merge_options` 合并、调用 `generate_with_settings`、失败经 `into_legacy_error` 保留原 `AppError`、成功则 `log_ollama_metrics` 并返回 `LlmGenerateOutcome { reply, prompt_eval_ms }`——**它不检查 `done`**，metrics 也不是本片新增。`LlmClient` 的 trait 默认实现（`contracts/llm.rs:110`）虽然直接构造 `LlmGenerateOutcome { reply, prompt_eval_ms: None }`，但改前的生产链**从未走到它**。因此本片的实际改动是：在同一观察包装内替换承接对象，并在 with-opts 分支**新增 provider `done` 完成判据**（参数合并、传输、metrics 记录与错误路径沿用旧链）。另需分清：固有方法 `OllamaClient::generate_with_opts`（`ollama_client.rs:476`，返回旧 `OllamaGenerateResult`）与上面的 trait 覆写是**两个不同入口**，不可混为一谈。

**唯一批准的语义差异**：`generate_with_opts` 的受检投影按 provider `done` 判完成——`done=true` 的文本（含空串与纯空白）原样成功并携带同次 metrics；`done=false`（无论有无部分文本）**不是正常结果**，投影为 `AppError::OllamaError`。旧行为保持原样：`<OllamaClient as LlmClient>::generate_with_opts` 仍直接返回 `out.body.response`（不看 `done`），固有 `OllamaClient::generate_with_opts` 仍经 `into_legacy_result`/`into_legacy_error`，流式入口未动——这是有意的兼容边界，不是遗漏。

**一次执行、两个视图的真实调用图**：`LlmWithCheckedCompletion::generate_with_opts`（**调用方**每次调用读 `llm_params::main_chat_options()` 并把该采样对交给 helper）→ `checked_generate_with_opts`（**生产共享路径**：只合并一次、把 merged settings 借给可替换的传输步骤）→ `ProductionCheckedCall` → 既有 `OllamaClient::generate_with_settings` → `project_checked_call`。Base 侧共用的是**底层完成判据** `OllamaCallResponse::project_base`：C1 走 `project_ollama_call` → `project_base`，Host 走 `project_checked_call` → `project_base`——Host **没有**调用 `project_ollama_call`，两者共用的是同一次执行、同一个 `project_base` 判据与同一套传输（`generate_with_settings`），不是同一个高层函数。文本、metrics、完成事实与错误来自**同一次**调用，不存在为凑字段而二次生成。

**本片撤回的旧结论（CP-B3-C1-R1 / R2 收尾）**：上一版本节、桌面 §76 与 §77 的下列说法**不成立，已撤回**——①「`generate` 的改动已获批准」（未获批准，属超范围，已恢复为原委托）；②「T4 已证明生产参数隔离/每请求一次」（上一版 T4 断言的是**替身自造**的参数，未证明生产共享路径收到的值，故不构成证明）；③「Host 视图调用 `project_ollama_call`」（Host 走 `project_checked_call`，两者只共用底层 `project_base`）；④「测试用 helper 即生产 helper」（生产共享路径是 `checked_generate_with_opts`，测试只替换其末端传输步骤，测试替身不是生产 helper）；⑤「内部目标已足够完成全部责任」（本片只做组件级与受控替身验证，仍缺端到端与真实传输验证）；⑥「本片是把 trait 默认实现换成覆写」（改前生产链走的是 `SharedOllamaClient` 自己的显式覆写，从未到 trait 默认——撤回的是这个因果与措辞，不是 `done` 判据本身）；⑦「跨 Pending 借用与每请求进入一次已被证明」（上一版 `RecordingCall` 不持有参数引用、观测值在返回 Future 前已克隆，且只有 poll 计数、没有独立的传输入口计数；R2 收尾改为 future 持有真实引用、每次 poll 分别读取并新增独立入口计数后才成立）。相应的旧 PASS 计数（10/9 项等）属于被撤回的版本，不作为本片证据。

**本片实测（R2 收尾后，cwd `E:/OCLive/oclivenewnew`，Cargo 均 `--locked --offline -j 1`）**：`--lib cp_b3_c1_`（`--list` 10 项，**执行者=本次收尾会话**，运行 **10 passed**；均为本片测试，与下列既有 target 不重叠）：T1 完成与兼容（含 `done=false` 与 Base 投影同时拒绝、旧投影仍成功、缺 `done`/空 body 不是空成功）；T2 **生产共享路径实际收到**的 model/prompt/merged settings（`None`/`Some(default)`/单字段覆盖 + 其余选项字段）；T3 经**共享路径**的成功 metrics 两态与合成失败的原 `AppError` 保真（含 `known_timeout` 两态都不改写变体/文本），Base 的 `TimedOut`/`Failed` 映射由独立 Base 投影测试证明，**不**声称 Host 返回 `BaseCallError`（六字段完整性仍由既有 `b2_c1_metrics_cover_all_six_fields_and_both_switches` 承担，本片未重复造）；T4 两请求 `A Pending → B Pending → B Ready → A Ready`，测试 future **持有真实 `&str`/`&settings` 引用**并在**第一次 poll 与恢复后的 poll 分别读取**（记录两阶段值，非 Pending 前快照）、每个请求传输入口计数 1 / poll 2 / wake ≥2、调用后局部 `String` 仍可用；T5 `LlmWithCheckedCompletion: Send + Sync`、共享路径 future 与 `LlmClient` trait future 的 `Send` 编译断言（只构造不 poll）。**证据分层**：共享 helper（T1/T2/T3/T4）、投影单测（Base 映射、旧投影兼容）、源码与编译（接线行、`Send` 断言）、**未验证真实传输**四类分开，前两类都由不发送请求的受控替身驱动。`--lib b2_c1_` **20 passed**（既有 B2/C1 测试，用例由前序片编写，本次会话重跑，与 `cp_b3_c1_` 无重叠）；`--test base_llm_adapter` **1 passed**；`--test base_only_fixture` **16 passed**（均为既有 target，本次重跑，互不重叠；计数只说明这组定向回归通过，不构成产品覆盖率）；`cargo check`（Host 默认 / `--features dual_core` / 桌面 lib）均 exit 0，仅为**编译证据**（真实消费者可编译），不是运行证据；`--workspace --doc` 0；`clippy --lib --tests -D warnings`、`fmt --check`、`git diff --check`、layering/module-compat/stale-paths、doc-registry 与本页链接、`cargo doc -p oclive_kernel_host --no-deps` 均 exit 0（rustdoc 全文 21 warnings，**逐条检索无一条出自本片写入集**；public `infrastructure/base_llm` 页面已实查：含公共适配器、私有绑定说明与「生产组合未被本适配器替换」的措辞，且不再出现已删除的 `into_llm_base`；`base_llm_binding` 的页面按预期不存在）。**负向验证（上一轮，如实留档）**：曾在共享路径合并之后临时注入 `settings.temperature = Some(9.99);`，`cp_b3_c1_shared_path_hands_model_prompt_and_settings_through` **按预期失败**（`left: Some(9.99) / right: Some(0.11)`）；复原依据是**单行逆编辑 + 全文件检索 `9.99` 零残留 + 复跑通过**，**没有** pre-mutation 字节身份对照，故不写「字节相同」或「逐字还原」；本轮未重复该破坏性验证。

**已知影响边界**：该绑定在 Performance 模式下作为 Ollama fallback 时，其 with-opts 分支同样采用新检查；Performance primary 本身未迁移。流式传输未迁移；若流式回合后续改走既有的非流式质量修复，则那条修复路径会经过新检查——这是两件不同的事，分别登记。该 Host 装配可被其他入口复用，本片**不**按 ChatPro 产品名做隔离，也**不**新增发行版开关；公共 `OllamaBaseAdapter` **仍未直接注册**，本片接入的是 Host 私有绑定，**不**宣称公共 Base trait 在产品路径被直接 await。

**未验证**：真实模型/HTTP/流式运行、生产传输的端到端回合、`post::run_main_llm` 的完整调用（本片只做组件级与受控替身验证），以及任何模型质量结论。**未迁移**：流式路径、其他 LLM backend（remote/directory/performance primary）、其他五槽。

**主控最终验收（2026-09-20）**：复核 R2 实际源码、生产装配与委托、七文件累计 diff 和保护身份后通过本片验收。独立复跑 `cp_b3_c1_` **10**、`b2_c1_` **20**、`base_llm_adapter` **1**、`base_only_fixture` **16** 项通过（四组不重叠）；`--workspace --doc` 通过（contracts 11 / host 2 / runtime 13 / types 16 / validation 3，含编译示例，不全部算运行断言）。Host 默认／dual_core／桌面 lib 的 `cargo check`、Host lib/tests Clippy、fmt 检查、分层、module-compat、stale-paths 均通过。上述是主控本轮实跑，不倒填早先版本；构建检查不等于产品运行。公开 rustdoc 构建通过，21 条警告均位于未改文件，不声称全仓无警告。

**证据口径补充**：缺 `done`／空 body 的用例直接验证生产 decoder，并未执行完整共享等待路径；借用跨 Pending 的证据来自 `RecordingCall<'p>` 持有引用并在每次 poll 读取，而非仅凭调用后 String 仍可使用。两请求的 wake 断言是共用计数器合计至少 2；每请求入口次数 1、poll 次数 2 分别断言。上文统计与未跟踪状态为 worker 交付快照，最终提交范围以 Git 为准；Host／桌面 check 等沿用项已由本轮主控重跑。

**收口与回滚**：本片只让已批准的 builtin 非流式 with-opts 装配执行完成检查，Kernel Base 公共面、旧 generate/tag/流式委托和其他槽未扩张。作为独立本地提交收口，可用该提交的 `git revert` 回退；提交前基线为 `34d483febbef231d77b6b5dbf767c2af67c509ef`。未 push，未执行真实请求、完整 AppState／整回合、数据库实验或质量试跑；没有远端 CI／全量发布结论，不自动开启下一片。

### CP-B3-C2：builtin Emotion 具体实现的 Base 能力适配（2026-09-21 · Implemented / 本片 Locally verified（待主控验收））

**目标**：让参考 Host 已注册的内置情绪分析具体类型 `BuiltinUserEmotionAnalyzer`（`plugin_backends.emotion = builtin` 绑定的就是它）除现行七维端口外**也能承担 Base 视图**，并与 `KeywordEmotionBase` 共用同一份私有 Base 路径和**同一个**词表分析入口；不改 Kernel 契约、不改 Host 装配、不改产品行为。

**实际改动**（两个源码文件，均属 `oclive_kernel_runtime`）：

- `domain/emotion_analyzer.rs`：新增 `pub(crate) fn analyze_material(text) -> Result<LexiconSuggestion>` —— **本片所涉及的 legacy 七维入口与两个 Base 入口共用的分析入口**（唯一词表加载 + 单次 `analyze`，错误原样传播）；`EmotionAnalyzer::analyze` 改为取同一 suggestion 经既有 `Lexicon::to_emotion_result` 投影，公开签名、七维与 `extension` 均不变。
- `domain/base_emotion.rs`：抽出私有 `base_clue_report(text, context)`（先执行既有 `context` 判定，再调上述入口，再按 `hits` 判 `None` 或构造既有报告）；`KeywordEmotionBase` 与新增的 `impl EmotionBase for BuiltinUserEmotionAnalyzer` 都委托它，不复制词表、匹配、报告渲染与错误映射；模块文档新增一个可运行例（同一 builtin 实例经两个 canonical trait，显式消歧、严格 `match`）。

**证据与限制**：新增内测统一 `cp_b3_c2_` 前缀共 **10 项**（`cargo test -p oclive_kernel_runtime --lib cp_b3_c2_` → **10 passed**，`--list` 同数），覆盖：具体类型同时满足两 trait 并经 `&dyn`/`Box<dyn>` **真跑**（`Pending` 即失败）；两 Base 实现逐字一致 + 独立逐字报告锚点；无命中/中性命中/全否定三分（七维对三者都是兼容 `neutral` 回退，Base 视图仍能区分，全否定仍算命中）；`context` 矩阵与同一实例复用序列；本地借用；七维与 Base 各自投影。既有回归**只运行不修改**：`b2_c4_` **17 passed**、`domain::emotion_analyzer::tests` **10 passed**、`domain::lexicon::tests` **18 passed**、外部 `--test base_emotion` **7 passed**、B1 `base_only_fixture` **16 passed**。编译/静态/文档：runtime 与 host `cargo check` 0、`--workspace --doc` 0、clippy（runtime lib+tests `-D warnings`）0、`cargo fmt --all -- --check` 0、`cargo doc -p oclive_kernel_runtime --no-deps` 0（该 crate 5 条 rustdoc 警告全部为既有，本片两个文件零新增；新 impl 已在实际生成的 builtin 类型页与模块页核对）。**首轮失败如实记录**：先写的能力测试首轮为 `E0277 BuiltinUserEmotionAnalyzer: EmotionBase is not satisfied`（预期红），实现后转绿；`cargo fmt --check` 首轮 1 处格式差异，已只在本片两文件内格式化。

**边界（不得扩大）**：这是"现有 builtin 具体实现可经 Base 或现行七维端口调用"的**适配基础**，不是迁移——参考 Host 仍绑定 `Arc<dyn UserEmotionAnalyzer>` 并只消费七维，**没有任何产品消费者读取 Base 报告**。两个入口各自调用即**两次分析**，没有单次调用双结果，也没有缓存、最近结果或去重；七维不从报告反推、报告不从七维生成。共同的是分析入口：legacy 七维用 `Lexicon::to_emotion_result` 投影，两个 Base 实现共用 `base_clue_report`/`build_report`；这不等于"七维与 Base 共用同一个投影函数"。`none`/Remote/Directory 未新增 Base 视图、未映射新错误、未补跑 builtin、未改授权与降级；七维数值（含 `neutral` 兼容回退）、Host 装配、领域应用与回退全部保持。**未迁移**：其他后端、其他五槽、流式与外围治理。**未验证**：真实模型/HTTP/服务、完整 `AppState` 与整回合（桌面目标需 loopback 网络与完整宿主，本轮 NOT RUN）、产品侧 Base 消费。**待主控验收**，未 commit/push。

**主控收口（2026-09-21 · Locally verified）**：上述待验收状态为 worker 交付快照，本片现通过主控复核。两个 Rust 文件及两份 CHANGELOG 与前序已审查、定向验证的字节身份一致；C2 记录之前的 README 历史块与原 HEAD 比对一致（统一换行并仅消除末尾空白），C1 验收与回滚段没有被移入 C2。本轮另独立复跑 `cargo test --locked --offline -j 1 -p oclive_kernel_runtime --lib cp_b3_c2_`，**10 passed**；其余已核对的同版本回归、doctest、编译和静态证据沿用，不倒填为本轮重跑。文档收尾按 doc-registry、本页链接与 diff 检查验证。以原基线 `ba3fcaf821ed8dca52864efc710e53f405c90b61` 之上的独立本地提交建立回滚点，未 push；能力增量不等于产品侧开始消费 Base 报告，未扩大上文的未迁移及未验证范围。

### CP-B3-ALL：六槽第二阶段四单元连续实施（P / M / A / E）（2026-09-21 · Implemented / 局部离线验证完成（待主控验收））

**基线与状态**：本批唯一批准起点是 C2 收口后的 HEAD `54b5d7a4aa454bf0e0137ab7486d5f1a65f58b5b`（提交标题 `feat(runtime): expose builtin emotion Base capability`），开工时工作树干净；开工复算的保护面身份（`handoff/README.md` `17BD02F1…4335`、`MODULE_MAP_AND_HANDOFF.md` `4BDB7900…DFD6`、`types/src/slot_base.rs` `97CEE6BF…1BA`、`contracts/src/slot_base.rs` `8A0064CE…D11`、`contracts/tests/base_only_fixture.rs` `785D6AEC…C77`、`runtime/.../emotion_analyzer.rs` `8E94F702…ADD`、`runtime/.../base_emotion.rs` `D774E15F…DBF`）全部相符。四个单元（P Prompt → M Memory → A Agent → E Event）已完成可离线验证部分；状态只写 **Implemented / 局部离线验证完成 / 待主控验收**。本页只**追加**本批记录，未改写上文任何历史结论（包括 C2 之前的"当时未接"与"待验收"表述）；本批未 commit、未 push、未做真实产品试跑。

**唯一授权范围内的产品差异**：本批只批准两处有限产品行为变化——M 的"按当次检索需求筛选原件"与 E 的"同次模型请求增加 `analysis` 成员与叙述要求"。P 为字节等价重构；A 的产品输出、调用序列、持久化与回落策略不变。LLM C1 与 Emotion C2 的**已收口范围受本批保护**，本批只对它们做回归；旧插件协议、权限/恢复/幂等、DB/迁移/前端/流式、Event Stream 与 R7、R1–R4 全部未动。

**逐单元实际调用图（旧 → 新，同一具体执行）**

| 单元 | 旧路径 | 新路径 | 产品可见差异 |
|---|---|---|---|
| P | `BuiltinPromptAssembler::build_prompt` → `PromptBuilder::build_prompt` 就地拼接；Base 侧 `LiteralMaterialAssembler::assemble` 自拼 | `build_prompt` → `prepare_prompt_blocks` → `concat_prepared_text`；`PromptSegments::full()` 用同一连接核心；`assemble_literal_material` 与新增 `impl PromptBase for BuiltinPromptAssembler` 委托同一函数 | 无（改前五组输入逐字节摘要复算不变） |
| M | `plugin_backends.memory = builtin` → `BuiltinMemoryRetrieval::rank_memories`（全候选加权排序） | 私有工厂 → `QueryMemoryRetrieval`：`select_indices`（唯一核心）→ 命中时仅把**原件**交给既有 `MemoryEngine::get_relevant_memories` 排序 + `limit`；无命中时保留原全候选加权选择（Host 兼容回落，不算 Base 命中） | **有**：命中时检索范围收窄；原件字段与 `build_context`/`search_memories` 不变 |
| A | `BuiltinReActAgent::process` 内含整个 ReAct loop | `process` → `execute_react`（唯一核心，返回既有 `AgentOutput` + 真实分支事实）；新增 crate-private `HostAgentBaseView::execute` → 同一 `execute_react`，只读叙述投影 | 无（loop 上限 3、schema、prompt、模型来源、工具顺序、trace 保存与 40 条裁剪、fallback 文本、`handled` 与空 reply、失败后继续、`Err` 传播、拒绝与回落全部保持） |
| E | `BuiltinEventEstimator::estimate` → `estimate_event_impact` → 规则预估 + 一次 `generate_tag` + 旧解析 + 后处理 | `BuiltinEventEstimator::estimate` 与 `estimate_event_impact` 都 → `estimate_event_impact_core`（规则预估一次 + 至多一次 `generate_tag` + 同一共享解析结果 + 同一后处理）→ crate-private `EventImpactCore { estimate, analysis }`；新增 crate-private `HostEventBaseView::analyze` → 同一核心，只读 `analysis` | **有**：同一次 prompt 增加 `analysis` 成员与叙述要求（真实模型数值/失败率可能改变）；数值解析与后处理策略不变。措辞更正（CP-B3-ALL-R1）：这里说的是两个投影读**同一份解码结果**，不是说 JSON 解析只尝试一次——该解码自己先做直接解析、失败再走既有宽松提取 |

**证据分层与实测**（cwd `E:/OCLive/oclivenewnew`，Cargo 一律 `--locked --offline -j 1`；下列命令均为本批收尾会话实跑，退出码全部 0；新 prefix 先 `-- --list` 核对条目名，`0 tests` 不计通过）：

| 命令 | 结果与证明范围 |
|---|---|
| `-p oclive_kernel_runtime --lib cp_b3_all_prompt_` / `--lib domain::prompt_builder::tests` / `--test chatpro_base_prompt` | **5 / 44 / 2 passed**；连接核心、改前 golden 摘要、公开两入口 |
| `-p oclive_kernel_runtime --lib cp_b3_all_memory_` / `--test chatpro_base_memory` / `--lib domain::memory_retrieval::tests` | **7 / 2 / 2 passed**；选择核心、两个 trait 视图、公开双视图外测与既有回归 |
| `-p oclive_kernel_host --lib cp_b3_all_memory_` | **3 passed**（`slot_runner` 单实例消费者 1 + 合并策略 1 + `backend_registry` 私有工厂 1） |
| `-p oclive_kernel_host --lib cp_b3_all_agent_` / `--lib domain::fallback_agent::tests` | **16 / 3 passed**；两者重叠 1 条（`cp_b3_all_agent_authorization_denial_does_not_fall_back`）。独立条目为 agent 模块 **15** + `fallback_agent::tests` **3** = **18**（CP-B3-ALL-R1 更正：先前写的 15 + 2 = 17 把 fallback 两条既有用例漏算在独立条目之外）；重叠条目不重复相加 |
| `-p oclive_kernel_host --lib cp_b3_all_event_` / `--lib domain::event_impact_ai::tests` | **12 / 17 passed**；新分支表 + 既有数值/策略回归（既有 17 条未改） |
| `-p oclive_kernel_runtime --lib b2_c3_` / `--lib cp_b3_c2_` | **8 / 10 passed**；P 未扰动的既有 Prompt 回归与 C2 基线 |
| `-p oclive_kernel_runtime --test base_memory --test base_prompt --test base_emotion --test base_event --test base_agent`（逐 target 单跑） | **3 / 6 / 7 / 4 / 6 passed**；五个有限 Base 实现的既有外部调用 |
| `-p oclive_kernel_contracts --test base_only_fixture` | **16 passed**；B1 独立契约夹具 |
| `-p oclive_kernel_host --lib cp_b3_c1_` / `--test base_llm_adapter` | **10 / 1 passed**；C1 完成语义与既有适配器构造（不 poll、不发请求） |
| `cargo check -p oclive_kernel_host --lib`（默认与 `--features dual_core`）、`-p oclivenewnew-tauri --lib` | 均 exit 0，**仅编译证据** |
| `cargo test --workspace --doc` | 通过：contracts **11** / host **2** / runtime **14** / types **16** / validation **3**。runtime 的 14 与 C2 记录写的 13 相差 1：本批写入集未新增任何 doctest（`query_memory.rs` 无代码块；`base_prompt.rs` 与 `prompt_builder` 的既有 2 + 1 条未变），该差异**无法由历史日志核实原因**，标记为**未核实**，不倒填旧轮输出 |
| `cargo clippy -p oclive_kernel_runtime -p oclive_kernel_host --lib --tests -- -D warnings` | exit 0，**0 条诊断** |
| `cargo fmt --all -- --check` | exit 0（本批只对本写集的 12 个文件单独调用 `rustfmt --edition 2021`，未执行 `cargo fmt --all` 写入） |
| `cargo doc -p oclive_kernel_runtime -p oclive_kernel_host --no-deps` | exit 0；host **21** + runtime **5** 条警告逐条定位后**全部位于本批未改文件**（与 C1/C2 记录数一致）。已实查页面：`domain::query_memory` 模块页与 `QueryMemoryRetrieval` 页存在，`BuiltinPromptAssembler` 页出现 `PromptBase`；两个 crate-private 绑定**没有** API 页，只有 `doc/src/` 源文件视图。**公共面更正（CP-B3-ALL-R1）**：首个版本另有 `fn.query_terms`/`fn.select_indices` 两个公开页——两个 helper 当时被越界声明为 `pub`；本轮收回为模块私有，不得再把"产生公开页"写成原计划已满足的公开面。旧构建的 HTML 不删除（不以删目录消证），公开性以**全新独立 target** 的重建结果为准 |
| `check-domain-layering.mjs`（`rg` 不在 PATH，仅在**单条命令进程内**临时前置既有 `rg.exe`；未安装、未改系统 PATH、未改门禁） | exit 0：`use` 3 ≤ 3、FQ(prod) 1 ≤ 1。首次运行因本批测试模块新增 1 行 `use crate::infrastructure…` 而 4 > 3 失败，已改为测试内全路径引用，未改基线文件 |
| `check-module-compat.mjs` / `check-stale-paths.mjs` / `check-doc-registry.mjs` / `check-markdown-links.mjs handoff/README.md` / `check-changelog-parity.mjs` / `git diff --check` | 全部 exit 0 |

**关键反例与断言位置（不把"跑通"当证明）**：M 的外测最初把 Base 视图与 Host 视图的**顺序**断言为相同而失败——Base 保持原始输入顺序、Host 按既有加权排序，两者只保证**同一集合**（`chatpro_base_memory_host_and_base_views_agree_on_one_selection` 现分别断言次序与集合）。A 的 7 条用例最初在**同一个 agent 上**先跑 legacy 入口再跑 Base 入口而失败（第二次执行把脚本耗尽，`Unknown error: script exhausted`）——这正是"两次独立执行不能证明同次"的反面教材；现改为每个入口各用一套全新替身，并以 `cp_b3_all_agent_one_execution_serves_both_views` 直接驱动共享核心一次、在**同一 carrier** 上投影两视图。E 的"格式合法但颠倒否定/主体的叙述"被解析层**原样接受**（`cp_b3_all_event_semantic_quality_is_recorded_as_unverified`）：本批只证明机制，未证明模型叙述的语义质量。A 的 `tool_refusal` 用例初版只给 1 轮脚本，实际循环在工具失败后继续到第 2 轮（原行为），故补第 2 轮"无 function call"以落到确定性分支。`cargo fmt --all -- --check` 首轮 exit 1（12 个写集文件），clippy 首轮因测试内 `useless_format` exit 101，均已按上表修正。

**跨 Pending 与证据纪律（CP-B3-ALL-R1 更正范围）**：A、E 各有两请求**真正相交于 Pending** 的用例，交错了 A Pending / B Pending 的次序、断言每请求各进入一次、互不串用、恢复后不再生成（`cp_b3_all_agent_intersecting_pending_keeps_requests_isolated`、`cp_b3_all_event_pending_resumes_without_regenerating_or_mixing_requests`）。**以下旧说法已撤回**：最初写"恢复后再次读取真实引用"——返修前那两套替身在**等待前就把 model/prompt 克隆**进记录，恢复后只取拥有式响应，因此它们证明的是"真实 Pending 交错 + 隔离 + 不重发"，**不是**"恢复后仍从真实借用读取"。该证据由本轮返修版本补齐：两套替身分别在**等待前**与**恢复后**各读一次端点 `&str` 参数并分别记账（`before_wait`/`after_resume`），`Gate` 统计每次 poll（含 Ready）并单独统计 wake，入口计数独立；旧的绿测不被否认，只是明确其证明范围。Host 具体 future 的 `Send` 只用编译断言证明，Base 的 `BaseCallFuture`（无 `Send`）**未**被强化、也未被断言。测试替身只出现在末端资源接缝（LLM 端口、MCP bridge 端口、选择/词表核心的直接输入），A 使用**现有纯 parser**，E 使用**现有解码与后处理**，未写同形模拟函数替代关键流程。

**兼容边界（不得扩大，含 CP-B3-ALL-R1 更正）**：四个单元新增的 Base 面（Prompt 的附加 impl、`QueryMemoryRetrieval`、两个 crate-private 请求绑定视图）**都没有产品消费者读取 Base 结果**——参考 Host 仍绑定 `Arc<dyn MemoryRetrieval>`/`Arc<dyn AgentProvider>`/`Arc<dyn EventEstimator>` 并只读旧端口；M 的接线改变的是"内置 memory 选哪批原件"，不是"产品开始读 Base"。**公共面更正**：M 的选择核心与词项 helper 在首个版本里被越界声明为 `pub`，本轮收回为模块私有（外部 target 改经两个 trait 真入口验证），公开面只剩模块、`QueryMemoryRetrieval` 与必要 trait impl。**输入更正**：Agent 的共同核心改为只收四个实际消费字段的私有借用视图（`ReActInput`），Base 路径不再构造 `AgentInput`，因此不再经 `AgentInput::default()` 带入人格向量、关系状态、好感、场景或协议版本等默认领域事实。`HostAgentBaseView` 与 `HostEventBaseView` 目前只由测试驱动（模块声明处带说明性 `#[allow(dead_code)]`，不扩大到共同核心；**不**为消除该提示而新增产品消费者或第二次执行）。E 的新叙述不自动进主 Prompt、不写回领域、不落库、不加 DTO/wire 字段，B2 `LlmEventAnalyzer` 未动。A 的非空 `context` 在执行前 `Unsupported`；E 的非空 `context` 经独立小节进入**同一次** prompt——两者是各自具体适配器的口径，不是 Agent/Event Base 的通用限制。

**未验证 / 未迁移（CP-B3-ALL-R1 更正）**：真实模型、真实 HTTP/网络、流式生成与 `process_message_stream` 的 token/短路次序；完整 `AppState`/Builder/BackendRegistry 初始化与整回合产品路径；数据库迁移与用户库；桌面与前端消费者；注入式 `LlmClient` 的网络授权问题；"原规则步骤本身 `Err`"的传播（现有规则检测在离线输入下不返回 `Err`，仅结构性保留 `?`）。**Event 的已知取消/超时**：本轮只限定"当前被读入口能拿到什么事实、什么已在此前被文本化"，即产品错误类型没有类型化取消/超时变体可用；**不**声称"保留这两类必然要改 Kernel 错误注册表"——私有 carrier 本可保留事实而不改公共枚举。本轮不实现错误治理、不解析 `detail`、不新增错误类别，把已知缺口如实留下。**未迁移**：各自尚未接的限定范围——LLM 的流式与其他 backend、Emotion 的其他 backend、Memory/Agent/Event 的 remote/directory/none 后端、任何产品侧 Base 消费，以及 Event Stream/R7、R1–R4、旧插件协议、权限/恢复/幂等、前端。**LLM C1 与 Emotion C2 的已收口范围受本轮保护**，不属于未迁移项。

**止点**：本批不无边界找样本，不在离线证据之外声称产品效果，也不自动开启下一阶段。回滚点为上述 C2 收口提交 `54b5d7a4aa454bf0e0137ab7486d5f1a65f58b5b`；本批**未** commit、**未** push，主控集中复核后再决定是否提交与是否做真实质量验证。

**主控验收与提交准入（2026-09-21 · Locally verified）**：上文待验收/未提交为 worker 交付快照。本批 P/M/A/E 的共同核心、适配能力与已批准有限行为变化通过集中复核；Memory 两个 helper 已收回私有，Agent 不再经默认 DTO 补造领域事实，Agent/Event 的末端借用跨 Pending 与 Event 独立完整输入预期已具备有效证据。主控在返修最终字节上独立复跑 runtime/Host `cp_b3_all_`（12/32）、外部 Memory/Prompt（2/2）、原 Prompt/Memory/Agent fallback/Event 策略（44/2/3/17）、C2（10）、五个 Base 外测（3/6/7/4/6）、B1（16）、C1（10）及 LLM 构造外测（1），以及 workspace doctest、Host 默认/dual_core/桌面 lib 编译、Clippy、fmt 检查、分层/模块兼容/路径/文档登记/本页链接/CHANGELOG parity/diff 检查，均通过；重叠条目不累计。公开文档构建成功并实查私有 helper 不在新建输出的公开索引中。两点口径以本段为准：Event 交错测试的模型名仍为字面量，不计为局部 String 模型借用证据；rustdoc 26 条既有警告包含本批已改文件的未改行，不称全部来自未改文件。原 20 文件中返修未获准修改的 12 文件、Kernel 契约与历史 README 块均核对未变。维护者已授权建立本地提交；不推送、不改技术债状态。该验收不等于所有产品消费者已切到 Base，不证明真实模型质量、HTTP、完整回合或远程 CI；下一步仅按独立任务推进受控验证。

### CP-B3-V1：相邻消费点离线组合验证（2026-09-21 · 消费者组合离线验证完成／待主控验收）

**基线与范围**：本片基线为 `448c4dc3e14f8fca0b1a35c7ae97a3195bc972ea`（父 `54b5d7a4aa454bf0e0137ab7486d5f1a65f58b5b`，标题 `feat(host): integrate shared Prompt Memory Agent and Event cores`），开工 `git status --porcelain` 为空；任务书 `.cursor/plans/chatpro-base-consumer-verification-v1.plan.md`（13559 字节 / `3608DE5E…431C`）已读至结束。只为本 Host **已接线的一条**相邻消费链（Memory/Emotion → Prompt → 非流式 LLM）补组合证据；Agent/Event 只做"未调用"记录与既有回归；不重复已收口的 Base 独立夹具，不把六槽写成固定流水线，不添加叙述消费者，不为"全六槽"强行调用所有槽。

**写入面**：`slot_runner.rs` 仅在既有 `#[cfg(test)]` 区新增 `cp_b3_v1_` 测试与私有 fixture，**生产方法、既有测试断言与路由策略均未改**；`#[cfg(test)]` 之前的生产前缀 30814 字节在开工与收工**逐字节相同**（SHA256 `C2EECCE138C4A41960B11EB79B3FA7B3D72E61D86DBFBBAC464D982CA8119E58`，同一算法对同一分段计算）。未改 Cargo/lock、契约、MODULE_MAP、技术债状态、CHANGELOG（本片纯测试非能力发布）、配置、角色包、前端、插件与门禁脚本；未新增 crate/target/依赖/公开面。另在本页与桌面末节追加记录，并新建 ignored 进度文件。

**CP-B3-V1-R1 集中收尾（2026-09-21 · 待主控验收）**：本段 6 项测试已由主控在返修前独立复跑通过（`6 passed / 0 failed`，`git diff --check` exit 0），但那次 PASS **不覆盖**下面两项当时缺失的断言，本段也不把它倒填为新覆盖；返修只补测试与措辞，**未改生产**。① **R1 空 query 交接**：原先空 query 只比较 id、其结果没有实际传给 `build_prompt`；现补一次真实 `SlotRunner::build_prompt(&h.plugins, &fixture.input(&empty_query, …))` 并对实际输出断言标题与三个独立标记，计数由 Memory 4 / Prompt 3 / 生成 0 更正为 **Memory 4 / Prompt 4 / 生成 0**，并删去"limit 0 没有东西可喂"这一与实际执行不符的解释（`limit=0` 的空切片确实进了 Prompt 入口，其"无记忆证据"断言正是在该次输出上成立）。② **R2 段边界正反两侧**：原先只证明"世界观在 stable、guardrail 在 dynamic、记忆不在 stable"，**没有**证明"世界观不在 dynamic、guardrail 不在 stable"；现补两个负向断言，既有 `full()`/`stable_len()`、记忆、普通入口与生成交接断言全部保留。两处缺口都是由本轮**实际执行**的新断言补上的（若未执行，`prompt_calls() == 4` 或负向断言会直接失败）。措辞更正见下各条与桌面 §89。

**真实调用图（本轮测试实际走的路径，不是产品整回合）**：

| 消费点 | 测试调用的真实入口 | 实际抵达 |
|---|---|---|
| 选择 | `SlotRunner::rank_memories(pl, MemoryRetrievalInput)` | （`slots: None`）`pl.memory.rank_memories` → 计数包装 `CountingMemory` → **真实** `QueryMemoryRetrieval`（`select_indices` 选择 + 既有 `MemoryEngine::get_relevant_memories` 加权与 limit） |
| 组装 | `SlotRunner::build_prompt(pl, &PromptInput)` | `pl.prompt.build_prompt` → `CountingPrompt` → **真实** `BuiltinPromptAssembler` → `PromptBuilder::build_prompt` |
| 分段 | `SlotRunner::build_prompt_segments(pl, &PromptInput)` | `PromptBuilder::build_prompt_segments`（builtin-only 分支，**不**进入 assembler 消费者） |
| 生成 | `SlotRunner::generate_llm(pl, model, prompt, opts)` | （无 registry 实例）→ `generate_llm_single` → `LlmClient::generate_with_opts`（替身**重写**该入口；`generate`/`generate_tag` 只计数，以证明未被旁路） |
| 情绪 | `SlotRunner::analyze_emotion(pl, text)` | `pl.emotion.analyze` → `CountingEmotion` → **真实** `BuiltinUserEmotionAnalyzer` |
| Agent / Event | 未调用（记录型替身） | 计数器在每条链上均为 0；替身不触模型或工具 |

**新增证据（`cp_b3_v1_` 共 6 项，全部 exit 0）**：

- **V1-1 命中交接**：三个原 Memory（高权重不相关 / 较低权重相关 / 另一相关项），给定 query 后由真实 `rank_memories` 选出 `v1-mid-matched`、`v1-low-matched`；把返回的**原件切片原样**交给真实 `build_prompt`，断言两个选中标记进入记忆证据区、未选中的 `MARKER_UNSELECTED` **不出现**、原件字段（id/role_id/importance/weight/created_at/scene_id/mention_count/accessed_at/content）逐项保持；记忆证据区标题按独立字面量断言；计数 Memory 1 / Prompt 1 / 生成 0。
- **V1-2 兼容回落**：同一候选集下无命中 query 走既有 Host 全候选加权回落（`v1-high-unmatched`/`v1-mid-matched`/`v1-low-matched`）并进入真实 Prompt；空（`trim` 空）query 得到同一集合，且**该次返回的切片实际传入**真实 `build_prompt` 并对输出断言（标题 + 三个独立标记），不是只比较 id；`limit=1` 只留顶部一条、`limit=0` 时 Prompt 内**没有任何**记忆证据（候选不会被测试 helper 偷塞）；Host 回落**不**被称为 Base 命中（Base 的 None/空集语义由既有外测承担）。计数 **Memory 4 / Prompt 4 / 生成 0**（CP-B3-V1-R1 更正：先前写的 Prompt 3 与实际执行不符，且"limit 0 没有东西可喂"的解释是错的——`limit=0` 的空切片同样被送进了 Prompt 入口，其"无记忆证据"断言正是在该次输出上成立的）。
- **V1-3 交给生成端**：真实 `generate_llm` 收到上一步实际产生的**整串** prompt；替身重写 `generate_with_opts` 并记录 model/prompt/`Option<opts>` 与入口次数；断言 `None` 保持 `None`、显式 opts 六个字段逐一到达、outcome 原样（`reply` 与 `prompt_eval_ms`）、空 reply 保持空、单实现**一次**调用、`generate`/`generate_tag` 计数为 0；脚本化 typed `AppError::OllamaError` 原样返回且**不重试**（不对完整回合的 fallback 行为作外推）。
- **V1-4 segments**：同一份 `PromptInput` 分别走真实 `build_prompt` 与 `build_prompt_segments`；`full() == stable_prefix + dynamic_suffix`、`stable_len() == stable_prefix.len()`；段边界按**正反两侧**断言——世界观片段**在** stable 头且**不在** dynamic 尾，`【对话硬约束】`**在** dynamic 尾且**不在** stable 头，记忆证据只在 dynamic 尾（CP-B3-V1-R1 补齐后两项负向断言；这是**本 fixture + 本输入**的段边界事实，不提升为"任何自定义模板/后端/`PromptInput` 都如此"的全称结论）；**不假定两入口整串相等**（两种文档布局不同，普通布局的字节 golden 由该入口自身测试承担）；把 `segments.full()` 交给生成端时断言整串逐字交接；分段入口不额外进入 assembler 消费者。
- **V1-5 Emotion**：真实 `analyze_emotion` 经 builtin 返回七维——词表命中样本 `[1,0,0,0,0,0,0]`、无命中样本走既有兼容回退 `[0,0,0,0,0,0,1]`、两者 `extension` 均为 `None`（未为"更合理"改动任何 production 值）；计数 2。**七维分析与进入 Prompt 的语气标签是两个分别受测的事实**：标签（`v1-fixture-happy`）由 fixture 手写，本轮**没有**验证任何"七维 → 标签"转换边，也不把这条链画成所有边都已动态串通。
- **V1-6 隔离与计数**：两条**顺序执行**的消费组合（链 A「咖啡」+ `v1-model-A` + 无 opts；链 B「茶」+ `v1-model-B` + 显式 opts）；断言两条链的选择各自正确（「茶」只命中真正含该字的原件，正是隔离证据）、Prompt 各含自己的用户输入且不含对方的、两条 Prompt 不相等、两次生成条目按序保留各自的 model/prompt/opts；计数 Memory 2 / Prompt 2 / 生成 2 / 情绪 0。**口径更正（CP-B3-V1-R1）**：撤回"各用局部 query/material/model/opts"所暗示的全量局部值证据——实际 local 的只有候选 `Vec`、组装出的 prompt 与链 B 的 opts，而 query/model/`user_input` 是**字面量**、候选集合被两条链**复用**；本轮**不**声称跨 Pending 借用、生命周期或并发证据（不要求为此重造并发测试，也不要求把字面量换成 `String`），隔离断言本身仍然有效。这**不是** Kernel 阶段顺序，**不**声称整回合总共一次模型调用；计数是本测试的入口次数，不是整回合预算。

**证据分层与边界**：① 源码/编译证明真实产品入口连到哪些方法（上表 + 生产前缀哈希不变）；② 本轮测试直接调用这些真实方法并**手动组合**（相邻调用由测试自己接线、装配选择由测试给定，不代表产品 assembly 已在运行中验证，也不构成一条所有边都动态串通的完整产品链）；③ 未运行完整 pre/post/turn/`AppState`/HTTP/模型链。②不写成③。断言"未调用"使用记录型替身而非静默 `Noop`；`Noop` 返回值未被当作分析依据。人工 fixture 的领域字段来源明确：`Role::default()` 内存构造（不加载角色目录），七维是固定向量，情绪标签为手写字符串。**值来源口径（CP-B3-V1-R1）**：本节断言里既有字面量也有局部值，逐条以各自测试注释为准；"交接的是调用参数"不等于"经过 HTTP wire/序列化"——本轮观察到的只有 `LlmClient::generate_with_opts` 的调用参数（`None`/`Some(opts)` 与六个字段），没有请求体、编码或传输。

**命令与结果（收工字节实跑；cwd 仓库根；Cargo 一律 `--locked --offline -j 1`）**：

| 命令 | 结果 |
|---|---|
| `-p oclive_kernel_host --lib cp_b3_v1_`（先 `-- --list` 得 6 项） | **6 passed** |
| Host `--lib cp_b3_all_` / `--lib cp_b3_c1_` | **32 / 10 passed** |
| Host `--lib domain::fallback_agent::tests` / `--lib domain::event_impact_ai::tests` | **3 / 17 passed** |
| runtime `--lib cp_b3_all_` / `--lib cp_b3_c2_` / `--lib domain::prompt_builder::tests` | **12 / 10 / 44 passed** |
| runtime `--test chatpro_base_memory` / `--test chatpro_base_prompt` | **2 / 2 passed** |
| contracts `--test base_only_fixture` | **16 passed** |
| `cargo check -p oclive_kernel_host --lib`（默认与 `--features dual_core`）、`-p oclivenewnew-tauri --lib` | 均 exit 0（仅编译，不运行桌面） |
| `cargo clippy -p oclive_kernel_host --lib --tests -- -D warnings` / `cargo fmt --all -- --check` | 0 诊断 / exit 0（需格式化时只对 `slot_runner.rs` 调 `rustfmt`，生产前缀哈希复算不变） |
| layering（`rg` 只在单条命令进程内临时前置，未安装、未改系统 PATH）/ module-compat / stale-paths / doc-registry / markdown-links / `git diff --check` | 全部 exit 0 |

**重叠与未跑**：`cp_b3_v1_` 的 6 项只存在于 `slot_runner` 测试区，不与 `cp_b3_all_`（Host 32 / runtime 12）、`cp_b3_c1_`、`cp_b3_c2_` 等前缀重叠；上表按目标分别列出、不累计。runtime/Host 无 filter 全量、`check:rust` 系列、ci-local/release/dimension5/drift、桌面集成运行、真实或环回请求、DB/迁移、服务/MCP/模型/设备/用户目录操作全部**未跑**；桌面只做 `cargo check --lib`。承接主控上一段的口径更正：上一批 Event 交错测试中的**模型名仍是字面量**，不计为"局部 String 的模型借用"证据。

**真实阶段最小方案草案（仅列待批准条件；未查凭据、未启动进程、不假定授权）**：拟用本地 Ollama 的非流式 `generate_with_opts`（C1 已验收范围）跑一条完整 pre→post 回合。需维护者提供：① 模型名与是否允许下载/联网；② 独立隔离的数据目录与临时 SQLite（不得使用用户库）；③ 是否允许 MCP/工具与语音，默认全关；④ 可撤销范围（仅测试会话与临时文件）；⑤ **副作用白名单**（本轮草案已含隔离目录与临时 SQLite 写入，因此判据必须按白名单而非"禁止一切写入"）。通过判据：同一回合内 Prompt 整串与 `generate_with_opts` 实收一致、Memory 选择与提示词证据区一致、回合结束时写入只落在白名单范围内。**失败判据（CP-B3-V1-R1 收窄）**：越过将来明确批准的副作用白名单，或违反已核实的转换/默认值/绑定约定（例如参数在更高层被静默改写、默认值覆盖了显式值）；**不**把 Host 正常的准备、改写与写入当作故障。**离线消费者证据已足够的部分（选择 → 组装 → 生成端参数交接）冻结，不因"更多样本"再延长。**

**止点**：状态为**消费者组合离线验证完成／待主控验收**，不是整回合或真实模型验证完成；未 commit、未 push，不自行开展下一阶段。

### CP-B3-V2：六槽与 ChatPro 验证总计划 S0–S2 验收索引与覆盖边界（2026-09-21 · S1 PASS／S2 修订为分阶段准入候选／S3 NOT_RUN；待主控验收）

**基线与执行**：总计划 `.cursor/plans/chatpro-verification-master-v2.plan.md`（23881 字节 / `FBE29CBC…EC63`）；受测提交 `448c4dc3e14f8fca0b1a35c7ae97a3195bc972ea`（父 `54b5d7a4…8b5b`）+ 同一工作树快照（`slot_runner.rs` 82071/`9211EBDE…DC1F`、`handoff/README.md` 99825/`C1FC4A7D…02CA` 两个未提交 M 原样保护，本轮未 commit/stash/reset/push）；run_id `CPB3V2-20260921T0550-448c4dc3`，逐命令日志与 `baseline-manifest.json`（538 文件 / 112533 字节 / `8C0CD1B7…9C21`）在 `E:/OCLive/_recovery/CPB3V2-20260921T0550-448c4dc3/`。**本片只读验证与记录：未改任何 Rust/TS/测试/依赖/配置/角色/前端/门禁/台账字节**，仓库内仅本页追加。

**S1 离线验证（本轮实跑，全部 exit 0；按 target 计数、重叠不相加）**：`base_only_fixture` 16；五个 Base 外测**按原始运行顺序**为 Agent **6** / Emotion **7** / Event **4** / Memory **3** / Prompt **6**（CP-B3-V2-R1 更正：此前按命令参数顺序写成 `6/7/4/3/6`，把 Memory 记成 6 是错的；原始 `logs/F2.log` 顺序为 agent→emotion→event→memory→prompt，无需重跑）；`base_llm_adapter` 1；`b2_c1_` 20；`cp_b3_c1_` 10；`cp_b3_c2_` 10；`b2_c4_` 17；runtime `cp_b3_all_` 12；host `cp_b3_all_` 32；外测 Memory/Prompt 2/2；`prompt_builder::tests` 44；`fallback_agent::tests` 3；`event_impact_ai::tests` 17；`cp_b3_v1_` 6；`lexicon::tests` 18。编译/静态/文档：host lib 与 `--features dual_core`、桌面 `--lib` 编译 0；clippy（host+runtime lib/tests，`-D warnings`）**0 诊断**；`fmt --all -- --check` 0；`--workspace --doc` 0（contracts 11 / runtime 14 / types 16 / host 2 / validation 3）；layering / module-compat（含 plugins index validator）/ stale-paths / doc-registry / 本页链接 / `git diff --check` 全 0。**setup 审计**：集成 target 与 runtime 侧危险面检索 0 命中；`base_llm_adapter` 只构造并 drop 未 poll 的 future；`base_llm_binding` 用预设传输驱动生产共享路径；host crate 内 13 个会触网络/环回的名字（`test_health_check_offline`、`openai_compatible_llm` 与 `performance_llm` 的 loopback 测试等）**全部不在**任何白名单 filter 的选中集合中——这也是禁止无 filter 全量的具体理由。

**六槽覆盖（L1/L2/L3）**：Memory（B1/`base_memory`/`chatpro_base_memory`/runtime+Host `cp_b3_all_`/V1：显式材料+query 真实参与、Base 选择与 Host 加权/无命中回落分层、原件字段与 limit、选择进 Prompt）；Emotion（`base_emotion`/`cp_b3_c2_`/`b2_c4_`/`lexicon`/V1-5：命中、无命中兼容回退、否定与 UTF-8 原反例）；Event（`base_event`/Host `cp_b3_all_`/`event_impact_ai`：一次生成内数值与叙述各自投影、输入/context 保真、失败走既有规则回落）；Prompt（`base_prompt`/`chatpro_base_prompt`/`prompt_builder`/V1-4：Base 有限拼接与非空要求拒绝、rich input 保留、guardrail/segments 正反两侧边界、生成端收到同串）；LLM（B1/`b2_c1_`/`cp_b3_c1_`/`base_llm_adapter`/V1-3：`done` 两态与两投影差异、with-opts 唯一获准变化、采样与 metrics 来源、typed timeout carrier）；Agent（`base_agent`/Host `cp_b3_all_`/`fallback_agent`：纯计算 Base 与工具型 Host 分开、一次执行两视图、输入不补未知领域值、授权拒绝不降级）。**仍然只是源码/编译边或未验证**：Emotion 七维→Prompt 语气标签（V1-5 明示为 fixture 手写，无转换边证据）；Agent/Event 报告的产品消费；真实远端/目录实现；DB 记忆写回与语义检索最优性。

**S2 准入包（CP-B3-V2-R1 修订；只读调查，不是"运行目标"）**：S2 三态分层＝**已确认**／**技术未知**／**需维护者批准**，**当前不是可直接执行的安全附录**。已确认（附锚点）：非流式入口链与槽调用点；Event 开启时额外一次 `generate_tag`；Agent `handled=true` 短路主生成；`apply_user_llm_env`（`state/app_state_builder.rs:464`；`domain/user_llm_env.rs:160-209`）在 build 时总会运行并**改写当前进程环境**（7 个键 set/remove，:101-127）、可能写 DB 与 `app_data` 内 token 文件；**A 候选**（注入记录型客户端）下 `performance_llm=None` 不进 warmup（`app_state_builder.rs:202-203/475-490`）、`self.ollama=None` 使 `schedule_ollama_preload` 早退（`state/mod.rs:290-292`）⇒ **build 期无网络来源**；临时 app_data/roles/DB 覆盖主 DB·备份·失败 marker·授权文件·token 文件·聊天存储根·模型目录，`<roles_dir>/../plugins` 与 `<roles_dir>/.oclive_directory_plugin_data` 随 roles_dir；**请求预算**（主生成 ≤3 含质量修复 `turn_pipeline/post.rs:581-631`、Event ≤1、后台人格演化 0/1、预热与探测在 A 候选为 0）；**退出**只覆盖目录插件/Performance/trace/SQLite 池（`role_kernel.rs:352-359`），probe/preload/warmup/演化四类 `tokio::spawn` **无句柄、无法 join**。技术未知（U1–U6：空 DB 默认值、合成角色最小文件集、首次迁移清单、授权文件创建、空目录是否 spawn、Profile 演化 interval）全部可由**只读源码**收敛，不交给用户猜。需维护者批准的最小集合＝① A 候选（临时写入 + harness 写集 + 以子进程整体退出为收尾条件，**不必先挑模型**）；② B 候选（模型/端点/联网/Event 额外请求/设备探测/角色包/上限）；③ 两者共需的副作用白名单。候选 A＝`OcliveKernelConfig`+`OcliveKernelBuilder`（临时 app_data/roles + `with_database_path` 临时 DB + `with_llm_client` 注入记录型客户端）为首选，B＝批准模型的真实整回合可延期；**两者均未准入**。详见 `E:/OCLive/_recovery/CPB3V2-R1-20260921T0700-448c4dc3/`（`r1-isolation-and-candidates.md`、`r2-request-budget-and-exit.md`、`r3-layered-expectations.md`、`r4-corrections-and-admission.md`）。

**两句口径纠正（只在新记录，不返修历史）**：① V1 回执"原版本 Prompt 3 错误"收紧为——原版本**实际就是 3**，V1-R1 新增空 query 那次调用后才是 **4**；差别是当时口径未覆盖该交接，不是记错了一个当时不存在的数。② README 保护范围收紧为——受保护的只有**已提交历史主控段**；V1/V1-R1 追加段位于未提交工作树且 R1 改过 V1 段，**不能**声称整段旧 V1 正文仍是字节前缀。

**CP-B3-V2-R1 两条撤回与三条补正（只记新结论，不返修上文）**：① **撤回**"关闭 Event 可把整回合压到 1 次请求"这一通用结论——`turn_pipeline/post.rs:581-631` 的 `for attempt in 1..=2` 修复循环会使主生成路径最多 3 次（反例已由源码给出，无需真实模型复现）。② **撤回**"注入 typed `AppError` 会如实上抛且不静默重试"的整回合断言——`post.rs:555-578` 会把它捕获成 `main_llm_fallback` + 本地兜底回复并**继续**持久化；端口层事实成立、上层行为另论。③ 补正：正常空文本在 ChatPro 层会被 `rejected_reply_reason`（`post.rs:164-168`）判为"没有可显示台词"并进入修复循环，Base 端口 `Ok("")` 合法与此不矛盾——四个场景的分层表在 R1 附录 `r3-layered-expectations.md`。④ 补正：初始化网络来源不止首回合探测，还包括 Performance warmup（`app_state_builder.rs:482-490`）与 `schedule_ollama_preload`（`state/mod.rs:274-314`，A 候选下二者均不触发）。⑤ 补正：退出闭环存在技术阻塞——四类 detached 任务无句柄，`shutdown` 不等待它们，故"安全收尾"当前不可由现有机制证明。

**未运行／未证明**：S3 真实进程、HTTP/环回、模型、临时/真实数据库与完整 `AppState` 运行**均未执行、未标 PASS**；真实模型分析质量、远程 CI、发布级结论不适用；外围冻结项未恢复。状态：**S1 PASS（本片实跑，执行者＝worker 会话，主控已独立复核）；S2 已修订为分阶段准入候选（已确认／技术未知／需批准），非可直接执行附录；S3 NOT_RUN、待准入**。

**A-P1 准备轮点补（2026-09-21 · 全程只读；只记新结论，不返修上文）**：候选 A 的实施草案已产出 `.cursor/plans/chatpro-verification-a-execution-draft.plan.md`（34194 字节 / `D5475610…C2B9A`，首行"待批准，不得运行"），含唯一配置表、合成角色夹具全文、三场景与请求预算、退出三层、命令白名单与待批准权限；**未获批准前不编译、不运行**。四处口径收紧：① 上文"**A 候选 ⇒ build 期无网络来源**"只对**暖机**（`app_state_builder.rs:202-203/475-490`）与 **preload**（`state/mod.rs:290-292`）两个来源成立，**不得**外推为"整个 build 不联网"——须叠加注册/扫描/后端解析排除项；② 因此 A 必须显式固定角色 `plugin_backends`（`llm=ollama`（注入替身即 `backend_registry.rs:154` 的 `llm_ollama`）、`agent=none`）并在子进程设 `OCLIVE_LLM_BACKEND=ollama`，否则 `plugin_resolution.rs:73-90` 的 env 覆盖会把 llm 改成 remote/directory 从而绕过注入替身并构造远端客户端；③ 上文 §S2 ⑤ 的"无句柄 ⇒ 安全收尾不可证明"不作通用结论——A 的配置使 preload/warmup/人格演化三类任务**不启动**（无需 join），首回合启动探测 1 次并由录制替身兜住；④ 主生成 `Err` 后 `post.rs:629 Err(_) => break` 只结束修复循环，且 `!main_llm_fallback` 为假使质量修复**被跳过**——不得写成"仍继续质量检查"，同理持久化仍有条件与失败点，"继续执行"≠"必定提交成功"（`persistence.rs:295-379`）。本轮未运行任何 Cargo/npm/harness/DB/子进程/网络。

**A-P2 执行段（2026-09-21 · 候选 A 三条隔离非流式整回合 PASS · 待主控验收）**：按主控《A-P2 主控执行补充约束》实施并实跑，**只新增测试面 7 文件**（`kernel/crates/oclive_kernel_host/tests/a_turn_harness/{main.rs, support/{mod,env,fixture,recording_llm,driver,artifacts}.rs}`，合计 67252 字节），集成测试目标 `a_turn_harness`，**四个用例全部 `#[ignore]` 显式 opt-in**（不带 `--ignored` 时 `0 passed / 4 ignored`，已实测）。静态/编译：`cargo check`/`--no-run`/`--list`/`clippy -D warnings`/`fmt --check` 全部 exit 0（统一 `--locked --offline -j 1`）。实跑（`--ignored --exact … --nocapture --test-threads=1`，一次一个，`OCLIVE_A_RUN_ID` 只设在命令进程范围）：S1 正常回复（`reply` 与脚本常量逐字一致、非 fallback、主生成 `generate_with_opts` **1** 次）、S2 空回复修复（首次空 ⇒ 修复一次，**2** 次、最终回复为脚本常量）、S3 主生成 `Err(OllamaError("A-HARNESS-SYNTHETIC-ERR-3"))` ⇒ `reply_is_fallback=true`、`llm_fallback_reason` 含该合成标记、**恰好 1** 次主生成（修复循环被跳过），三场景替身 `violations=0`（无 `generate`/`generate_tag`/流式/`startup_probe`）。DB 核验在 `shutdown`（返回 `()`）释放池后用**只读** sqlx 连接完成：`chat_messages` 恰好 2 行（`session_id=a-probe-role`、`sender=user/assistant`、内容与输入/响应逐字符对应、ID 与响应字段对应）、`chat_sessions` 1 行（`role_id=a-probe-role`、`scene_id=default`、`message_count=2`）。本片按补充约束**关闭 JSON mirror**（有效开关是 env `OCLIVE_CHAT_STORAGE_BACKEND=sqlite`；角色 `config.json` 的 `mirror` 字段在该运行路径上不生效——首轮实测失败即由此暴露，已在**新 run_id** 下重跑，首轮树原样保留）⇒ `chats/**` 文件数 0；退出码 0、恰好一条 `test result: ok. 1 passed`、无超时、无强制终止、`shutdown=returned`。证据：`E:/OCLive/_recovery/CPB3V2-A-20260921T1920-448c4dc3/`（含 `SELF-REVIEW.md`）与首轮 `…T1919-448c4dc3/`。**未覆盖**：候选 B、JSON mirror/replay 完成性、HTTP 与模型推断计数（A 下不可观测）、OS 级全程监控、`policy_file_applied`/`scan_now`、detached probe 调度；本段只证明"指定夹具 + 注入客户端 + SQLite 权威存储配置"下的三条隔离整回合，**不是**产品级验证通过。

**A-P2-R1 独立说明（2026-09-21 · 测试工具返修与复验 · 待主控验收）**：按主控《A-P2-R1 集中返修与复验任务书》（14010 B / `981CADCA…5120`）修三类验收缺口，**只改测试面**（`tests/a_turn_harness/{main.rs, support/{mod,artifacts,driver,fixture,recording_llm}.rs}`）。① **R1 路径先验与失败关闭**：新增测试私有接缝 `FsProbe`/`MetaFacts`/`OpsLedger`（Windows 用 `MetadataExt::file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT` 判重解析，覆盖 junction），任何创建/写入/spawn/build 之前先逐组件核验（含恢复区根），新组件在已验证父下逐个新建再核验，场景子树拒绝复用；`artifacts` 全部改 `Result`——枚举/条目/metadata/读取/哈希/重解析任一失败即失败，`Absent` 与"存在但为空"分开，坏 JSONL 不再静默跳过。② **R2 严格判据**：S1/S2 回复**完整等于**独立常量；S3 解析 `llm_fallback_reason` JSON 断言 `code=LLM_ERROR`、`message="Ollama error: A-HARNESS-SYNTHETIC-ERR-3"`；S2 第二条 prompt 必须含 `【上一候选被拒原因】` 与 `没有可显示台词`、第一条不含；SQLite 按 sender+id+session_id+content 四者绑定并核对会话行；opts 要求 `Some`、`want_metrics=true`、`keep_alive="30m"`，另四字段**键必须存在且为 null**；libtest 摘要改为严格解析（恰好一条、ok、1 passed、0 failed、0 ignored）。③ **R3 失败收尾**：`supervise` 对 spawn 失败/wait 错误/超时/坏报告统一先落终态报告再失败，kill/reap 尝试与结果如实记录、只处置确切句柄、正常退出不额外 kill；`load_role` 失败与回合 Err 均走一次 `shutdown`，超时后**不**按"池已关闭"查 DB。静态门禁全绿（`check`/`--no-run`/`--list` 19 用例/`a_p2_r1_` 15 passed/默认 15 passed+4 ignored/`clippy -D warnings` 0 诊断/`fmt --check` 0，均 `--locked --offline -j 1`，未新增 allow、未降标准、未删测试）。新 run_id `A-20260921T1942-448c4dc3-R1` 三场景复验 **S1/S2/S3 全 PASS**（严格回复判据、主生成 1/2/1、violations 0、DB 2 行+1 会话只读核验、`chats` 0 文件、退出码 0、`shutdown=returned`）。**撤回上一段"旧断言已严格验证"的暗示**：§93 与旧报告仅为当时观察（当时为 `contains`/非空判据、SQLite 未绑定 sender-ID-会话），旧 PASS 不抹去也不倒填为新版覆盖。证据：`E:/OCLive/_recovery/CPB3V2-A-20260921T1942-448c4dc3-R1/SELF-REVIEW.md`；旧树 `…T1919`、`…T1920` 原样保留。

**A-P2-R2 集中说明（2026-09-21 · 三处安全修正与复验 · 待主控验收）**：按主控《A-P2-R2 安全拒绝与有界收尾集中返修》（11674 B / `200A9C7D…8886`）只修三处测试工具安全缺口，**改动仅限 `tests/a_turn_harness/{main.rs, support/{mod,driver,artifacts}.rs}`**（`support/{env,fixture,recording_llm}.rs` 与生产面未动）。① **拒绝后零写入**：新增 `ReportSink`/`handle_failure`/`PreparedTree` ++ `probe_dir_state` 三态（仅 `NotFound` 可在已验证父下创建，`PermissionDenied` 等状态不明一律先拒绝）；只有"本次全新且已验证的 reports 目录"存在时才允许写报告，路径被拒阶段不调用任何 writer、不覆盖已存在场景的旧字节；`artifacts` 的 JSON/JSONL writer 取消 `create_dir_all`，改为**核验父目录**后写入。② **清理真正有界**：`ChildControl` 只保留非阻塞 `try_wait`（`Exited(Option<i32>)`/`Running` 明确区分，退出无码 ≠ 仍运行）+ `kill`，删除无界 `Child::wait`；清理期限独立 2s、`kill` 至多一次并保留失败原文，未确认回收记 FAIL/BLOCKED；guard 的 `Drop` 只做 ≤1s 非阻塞轮询且清理完成后不再隐式处置。③ **通过后才发布**：`joint_checks` + `publish_if_all_checks_pass` 先算全部联合检查，全通过才写 `ok=true`，否则只写失败状态。静态门禁全绿（`check`/`--no-run`/`--list` 29 用例/`a_p2_r` 25 passed/默认 25 passed+4 ignored/`clippy -D warnings` 0 诊断/`fmt --check` 0）。新 run_id `A-20260921T1954-448c4dc3-R2` 三场景复验 **S1/S2/S3 全 PASS**（回复完整等于常量、主生成 1/2/1、violations 0、DB 2 行+1 会话、`chats` 0 文件、退出码 0、`shutdown=returned`、正常路径未触发 kill）。证据：`E:/OCLive/_recovery/CPB3V2-A-20260921T1954-448c4dc3-R2/SELF-REVIEW.md`；旧树 T1919/T1920/T1942 与 §93/§94 原样保留。

**A-P2-R3 集中校正（2026-09-21 · 三阻塞项闭合 · 已获主控限定验收）**：按主控《A-P2-R3 监督失败判定与反例接线集中收尾》（10561 B / `4AFC8B3F…62F7`）修正三处，写集仅 `tests/a_turn_harness/{main.rs, support/driver.rs}`。① **F1 监督失败不得被成功收尾洗白**：新增 `driver::supervision_failure`（优先级：清理未确认 BLOCKED → 监督查询错误 → 运行期限到 → 清理 kill 错误 → 未确认退出 → 退出码非 0/缺失），`run_driver_with` 在读取成功材料前调用该准入并走失败报告，`joint_checks` 增第 8 项 `supervision_clean`；**清理确认只证明进程已退出，不撤销超时/查询错误/清理错误的原失败**。② **F2 连续查询错误必须真连续**：`FakeChild` 改 `poll_err_from/poll_err_until` 直接控制实际 `try_wait`，回归断言实际多次错误查询、清理期限到、kill 恰一次、`blocked_reason` 保留、轮询有界，并删除无效的独立 `err_every`。③ **F3 拒绝零写入接到实际连接**：新增 `driver::prepare_or_reject`（driver 实际使用，无 `PreparedTree` 即不调用 writer），`CountingSink` 增内存 sentinel，覆盖祖先/恢复区根/**仅 run** 重解析、tree/run `PermissionDenied`、旧 scenario 已存在、部分创建后停止 —— 全部零写入且旧字节保持原值。静态门禁全绿（`check`/`--no-run`/`--list` 31 用例/`a_p2_r` **27 passed**/默认 27 passed+4 ignored/`clippy -D warnings` 0 诊断/`fmt --check` 0）。新 run_id `A-20260921T2006-448c4dc3-R3` 三场景复验 **s1/s2/s3 全 PASS**（回复完整等于常量、主生成 1/2/1、violations 0、DB 2 行+1 会话、`chats` 0 文件、exit 0、`shutdown=returned`、正常路径未触发 kill）；主控已独立复跑 27 项、`clippy`/`fmt`/`git diff --check` 并核对 48 个清单文件哈希。**历史更正**：R2 的"连续查询错误""实际 run 重解析""旧报告字节对照""完整错误写连接"四项过宽覆盖由 R3 证据取代，R2 当时 25 项通过不倒填。**A 的最终口径**：仅"指定合成角色 + 记录型注入客户端 + SQLite 权威存储且关闭镜像"下的三条隔离非流式整回合；真实模型/HTTP/质量、镜像与 replay 完成性、多轮/流式/情景移动/成人 staged、目录插件/工具/设备、`policy_file_applied`/`scan_now`/detached probe 调度、全 OS 级监控均未覆盖。证据：`E:/OCLive/_recovery/CPB3V2-A-20260921T2006-448c4dc3-R3/SELF-REVIEW.md` 与 `.cursor/plans/chatpro-verification-a-p2-r3.controller-review.md`；未 commit/push、未进入 B。
