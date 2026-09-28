# 技术债变动记录与 AI 接手协议

**SSOT 范围**：技术债的依赖登记、变动事件、证据关联与安全续跑坐标；不维护第二份当前债务状态表。
**最后更新**：2026-09-28。

当前状态唯一以 [TECHNICAL_DEBT_INVENTORY](../TECHNICAL_DEBT_INVENTORY.md) 为准；执行准入以 [QUEUE](MARATHON_QUEUE.md)、对应 long-plan 和 [GATES](AI_AND_PIPELINE_GATES.md) 为准。通用过程以[仓库流水线](../workflows/dev-pipeline/SKILL.md)为准。本记录中的旧事件是当时快照，后续事件不能改写原始证据。依赖读下节，实际开工顺序及首批切片读 [第二轮计划](ROUND-02-PLAN.md#当前开工准备2026-09-28)。

## 依赖登记与判读

**核对基线**：2026-09-28，`415d9592990002923ab1d2285ecad9bc998a01bd`，开场工作树干净。下表整理主台账现行事项、覆盖缺口和已有计划的关系；只读核对，不代表重测全部行为，也不复制债务状态。历史已结案项只作为可复用基础，不重新列为待做工作。未列出的事项仍须按 owner 检查，**未登记不等于无依赖**。

### 关系类型与登记字段

| 类型 | 含义 / 对开工的影响 |
|------|--------------------|
| **硬前置** | 目标切片实际使用的契约、接口或数据必须存在；检查具体能力及版本，不要求上游整债 Done |
| **决策门** | 存在互斥语义、公开接口迁移或架构取舍；可先只读取证，涉及取舍的写入等待维护者决定 |
| **外部条件** | 指定平台、硬件、跨仓范围、模型预算、凭据或发布权限；只阻断需要该条件的切片或验收 |
| **证据门** | 实现可能已有，但原始证据、目标 SHA CI 或范围内实跑未齐；不把结构/编译通过当结案 |
| **共因 / 协同** | 同一调用链、问题来源或可共享测量；可以合并计划或共同取证，不虚构先后箭头 |
| **建议顺序** | 控制验证成本和风险的工作安排；不是技术依赖，也不修改 QUEUE 的 runnable 条件 |
| **待核实** | 缺少源码或规范依据；记录具体问题与取证动作，禁止靠标题相似推导硬前置 |

每次新增或改变关系，同一检查点登记：**目标 ID + 切片/Minimal/Full + 关系类型 + 前置 ID 或具体能力/版本 + owner/依据 + 解除条件 + 可先做范围**，并追加事件。原始日志不可携带、外部权限未定和源码关系未知分别记录，不合成一个含糊的“blocked”。依赖环先拆开契约冻结、实现和验证；无法拆开的决策环请维护者裁定，不靠重跑解开。

### 工程基础与安全切片

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-TOOLS-01、K-VERIFY-01 | **共因**：门禁真实调用依赖与体检口径；二者不互为整债硬前置 | 主台账 §1；当前 `check-domain-layering`、`check-host-reexport-imports`、`dimension5-acceptance` 均调用 PATH 上的 `rg`。修补须区分无匹配、缺命令和执行失败；体检区分存在/可用/真实门禁通过 | 仓库内梳理调用链、受控子进程缺依赖负控和项目检查方案；`E:\Env` 原始记录/个人脚本只读，外部修改另定范围 |
| K-BUILD-06、K-BUILD-07 | **共因 / 建议顺序**：共享构建环境；先测量，再分别评估链接与缓存策略。互不要求整债 Done | 主台账 §1、`.cargo/config.toml`、`package.json`；历史体积/耗时不是当前测量。当前 CLI 集成命令只有 `--test-threads=1`，不能照抄旧“所有命令均带 `-j 1`” | 只读核配置、产物占用与构建进程；实验前冻结冷/热构建口径、预算和回退。无等价证据不移除现有 `-j 1`，不自动清缓存或删 rlib 变体 |
| D-DEBT-LEDGER-01 | **协同**：支撑接手可靠性；全表规范化不是其他债实施的硬前置 | 主台账前瞻风险、[COVERAGE](COVERAGE.md)；需明确一个权威状态行与显式引用，再逐类迁移历史/补冲突检查 | 随被触及条目增量整理 owner、范围和事件；有状态冲突时只暂停受影响任务，不要求全仓先重写台账 |
| K-ENCODING-01 | **共享执行约束**：所有中文文档写入；不是“先把此债做完才可开发” | [GATES](AI_AND_PIPELINE_GATES.md) §7 的编码红线及写后检查 | 对本轮改文核 UTF-8、无 BOM、正常汉字；长期执行观察与新的自动检查范围分开 |

### 安全、供应链与韧性

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-PLUGIN-SEC-01 | Stage 3 原生隔离是**证据门 / 平台条件**；Stage 4 身份绑定与 K-SUPPLY-09 **协同**，其可信身份能力是该切片的**硬前置** | [插件计划](long-plans/K-PLUGIN-SEC-01.md)；Full 仍需规定平台的原生拒绝证据及可信签名/轮换/撤销，不以通用窗口 smoke 替代 | 核现行 broker/capability、既有负例和原生测试可行性；签名缺口不妨碍独立核 Stage 3，但不能据此关 Full |
| K-SUPPLY-09 | **决策门 / 外部条件**：发布者信任、签名验证、轮换/撤销与开发 opt-out | 主台账 §1.5、[SUPPLY_CHAIN](../../creator-docs/security/SUPPLY_CHAIN.md)；sidecar SHA-256 不等于可信签名 | 梳理安装→身份→桥权限链及失败关闭用例；不自行读取密钥、选择信任根或开放未验证插件 |
| K-SUPPLY-10 | **独立 CI 安全面**，不依赖 K-SUPPLY-09 结案 | 主台账 §1.5；外部 Action 完整 SHA、可追溯来源和升级维护须对应实际 workflow | 只读盘点 Action 和升级配置；若实施，单独冻结写集与 CI 对照，不混进工具链缺依赖修补 |
| K-SUPPLY-05-Full | **生态条件 / 证据门**：当前 lock、重复依赖族与零 skip Full 条件 | [Full 计划](long-plans/K-SUPPLY-05-Full.md)；历史合并不代表当前零 `[bans.skip]`，也不重开已结案 Minimal | 只读核剩余依赖族；选定可收敛族后再修订 Stage，保留供应链审计及中英 advisory 闭环 |
| K-SUPPLY-06、07、08 | **触发条件**：稳定 tag/专用镜像、采购合规需求或成熟自动化方案，分别处理 | 主台账 §1.5、§4；位级可重现、SBOM、`vet` 不互相自动解冻 | 记录触发是否成立；不把供应链 Full 当一个可无界自动推进的总任务 |
| K-RESILIENCE-01 | **决策门**：Full ResilienceLayer 与历史 Minimal 对账是不同切片 | [现有计划](long-plans/K-RESILIENCE-01.md) 只拥有 Minimal；Full 要另定 RFC/写集和语义 | 只读映射调用点/旧 Wave，决定关闭历史计划或修订剩余范围；不由 Minimal 冻结身份启动新层 |

### Kernel 延伸、Host 契约与待决语义

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| D-CLI-BLUEPRINT-05 | **决策门 / 硬前置**：公开加载/Prompt 接口迁移范围，随后统一磁盘封装与生命周期适配 | [ROLE_PACK_BOUNDARY §0](../ROLE_PACK_BOUNDARY.md)、主台账 §1；复用既有最小定义、快照、媒体和 load-only 入口，不复制 parser。关系不是待决必填项，已确定归发行版 | 核调用者和迁移影响；接口取舍未定前不激活旧 `Role`、增加产品默认值或切换 CLI 默认生成目标 |
| K-CORE-BOUNDARY-01、V-EMBED-01、V-PORTABLE-01 | **互补验收面**：逻辑 Kernel、完整参考运行时嵌入、跨发行版映射；物理拆分与 Full 实机条件分别是**决策门 / 外部条件** | [MODULE_MAP](../MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)、主台账；已验 B1/B2、进程内门面及 [ChatPro 案例](../CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md) 只在所列范围复用 | 按选定 Kernel 延伸合同核 Host；不把跨发行版 UI Full 或物理小 core 交付作为工具链、CLI 或嵌入研究的统一前置 |
| K-UID-DEFAULT-02 | **决策门**：profile 默认身份优先级与 compatibility fallback 取舍 | 主台账 §1；维护者选择后才同步 loader、profile、global/per-scene/恢复默认与文档 | 只读追踪现有默认路径及测试；不凭字段名默改用户可见默认身份 |
| K-AGENT-MERGE-01、V-FUSED-01 | **独立范围 / 决策门**：工具 composite 与多槽实例融合不能混为同一实现 | 主台账前瞻风险及 [融合 stub](long-plans/V-FUSED-01.md)；前者须定工具冲突、权限、顺序/短路、超时/隔离与 trace，后者仍受其 Phase 3 条件约束 | 核单 Agent 行为与诊断声明；不从多 ID 推断已合并，也不以此阻断单 Agent/MCP 使用 |
| K-DUAL-ROLLBACK-02、DUAL-CORE-FREEZE | **冻结 / 决策门**：仅在继续维护 opt-in 双核 Beta 的范围内选择 NULL 恢复或收窄实验写入 | 主台账 §1/§2及 [冻结 stub](long-plans/DUAL-CORE-FREEZE.md)；补偿不是数据库事务 | 静态核恢复路径；不解冻实验、不开启默认 dual_core，也不要求 Stable 先等待 Beta 全部修复 |
| D-HOST-RECOVERY-01、D-HOST-ERROR-CONTEXT-01 | **独立残留窗口 / 证据门**：当前源码、确切可达路径和新故障范围；旧历史推断不是当前复现 | 主台账 §4、[Host 检查单](../CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md)；同 ID 恢复与取消已有所列限定证据，提交/收据原子性等另定 | 复核残留路径并保留历史；不重跑旧业务身份，不把所有崩溃窗口设为已收口接入案例的再验收前置 |

### 事件、资源与开发工具

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-EVENT-STREAM-01 | Stage A 的 Session/wire/checkpoint、存储/隐私/ACK及未定恢复策略是 Stage C consumer runtime 的**硬前置 / 决策门** | [RFC_RUNTIME_EVENT_STREAM](../../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md)；Trace/DTO/探针不是 Production，A.2.2.2 的宿主密钥恢复、多宿主协调尚不能当已冻结 | 在获准 A 切片中取证与固化合同；不启生产读取/消费循环、不真实发送或撤回外部消息 |
| K-PROACTIVE-01 | **共用能力**：现有 Event Ring、可信 origin 与一次性 permit；不要求 Production Stream 整债 Done 才能核现有非用户回合 | 主台账前瞻风险、[Event Ring](../../creator-docs/plugin-and-architecture/EVENT_RING.md)；生产持久消费属于上行独立范围 | 核现有主动链与拒绝/零副作用合同；不建立第二套回合入口或把 Trace 接到行为反馈 |
| K-RESOURCE-COORD-01、D-SCAFFOLD-RESOURCE-01 | **契约能力硬前置 / 协同**：CLI 复用已实现 types/Host Plan Compiler；目录自动装配、真实渲染和 soak 是资源父债的独立剩余范围 | [资源 RFC](../../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)、主台账；工具界面需要具体可用字段/原因码，不要求资源父债所有硬件条件先 Done | 核只读配置/诊断、round-trip 与无 GPU/observe-only 负例；不发明新 schema、设备探测或自动跨适配器授权 |
| D-SCAFFOLD-EVOLUTION-04 | **决策门**：使用问题、一个有界目标、迁移及来源/权限/沙箱 | [脚手架 RFC](../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)、主台账；Stage 2C 仍需单独取舍，资源工具并不自动解冻组合/namespace/离线生命周期 | 收集实际使用问题；不把依赖解析、市场/联网安装和第三方 CI 编排权并进首批 |
| K-CI-IMPACT-01、D-CI-AI-REVIEW-03 | **数据硬前置**：AI 评测复用 Shadow/Compare 的版本化真实数据；不要求 selective 全部开放才开始采集 | 主台账前瞻风险；漏选/过选与全量结果对应同事件，训练/评测隔离。AI 不拥有降级确定性门禁、Runner 或 Secret 的权力 | 核已有数据/目录边界；无冻结评测与运行预算不训练，模拟语料不代替真实 CI 漏选证据 |
| K-LLM-ENV-02 | **证据门**：已修并发事务仍缺更长、可控交错/进程级压力 | 主台账前瞻风险；复用当前单 mutex、版本/dirty 测试，不重复实现同一修复 | 静态核压力矩阵与隔离；运行前限定设置、进程和环境恢复，不碰用户凭据或真实 provider 配置 |

### 模型质量、语音与生态产品

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-EMO-01、03 | **协同**：词表与显式降级路径；按实际语言/重复实现边界区分 | 主台账 §1；英文扩充先核中文词表条件，降级出口收拢须保留现有主 LLM 与安全回退语义 | 固定文本回归与调用点盘点；词表完善不等于复杂语义或角色质量已通过 |
| K-EMO-05、06、07 | 趋势需 M2 历史数据，记忆权重需独立语义决策，复杂情绪优化需固定角色/场景评测：分别是**数据硬前置 / 决策门 / 证据门** | 主台账 §1；05/06 不因 07 研究自动获准；07 仅 shadow/advisor，对照通过后才可有界进入 Prompt | 准备可审计评测输入和数据字段；真实 7B 运行须预算，不让规则/CHS/VAD 成为第二个决策内核。历史 S01 FAIL 不因新文档变绿 |
| K-VOICE-01、09 与 H03 真实音频 | **共享测量 / 外部条件**：真实硬件、音频输出、隔离与 soak；不是彼此整债 Done 的前置 | 主台账 §1、[语音跟踪](../../human-docs/team/TRACK_VOICE_RECOGNITION.md)；TTFC/分段边界、30 分钟矩阵与听感分别记账。前端内存音频不能代替真实声音 | 核测量方案与既有消费者合同；不自行播放、使用模型或放宽 8 秒/显存安全线 |
| K-VOICE-07 | **规范硬前置**：精确文件和锚点定义 `voice_directive v2 / engine_extras` | [v2 计划](long-plans/K-VOICE-07.md)；任意编号 §4.1 或 `voice.asr` 小节均不满足 | 只读找规范；没有精确依据即保留 `needs-directive-v2-rfc-anchor`，不自创协议 |
| K-VOICE-02、03、05、08 | **独立产品/上游条件**：通用 adapter、CosyVoice 平台稳定、Fish 观察、非 CosyVoice chunked 抽象 | 主台账 §1及对应 stub；社区 RPC Minimal、角色语音路由已结案部分不重开 | 明确一个适配器/平台范围；不把全部语音上游成熟度作为无音频本地工作的前置 |
| K-CROSS-01、K-DIST-01、V-MARKET-01、V-VSCODE-PERF-05 | **外部条件 / 决策门**：三平台实机、签名/updater与发布权限、姊妹仓 owner、F5/VSIX；条件分别归对应切片 | [QUEUE](MARATHON_QUEUE.md) 与各计划；历史 Minimal 合入和文档缺口说明不满足 Full，也不授予跨仓/发布权限 | 本仓只读对账剩余能力；没有相应条件不自动重领旧 Stage，也不阻断独立工程基础修补 |
| K-CONTINUITY-01、PE-CONTINUITY-01、PE-TURN-01、PE-UID-01 | 编写器需要对应 schema 和跨仓范围，是**契约硬前置 / 外部条件**；运行时人工观感为单独**证据门** | 主台账 §3；PE-CONTINUITY 需可选 schema 冻结，PE-TURN 复用 K-TURN-F1，PE-UID 复用角色包身份规范 | 只读核当前规范及 round-trip 范围；不以运行时人工整债 Done 作为所有编辑器合同取证的前置 |
| D-ASSET-FOOTPRINT-01 | **证据门 / 契约协同**：视觉质量、解码兼容、loader/CSP/编写器/模块兼容链及实测体积收益 | 主台账前瞻风险；资产转换不能只改后缀或未经决策引入 LFS | 只读度量候选及兼容需求；不重写官方资产或将其当 CLI 最小媒体合同的必需迁移 |
| V-LORA-WORKSHOP-01、V-LORA-FORGE-02、V-LORA-PACK-03、V-LORA-PEFT-04 | **产品/契约分界与决策门**：训练 provider、`.ocadapter` 产物、运行时加载、PEFT 转换插件分别归各 owner；只对使用到的输出格式形成硬前置 | 主台账 §4、[LoRA 包契约](../../creator-docs/plugin-and-architecture/LORA_ADAPTER_PACKAGE.md)；本地 v1 已有，不代表多 adapter/信任/质量/训练全部交付 | 盘点一个具体剩余能力；不因训练工坊启动而耦合 PEFT 到稳定内核或替换本地已验加载链 |
| 其余 Observe / Deferred / 冻结项 | **触发条件，尚非排期**：行数/性能反例、外部实现、产品立项、对应 Phase 或 opt-in | 主台账 §2–§4及对应 stub；逐项核触发和授权，未知标 `needs-inspect` | 只读取证；不为了“依赖图完整”自动拆大文件、复活旧实现或批量造 Ready 计划 |

### 开工前的最小核对

1. 选一个具体能力切片；从主台账取状态，找到本节关系及 owner，再读已有计划。关系表不覆盖 QUEUE/机器契约，也不自动给全部 OPEN 项开工许可。
2. 把硬前置落实到实际文件/接口/数据及版本；已实现上游可以复用，不要求其 Full/人工/其他平台全部结案。若同 ID 范围有矛盾，先对账受影响任务。
3. 区分“可先静态/窄测”“可实施”“可验收结案”。真实模型/音频、跨仓、发布或架构取舍只影响对应步骤；本地窄测可在不依赖答案时继续。
4. 依赖通过也不代替本切片的正/负例、目标 SHA CI 与证据携带性。完整台账治理、全部 Host 合规、物理小 core 提取和跨平台 Full 不是首批工具链修补的前置。

## 接手顺序

1. 核 `git status --short --branch`、HEAD 与任务授权；用 `rg -n '<DEBT_ID>' handoff/TECHNICAL_DEBT_INVENTORY.md` 找到当前条目。相同 ID 可能有历史、观测或父/子项引用，不能按第一处 `Done` 判整债完成。
2. 在本文件检索该 ID，读最新相关事件及其证据；再读目标计划的当前 Stage。核对 **debt ID + Minimal/Full + 能力范围 + base/head**，不只按标题或时间接手。
3. 对照台账、QUEUE、计划契约、实际 diff 与远端 PR/CI。冲突先记录 `needs-reconcile`；检查器绿只证明其声明的结构，不替代语义对账。
4. 取原始证据；本机忽略目录或用户盘内文件无法取得时标 `needs-evidence-access`，不得把摘要补成原始日志。已消耗业务身份不复用；未知外部进程或 dirty 不擅自清理。
5. 继续一个本轮已授权的具体范围；恢复前确认最后命令、下一步和 `retry_safe`，再按本页依赖核对切片前提。本登记不自动开放全部 OPEN、Full 或跨仓工作。

## 更新规则

这里的“实时”指**在事实发生的同一个工作检查点写入**，不是后台定时任务。以下事件必须记账：新增/合并/拆分债、状态或优先级变更、Minimal/Full 边界或依赖/解除条件变更、阻断/解冻、新实现或验证证据、复核更正、交接/暂停/收口。

- 先取得证据再改 owner 文档；控制者同一检查点同步受影响的台账、计划、QUEUE 和本事件。无事实变化的轮询不追加事件；Wave 已有细节时只链 Wave，不再抄命令长表。
- 区分四层：**债务状态、计划/Stage 进度、验证结论、PR/CI 状态**。PR merged 不等于父债 Done，计划 closed 不等于 Full 完成，调度 blocked 不表示产品回退。
- 只记录本轮实际核验；旧数据注明日期/SHA，遗漏历史保留 unknown，不回填伪造历史。文案更正或失败归因追加事件并链接旧件，不改冻结日志/DB/二进制。
- 同一债优先复用 ID；新反例先查归属/重叠再决定拆债。Observe/Deferred 没有排期也保留触发条件，不因巡检自动解冻或提升优先级。
- 债 Done 必须满足该范围的全部门槛和 [核实协议](../AI_VERIFICATION_PROTOCOL.md)，含项目要求的目标 SHA 远端 CI及人工/实机证据。缺任一项只记执行进展，保留父债 OPEN/Partial。
- 活跃修改由当前控制者单写；有委派时执行方返回证据和 diff，不能自行变更全局状态。单 Agent 可以完成全部职责，不强制启用子 Agent。
- 事件用稳定编号 `DCL-YYYYMMDD-NN`，按发生顺序追加；更正引用旧编号。不要为补 CI run ID 单独造提交，先记交付报告，在下一次实质变更时入账。
- 本文件不无限堆细节：较长原始记录放 `waves/` 或相应证据目录，事件保留一段摘要和链接；需要分年时保留索引，不删除历史。

## 事件格式

下面只列字段，不是已发生事件：

```text
事件 ID / 日期 / 记录者；类型（新增/进展/复核/阻断/状态迁移/更正/交接）
debt ID 与范围（Minimal/Full、实际能力、owner）
依赖关系（目标切片、类型、前置能力/版本、依据/解除条件、可先做范围；无变动时无需重复长表）
before → after（分别列债务状态与计划/调度/验证，未变写未变）
依据（base/head SHA、实际 diff、原始日志/测试/PR/CI；本机证据可携带性）
关联更新（台账 / 计划 / QUEUE / Wave 的路径；无需改的说明）
未测与残留；最后动作；下一条精确动作；retry_safe 及原因
```

## 变动事件

### DCL-20260928-01 · 初始化审查与调度对账

- **记录者 / 类型**：主控 Codex；复核＋阻断。本轮起点 `6ed1dda1f0ab5317450ce4609a6c29c47d423e50`，开场工作树干净。该 SHA [主 CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36384352172) 17/17 success 是已有工程基线，不是本轮文档提交或全部债务重新验收。
- **范围**：读取台账 §1、前瞻风险、§1.5、冻结/观测/Deferred 与历史；核对计划覆盖、五条 `pr-open`、已验 Host 恢复与当前源码。本轮未运行业务场景、真实模型/语音或硬件矩阵；没有实施任何业务债修复。
- **变动**：五条 QUEUE seq 40/50/60/120/130 的 `pr-open → blocked:needs-reconcile`；对应机器契约 `ready → blocked` 并写明确前提，`currentStage` 原样保留。K-RESILIENCE-01 的本册仍为 Minimal；K-SUPPLY-05-Full、K-CROSS-01、K-DIST-01、V-MARKET-01 的父债均保持原 Full/Partial 边界，未转 Done。计划状态头与主台账对齐；K-SUPPLY-05-Full 撤回“紧急 skip 也可 Full”这一与其既有零 skip 合同矛盾的句子，保留例外只能记 Partial。
- **依据**：[PR #126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 已于 2026-07-16 合并（`23e4e1843ddc2c3ddf3c4cfd727131950fd50c66`），其文件列表包含这五本计划及 Wave；当前 open PR 列表没有对应未合项。已合历史切片不能替代 Full、跨平台实机、签名权限或跨仓工作授权。不能继续沿用“等待同一 open PR”作为下一步。
- **关联**：[覆盖审查](COVERAGE.md) 已撤回全量覆盖声明并列出缺口；[第二轮计划](ROUND-02-PLAN.md) 的历史启动检查继续保留，本事件更新其流水线来源和对账进度。台账中的 D-HOST-RECOVERY-01 只更正当前桌面恢复的描述，未关闭未覆盖的领域/崩溃风险；D-DEBT-LEDGER-01 保持 OPEN，记录本次交接机制切片而不宣称全表规范化完成。
- **下一步**：先选择一项已授权业务债，按其 owner 和完成条件形成现行计划；对五个 blocked 项逐项确认剩余范围和证据后再解除前提。可运行 `npm run check:debt-marathon` 检查结构；这不是开工或 Done 许可。**retry_safe**：只读对账和结构检查可重跑；旧 live 身份不得重用。

### DCL-20260928-02 · 仓库流程成为正式来源

- **记录者 / 类型**：主控 Codex；规则与接手前提变更，关联 D-DEBT-LEDGER-01，业务债状态不变。
- **before → after**：本机缺少 `~/.cursor/skills/dev-pipeline/SKILL.md`，项目入口引用机器私有来源 → [仓库通用 Skill](../workflows/dev-pipeline/SKILL.md) 成为正式来源，本机只安装同字节副本。依据为维护者本轮确认“以仓库的为准，然后放到本机”。
- **关联更新**：AGENTS、AI 阅读索引、handoff 入口、项目 Skill、GATES 与马拉松入口统一引用仓库；本文件承担变动事件与续跑规则。项目 G1–G17 和架构契约没有复制或降级。
- **边界与续跑**：本次安装仅限 `~/.cursor/skills/dev-pipeline/`，不改变其他技能、权限或自动续轮配置。后续改仓库通用 Skill 时，安装副本要重新同步并核 bytes/SHA256；不可只改本机后声称团队规则已更新。本轮命令结果在交付报告登记，文件包含在里程碑提交中，无技术债 Done 迁移。
- **验证出口（Locally verified）**：文档链接默认入口与本轮显式文件、登记、旧路径和 diff 检查；债契约 12 auto plans 结构通过，五个 blocked 项逐项 `--require-ready` 均以 exit 1 拒绝；中文 UTF-8/无 BOM 核验，以及三文件安装副本 bytes/SHA256 相等。官方 `quick_validate.py` 在两个 Python 运行时均因缺 PyYAML 无法启动；改用仓库已安装 `yaml` 解析器核 frontmatter、字段/命名与未完成占位，四份相关 Skill 通过，**不声称官方脚本 exit 0**。未为此安装新依赖。
- **最后动作 / 下一步**：完成上述治理切片与本机镜像；交付给后续 Agent 时先运行 `git log -1 --format=fuller`、`git status --short --branch` 和 `npm run check:debt-marathon`，再按 DCL-20260928-01 确认一个具体业务范围。只读检查可重跑，本次记录不授权业务场景或远端写入。

### DCL-20260928-03 · 依赖登记与首批开工准备

- **记录者 / 类型**：主控 Codex；依赖复核＋计划准备。维护者要求先划清债务依赖，放入本登记，再准备开工；受检起点 `415d9592990002923ab1d2285ecad9bc998a01bd`，工作树干净。
- **before → after**：依赖分散于主台账、专题和长计划 → 本页按切片登记关系类型、依据、解除条件与可先做范围；新增关系判读/更新字段，首批安排复用 [第二轮计划](ROUND-02-PLAN.md#当前开工准备2026-09-28)。债务状态、优先级、QUEUE 和所有计划机器契约未变，不自动把建议顺序升为硬前置。
- **本轮实际核对**：主台账现行/冻结/观测/Deferred 区段、覆盖缺口、五个 blocked 计划与插件/v2 语音计划；工具链三处 `rg` 调用及 `package.json` 构建命令。现行 `E:\Env\scripts\inspect-engineering-environment.ps1` 自述 `project-neutral`，没有登记 `rg` 或真实 OCLive 门禁；本轮只读，不将其个人环境报告冒充项目通过，也不把所有项目统一改成依赖 OCLive。历史报告保留原字节。
- **剩余未知与边界**：原始外部日志的当前可取得性、真实硬件/平台条件、签名信任决策和公开接口迁移仍按对应 owner 核；未执行新业务、真实模型/语音、压力构建或缓存清理，未开启自动马拉松、未复用 run ID。依赖登记不是所有债务行为已复验的声明。
- **验证出口**：适用文档链接默认 52 文件与本轮 4 文件均通过；文档登记 26 根文件/5 哨兵、旧路径文档检查、12 auto plans 结构、编码及 diff 检查通过。结构计数不是债务总数或依赖全覆盖率；本轮无 Done 迁移，未重复全量工程/实机验证。
- **最后动作 / 下一步**：完成本页登记与相应入口/开工计划，运行适用文档和计划结构检查。下一工程切片从 `ROUND-02-PLAN` 的工具链 0/1 开始；先冻结真实调用基线及缺命令/无匹配/执行失败判据，再实施仓库范围修补。只读/静态检查可安全重跑；真实运行与已消费预算的 attempt 仍须新身份。
