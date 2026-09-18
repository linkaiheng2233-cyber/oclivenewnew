# handoff · 维护者与 AI 工程入口

**SSOT 范围**：全仓文档的读者分层、状态判读、活跃 handoff 职责与维护/归档规则；不承载运行时业务契约。
**最后更新**：2026-09-18。
**新人开发者**从 [human-docs](../human-docs/README.md) 开始；**创作者**从 [创作者黄金路径](../creator-docs/getting-started/CREATOR_GOLDEN_PATH.md) 开始。

**六槽 B1 当前交接状态**：见 [B1 本地收口](#six-slot-b1-closure)。本页较早的“待实施／尚未编译”属于当时记录，不是当前开工指令。

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
| 后续 Event / Agent | 未开始 | — | 逐槽独立任务与验收；旧路径无法诚实适配时保留明确限制，不为凑齐六个实现返回默认值或扩 Base |

**推进口径**：每片主控验收后独立本地提交，再进入下一槽；顺序是工程推进顺序，不是 Kernel 执行流水线。生产接线、旧接口整体迁移、ChatPro 接入与远端 CI 结论都不由本节点推断。

**B2-C2 主控验收（2026-09-17）**：实际重跑 runtime 的 `b2_c2_` 单测 10 项、`base_memory` 集成测试 3 项、`domain::memory_engine::tests` 14 项（与前述 10 项有重叠，不相加作独立用例数）、`domain::memory_retrieval::tests` 2 项，以及 B1 `base_only_fixture` 16 项，均通过；`cargo test --locked --offline --workspace --doc -j 1`、runtime `clippy --all-targets -D warnings`、fmt、分层/module-compat/stale-paths/doc-registry/Markdown 链接与 diff 检查通过。runtime rustdoc 构建通过并实查公开模块/类型页面、边界正文和链接；保留 5 条未改文件上的既有警告，不记为零警告。生产匹配 helper 的共享调用、原文/顺序/重复保留和受保护文件哈希经源码复核。worker 的 runtime 全量 211 项记录仅作补充，未冒充主控重跑；未运行真实 I/O、模型、Host 产品链或网络探测。本片不证明语义检索、持久化恢复或旧 Host 已迁移。

**B2-C3 主控验收（2026-09-18）**：复核 §58 返修版本并实际重跑 runtime `b2_c3_` 8 项、`base_prompt` 外部测试 6 项、旧 `domain::prompt_builder::tests` 42 项、`base_memory` 外部测试 3 项与 B1 `base_only_fixture` 16 项，均通过；显式复用同一实例的成功→Unsupported→再成功测试已落实。`cargo test --locked --offline --workspace --doc -j 1` 通过：contracts 11、host 2、runtime 6、types 16、validation 3；本片模块示例实际运行并严格断言，类型示例仅编译，不能把 runtime 6 项都算成本片运行证据。保留 validation wasm 的 cdylib 不支持 doctest 提示，不计为已执行测试。

runtime `clippy --all-targets -D warnings`、fmt、分层/module-compat/stale-paths/doc-registry、本页 Markdown 链接与 diff 检查均通过；runtime rustdoc 构建及公开模块/类型页面的示例、边界正文和互链实查通过，5 条警告均在未改文件。主控核对生产逻辑与公共面没有随返修扩大，§58 对此前编译/执行及调用序列证据的更正成立；桌面旧全文前缀核对一致。本片只证明有限参考实现与独立调用，不证明任意自然语言组装要求、抗提示注入、Host 产品链或真实模型效果；未运行 Host 全量、宽 CI、模型/服务/网络/数据库实验，未 push，冻结项不变。

**B2-C4 主控验收（2026-09-19）**：复核 §61 返修版本、实际五文件与受保护哈希。初次验收发现的共享 UTF-8 panic 是本片阻塞，换成安全材料不能替代修复；本次已恢复原混排、引述与条件反例。`is_negated` 在中文窗口规则未短路时调用英文 helper，该路径不按 `MatchMode` 排除中文命中；修复把同一分隔字符的起始字节位置加一，改为加该字符的 UTF-8 长度。共享修复独立提交为 `dd430a15a4af53b1defaebf1bbedf4072eb0290d`，不依赖新 Base 类型；词库、分隔谓词、否定 token、中文窗口及旧七维映射未改。

**主控本轮实跑**（Cargo 测试均为 `--locked --offline -j 1`）：runtime `--lib b2_c4_` **17 项**（含共享 UTF-8 的 4 项）、`--test base_emotion` **7 项**、`--lib domain::lexicon::tests` **18 项**（与前项有重叠，不相加）、`--lib domain::emotion_analyzer::tests` **10 项**、`--test base_memory --test base_prompt` **3 + 6 项**、contracts `--test base_only_fixture` **16 项**全部通过。另外，将上一轮仓库外的同一份 8 项诊断重新链接到本轮 Cargo 生成的真实 runtime/contracts/types 库后运行，结果由上一轮 **4 passed / 4 failed** 变为 **8 passed / 0 failed**；不是重用旧 exe，也不是复制 helper 的模拟。worker 的修前红测单列为 §61 历史证据，不倒填成主控本轮执行。

`cargo test --locked --offline --workspace --doc -j 1` 通过：runtime **8**、contracts **11**、host **2**、types **16**、validation **3**；本片模块示例（line 83）实际运行并严格断言，类型示例（line 177）仅编译，保留 validation wasm 的 cdylib 不支持 doctest 提示。runtime `clippy --all-targets -D warnings`、fmt、分层/module-compat/stale-paths/doc-registry、本页 Markdown 链接与 diff 检查均通过。rustdoc 构建及公开模块／类型页的正文、示例和链接实查通过，**5 条既有警告**均在未改文件，不记为零警告。

**证据校正与止点**：桌面 §61 终态及其前 1124849 字节与 §60 的哈希均已核对；历史材料保留，源码／实际命令优先于回执中的简写。§61 中“B1 fixture 未在本轮重跑”与其命令 #8 冲突，主控本轮已独立跑过 16 项；额外 Host target 不属于 R1 指定验证集合，不作为扩大覆盖范围的依据。本片证明的是有限词表参考实现、新旧公开入口的定点回归及局部借用，不证明完整情绪理解、主体／引述／条件判断、真实模型或 Host 产品链。未接 ChatPro、未运行 runtime／Host 全量、宽 CI、模型／服务／网络／数据库实验，未 push、未删除任何目录或恢复件；B1、C1–C3、Host、MODULE_MAP 与冻结项不变。Event／Agent 尚未启动。
