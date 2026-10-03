# 技术债变动记录与 AI 接手协议

**SSOT 范围**：技术债的依赖登记、变动事件、证据关联与安全续跑坐标；不维护第二份当前债务状态表。
**最后更新**：2026-10-02。

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
| K-TOOLS-01、K-VERIFY-01 | **共因**：门禁真实调用依赖与体检口径；二者不互为整债硬前置 | 主台账 §1；两个 ratchet 实际调用 PATH 上的 `rg`，`dimension5` 调用 ratchet；其原局部 `rgCount` 无调用点，本轮移除。须区分无匹配、缺命令和执行失败；体检区分存在/可用/真实门禁通过 | 仓库内行为修补及可携带静态体检见 [工具链 Wave](waves/WAVE-20260928-TOOLCHAIN.md)；`E:\Env` 原始记录/个人脚本只读，外部修改另定范围 |
| K-BUILD-06、K-BUILD-07 | **共因 / 建议顺序**：共享构建环境；先测量，再分别评估链接与缓存策略。互不要求整债 Done | 主台账 §1、`.cargo/config.toml`、`package.json`；实测/历史纠偏与 CLI 启动器收敛见 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md)。CLI 集成脚本仍只有 `--test-threads=1`，真实生成项目构建仍在，存留 exe 数也不等于单轮链接次数 | 先核配置、逻辑与物理大小、缓存新鲜度及旧证据用途；受控复用编译和冷重建分开预算。无等价证据不移除现有 `-j 1`；清理建议不是既定白名单，不自动删缓存或 rlib 变体 |
| D-DEBT-LEDGER-01 | **协同**：支撑接手可靠性；全表规范化不是其他债实施的硬前置 | 主台账前瞻风险、[COVERAGE](COVERAGE.md)；需明确一个权威状态行与显式引用，再逐类迁移历史/补冲突检查 | 随被触及条目增量整理 owner、范围和事件；有状态冲突时只暂停受影响任务，不要求全仓先重写台账 |
| K-ENCODING-01 | **共享执行约束**：所有中文文档写入；不是“先把此债做完才可开发” | [GATES](AI_AND_PIPELINE_GATES.md) §7 的编码红线及写后检查 | 对本轮改文核 UTF-8、无 BOM、正常汉字；长期执行观察与新的自动检查范围分开 |

### 安全、供应链与韧性

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-PLUGIN-SEC-01 | Stage 3 原生隔离的证据门已在目标 SHA 完成；Stage 4 身份绑定以 K-SUPPLY-09 可信身份能力为**硬前置 / 当前暂缓的决策门** | [插件计划](long-plans/K-PLUGIN-SEC-01.md)与 [Stage 3 Wave](waves/WAVE-20260930-K-PLUGIN-SEC-01-s3.md)；Full 仍需可信签名、轮换/撤销和开发 opt-out，不以窗口隔离代替 | Stage 4 只读梳理可复用；维护者恢复签名决策前不改生产信任根或桥授权，不关 Full |
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

1. 核 `git status --short --branch`、HEAD 与任务授权；用 `rg -n '<DEBT_ID>' handoff/TECHNICAL_DEBT_INVENTORY.md` 找到当前条目。命中“引用”行时跳到其 `debt-…` 锚点；§5 与 Verification 是历史证据，不维护当前状态。不得按第一处 `Done` 判整债完成；若同一 ID 出现两个未标引用的现行状态行，先登记冲突，不自行选较新/较绿的一行。
   编辑登记后运行 `node scripts/check-debt-ledger.mjs`（只读、非零即结构异常）；`check:debt-marathon` 与 Dimension 5 同样调用这一检查。独立 ID/子 ID 表内的格式需遵守 [结构合同](waves/WAVE-20260929-DEBT-REFERENCES.md#持续登记结构门禁2026-09-29)。历史-only 行、组合 alias 和自由文本语义分别管理；结构绿不取代证据审查，也不给 QUEUE 开工许可。
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
- 每个 ID 的当前状态、优先级与完成/解冻条件只在一个权威行修改。其他章节用显式引用和稳定锚点，不复制当前状态；旧触发条件失效时登记依据，不能借历史行重开已结案上游。正文引用化和自动冲突检查分开验收。
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

### DCL-20260928-04 · 仓库工具链首批实施

- **记录者 / 类型**：主控 Codex；实施＋进展＋更正。维护者明确“开始工作”；base `32cd11a81a6fe7641b228fa8da2d39676d2791d8`，开场工作树干净。详细差量、命令、失败尝试和续跑统一见 [工具链 Wave](waves/WAVE-20260928-TOOLCHAIN.md)。
- **范围 / before → after**：K-TOOLS-01、K-VERIFY-01 `OPEN → Partial · repository slice`；两个实际 ratchet 共用必需工具/退出码处理，新增可携带静态项目体检及行为合同测试，进入 Dimension 5。父债未 Done，QUEUE/所有机器计划契约未变，五个 blocked 项不解冻。
- **更正 DCL-20260928-03 的盘点口径**：三处存在 `rg` 代码不等于三处实际调用；`dimension5` 的 `rgCount` 是无调用死函数，已移除。新增 NUL 记录解析保持匹配行计数，修正 Windows 盘符导致的 `domain/mod.rs` 排除歧义；既有 baseline 不改，正常实测仍 3/1 与 75。
- **依赖与验收边界**：仓库只承诺 Node/Git/Cargo/rg 和两个 ratchet、Git worktree、离线 Cargo metadata；工具存在/可用/实际链通过分列，未运行不记通过。个人 `E:\Env` 不入写集；新环境完整门禁、外部体检联动和目标 SHA 远端 CI 保留独立条件，不以本机静态体检关闭父债。
- **关联 / 续跑**：同步主台账与 [ROUND-02-PLAN](ROUND-02-PLAN.md#工程切片启动2026-09-28)；没有业务请求、模型或语音预算。工程原始日志在本机忽略目录，获取边界见 Wave；静态检查可重跑，失败日志不覆盖。首轮自测的环境对象断言导致非预期全环境打印，已改为只比较 PATH，保留日志对凭据脱敏并登记，不称原字节完整日志。
- **验证出口**：17 项行为合同与静态项目体检通过；综合 attempt1 因本机缺 `py` 失败，沿用脚本已有显式解释器入口、命令 finally 恢复后 attempt2 全链 exit 0。收口仅再补两处测试目录边界断言并定向复跑，不改生产门禁实现；最终字节与 Dimension 5/文档出口见 Wave。此为仓库切片 Locally verified，不是新环境、个人体检、父债 Full 或目标 SHA 远端已绿。

### DCL-20260928-05 · 构建与缓存实测、历史口径纠偏

- **记录者 / 类型**：主控 Codex；测量＋进展＋更正。维护者授权继续工作；base `d30f47c75bc9ef7454fb308204f1997c884b3422`，开场干净。复用 [ROUND-02-PLAN](ROUND-02-PLAN.md#构建成本与缓存切片2026-09-28)，全部原始依据、边界和命令只在 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md) 维护，不新建状态副本。
- **范围 / before → after**：K-BUILD-06/07 均保持 OPEN，优先级/QUEUE/机器计划契约不变；当前 target metadata 两种完整枚举精确相符，逻辑大小/硬链接/声明目标和构建配置分列。单 Host 编译目标的同参完全复用观测取得；workspace 与单目标短预算各一次 deadline 非零，未被换成绿结论。未人工清缓存、未改变 linker/jobs/profile、生产行为或依赖。
- **更正**：CLI 集成只限制测试线程，不是编译 `-j 1`；旧 27.3 分钟含冷 incremental 和删错 rlib 后恢复，不能全归因串行；存留 exe 数不等于一轮链接次数。撤回按月清 build/fingerprint/pdb 为既定白名单的口径，要求先核新鲜度/变体/实际分配与冻结用途。外部清理原件和四份 7 月错误日志不改。
- **自主调整与证据状态**：按止点终止所持 Cargo 子树并保留原 native/采样日志；收尾未见编译器存活。两次短预算不足后，在同一切片追加一次定额预算复评，再于独立输出完成单 Host 目标及短复测；并发与编译配置保持。单目标包含实际 Tauri command 特性，不冒充纯小 Kernel 或完整 workspace。此为测量进展，未完成链接峰值归因或维护策略验收。
- **适用检查 / 未测**：默认及改文链接、docs 旧路径、登记、债计划结构、编码和 diff；无生产或门禁源码改动，不重复工程全链/业务场景。物理可回收量、真实峰值/逐 target response-file、完整冷热对照仍未知；新场景、真实模型/语音、硬件及架构变更不随本记录授权。
- **CI 与接续**：上批工具链目标 [CI 36403228999](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36403228999) 本轮取证时尚未结束，不声称全绿或父债 Done。其 workflow 同分支新 push 会取消旧运行；新文档本地提交可先保留，待该目标结束再批量同步。下一步先核 CI/干净基线与 Wave 实际单位图、新鲜度证据；只读可安全重跑并使用新输出名，编译/实验不覆盖本轮日志，不做无依据冷重建或删除。

### DCL-20260928-06 · CLI 集成测试直接使用本轮二进制

- **记录者 / 类型**：主控 Codex；实施＋进展。维护者授权继续收敛；base `6c28b1e2e5cb42214120e7a5903c85c4420e5365`，开场干净；M 级合同见 [ROUND-02-PLAN](ROUND-02-PLAN.md#cli-集成启动切片2026-09-28构建债接续)，实际源码/测量/命令只在 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md#cli-集成测试启动器收敛2026-09-28) 维护。
- **范围 / before → after**：K-BUILD-06 的测试启动子问题收敛，K-BUILD-06/07 均保持 OPEN；六测试文件的旧 `cargo run` 自身启动统一为本轮 `CARGO_BIN_EXE_oclive-cli`。生成项目的真实编译、bench 夹具、原业务参数与拒绝断言保留；package/CI、默认特性、生产路径、并发配置、依赖和旧冻结证据不改。
- **关联 / 安全边界**：复用现有 scaffold/registry 测试机制和 `tests/common` 路径 owner；运行用独立 child `OCLIVE_HOME` 隔离注册表，离线设置不改父环境。已核对 Kernel/Host/Tauri/shared/角色包无需改；保留 Git 提交清单溯源，未引入新公开 API、安全选择、运行时旁路或实际缓存删除。
- **本地出口 / 剩余**：四目标前后测与新增缺 Cargo PATH 的真实进程负控通过；CLI crate 全测、all-targets/all-features clippy 和最终适用检查在 Wave 分列。测量出现不同依赖重编，不能把测试执行段下降夸大为整体编译加速；链接输出/峰值、物理分配、保留策略仍待受控证据，不移父债 Done。
- **CI / 接续**：上批工具链 exact SHA CI 已 success，新基线文档 CI 与本切片未来目标 SHA 分列，不能借父绿证明本轮。冻结后一次提交/推送；业务回合身份不存在，纯静态可新日志重跑；源码未再变时不重跑无关全仓验证，下一项链接/缓存实验另定输入与预算。

### DCL-20260928-07 · 单 Host 链接资源样本取得目标归属

- **记录者 / 类型**：主控 Codex；有界测量＋进展。base `787aa5cbbfa7b1aa2a600e8bf63cf2ac8e3df1cb`，开场干净；新合同见 [ROUND-02-PLAN](ROUND-02-PLAN.md#单-host-链接目标绑定2026-09-28诊断接续)，原生/参数/样本/身份与读取层更正只在 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md#单-host-链接目标绑定2026-09-28) 维护。
- **范围 / before → after**：K-BUILD-06 单目标“linker 样本无输出归属”缺口已补；唯一 compile-only 原生完成，实际 `/OUT` 与 Cargo artifact 精确绑定，并保留 response-file 只读快照。K-BUILD-06/07 状态、优先级与队列不变；不推出完整矩阵真峰值、原 OOM 根因或替代配置已通过。
- **依赖 / 关联 / 自纠**：保持 12 份源码/构建输入及原 target、串行编译、Git 清单溯源；不强制源码失效、不删缓存、不运行 harness/业务回合。离线夹具数组少两项、派生 debug flag 行首读取错误均保留旧件并定向更正，不改原始数据或重跑 live。
- **验证 / 接续**：采集器仅为本机一次性工具，8 项真实函数离线探针通过；最终本轮文档出口另验，CLI 已测字节不变，不重复无关 Rust/业务链。后续为明确单项候选作有界对照或逐路径保留核验；旧失败不覆盖、实际峰值未知不默选 linker/profile/并发策略，目标 CI 继续独立绑定。

### DCL-20260928-08 · 缓存元数据按文件对象去重

- **记录者 / 类型**：主控 Codex；有界只读测量＋进展。base `83af77bb74daf9eed6092770b80824c5272ce1f8`，开场干净；合同见 [ROUND-02-PLAN](ROUND-02-PLAN.md#缓存分配与文件身份去重2026-09-28只读接续)，原始逐项账本/夹具/接口与剩余边界见 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md#缓存分配与文件身份去重2026-09-28)。
- **范围 / before → after**：K-BUILD-07 的“仅路径逻辑大小/硬链接数”推进为 NTFS 文件身份去重及 Win32 默认数据流报告分配量；独立账本重算与实际枚举一致。K-BUILD-06/07 仍 OPEN，优先级、队列与维护准入条件不变；不将路径重复量当可回收量，不将采集器提升为 doctor/CI。
- **证据 / 依赖**：真实普通/空文件、硬链接、根外链接和越界拒绝 11 项夹具通过；扫描 5.68 s、无错误/预算截断，独立枚举路径、大小、硬链接数与修改时间全部相符。12 份源码/构建输入与限定父环境保持，编译器前后点检查为空；没有命名流/卷级或原子快照证明，不外推永久无根外链接。
- **适用检查 / 下一步**：仅本轮文档出口，不重复已测 CLI 或生产全链；逐路径保留用途、冻结证据关系和重建代价仍须另核，未批准删除/压缩/硬链接改写或冷重建。静态可新日志安全重跑；目标 SHA CI 与父绿分列，先分类提交，在父运行终态后按已有授权同步。

### DCL-20260928-09 · 缓存用途与保留准则落文

- **记录者 / 类型**：主控 Codex；有界只读关联＋规则收敛。base `2da472cae97638af9cd4141bf4001f9b3352dfb4`，开场干净；该 exact SHA [CI 36416485540](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36416485540) success、17/17 jobs 与 ci-gate 通过。本轮未来提交不得借父绿验收。合同和事实分别见 [ROUND-02-PLAN](ROUND-02-PLAN.md#缓存用途与保留准则2026-09-28元数据接续)、[构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md#缓存用途与保留准则2026-09-28)。
- **范围 / before → after**：K-BUILD-07 仍 OPEN；旧元数据按目录用途分组、共享文件对象去重，991 显式引用逐项关联；四类保留与维护准入落文，Windows 中英指南不再默认全量清理、按名杀进程或全目录安全软件排除。没有生成删除白名单、改编译器配置或提高维护权限；优先级、QUEUE 与机器契约保持。
- **历史证据边界**：10 个可达保护源只读，未作为全链验收。head34 的日常 target 路径字节已变，但独立冻结程序副本精确匹配原 hash；保留旧头原文，不补造原链当前通过或推断哪次构建改变。共享对象与未引用路径仍按实际用途未知处理，未做删除/搬移/压缩。
- **关联 / 接续**：主台账、计划、Wave 与 Windows 镜像同步；原始/派生/身份件在本机忽略目录，缺件仍 `needs-evidence-access`。后续实际维护另核逐对象别名、完整保全与有界重建；本轮静态检查完成后分类提交，链接候选的实验事实另记独立事件，不混作缓存维护通过。
- **本地出口**：默认/改文链接、docs 旧路径、登记、镜像、债结构、diff 七项 native exit 0，六文 UTF-8 无 BOM/无替换符；原 warning/stderr 保留，不以纯文档检查冒充完整工程或历史链验收。

### DCL-20260928-10 · 单 Host 稳定 LLD 候选取得限定对照

- **记录者 / 类型**：主控 Codex；有界实验＋进展＋自纠。测量固定 base `2da472cae97638af9cd4141bf4001f9b3352dfb4`；仅本轮文档 dirty，生产/测试源匹配该 base。保留规则已另提交 `96b67fd4`；命令、原生日志、资源样本及限制只在 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md#单-host-末端-lld-候选2026-09-28) 维护，不借父 SHA CI验本轮未来提交。
- **范围 / before → after**：K-BUILD-06 单 Host test/debug/tauri-commands 取得一次默认 MSVC 与稳定 LLD 末端候选对照，两份当前实际清单均 94、默认测试各 75 passed / 19 ignored。440 非末端 artifact 元数据相同且候选全 fresh；K-BUILD-06/07、队列/优先级均不变，默认 linker/profile/并发/生产 API与 CI组合未改。
- **自主修正与首因**：`--version` 不证明真实参数接受性，初版 B 的 `msvc-lld` 被稳定编译器拒绝（101）并停止；原件保留。核官方稳定 `lld-link` 接口并通过真实 metadata 正负控后，追加一次120 s候选成功，没有 unstable/nightly/BOOTSTRAP 或重复 A。第一次辅助比较沿用89而非94、控制端未按其失败先停的读数/编排缺口明确入账；实际默认执行没有 ignored/live，不把旧失败改绿。
- **取舍 / 剩余**：绑定 linker 样本的工作集/私有内存更低，LLD PDB更大；不是全矩阵真峰值、统计加速或调试器等价证明。两独立 binary/PDB累计低于原预算，12 输入/两个受保护程序/限定父环境不变，收尾 compiler点检查空。旧所有程序/记录/树/DB不改，未清缓存或安装依赖。
- **出口 / 下一步**：同步主台账与合同的当前清单口径，适用本轮文档/债结构/编码/diff；不重复无关全工程或真实业务。分类提交、在前批 exact CI 已结束后按已有授权批量推送；新目标 CI独立绑定。候选仅证明该目标边界，采用前另核明确目标/特性和 PDB/运行行为，不自动把它设为全仓默认。

### DCL-20260929-01 · 已冻结 Host PDB 的离线消费通过

- **记录者 / 类型**：主控 Codex；限定验证进展。开场 `c60a04c99459025f7279a923676b1397fa9c8ba6`、工作树干净，该 SHA CI 17/17 已成功；本轮新提交独立绑定。仍消费 `2da472ca` 构建的固定产物，Host 源码相对该 base 未变。合同见 [ROUND-02-PLAN](ROUND-02-PLAN.md#已冻结-host-pdb-的离线消费2026-09-29)，唯一事实详表在 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md#已冻结-host-pdb-的离线符号消费2026-09-29)。
- **范围 / before → after**：此前仅证 PDB 文件与默认行为；现在已有系统 DbgHelp 对 MSVC/LLD 精确 PDB、所选真实函数和源码行的消费，以及缺失/错配 PDB 两真实负控。四 helper native 0、独立核算通过；K-BUILD-06/07 仍 OPEN，不改优先级/QUEUE、构建入口/profile/linker/并发或生产 API。
- **限制 / 自主行动**：有限范围未找到 CDB/LLDB，改用已有 DbgHelp 做静态窄测；没有安装工具、执行受测 exe、附加进程或重编译。真实负控只新增普通副本，原件和源码 hash 不变；模块登记成功与符号实际加载分开，宽掩码闭包与唯一完整函数计数分开。没有真实断点/局部变量/栈证据，不能据此认定完整调试器等价。
- **接续 / 出口**：按现有授权同步四篇 owner 文档、适用文档/债结构/编码/diff 后分类提交推送；新目标 SHA CI 与本机静态消费分列。采用前只选一个具体入口/特性及调试取舍；未知原件可携带性仍 `needs-evidence-access`，不启动全局 LLD、无界矩阵或缓存删除。

### DCL-20260929-02 · Actions 固定来源与升级维护切片

- **记录者 / 类型**：主控 Codex；实施与验证进展。维护者授权持续收敛；base `e71e1c5fcd48fdb931a8e7c110bcafda4113af83`，开场干净，L 级范围见 [ROUND-02-PLAN](ROUND-02-PLAN.md#k-supply-10--仓库-actions-固定引用2026-09-29)，来源/差量/门禁只在 [Actions Wave](waves/WAVE-20260929-ACTIONS-PINS.md) 维护。
- **范围 / before → after**：K-SUPPLY-10 `OPEN → Partial · implementation awaiting verification`；72 处仓库引用和 14 处实际 CLI 生成模板固定原上游完整 SHA，新增 workflow 的兼容周更；主版本、CI 权限/触发/执行策略、stable 编译器选择、锁文件、公开 API 和构建并发均保持。QUEUE、优先级与其他父债不变。
- **证据与自主修正**：Rust Action 选可达 master 历史提交并显式 stable，以 metadata 深比较保留原运行步骤；一次网络 EOF 独立重试、一个新增测试换行定点格式化，失败均留原件。结构正例与 9 项变异负控、真实生成器 3 项通过。全链 attempt1 在 `oclive_ci_plan` 的旧 tag 断言 native 101；追加该测试到写集、定向修补复测 5/5，混合换行另以 YAML/文本等价并保留未改行处理。attempt2 在该新断言的 rustfmt 样式 native 1，只格式化该文件后定向 fmt native 0。第三轮全链与目标 SHA CI 待独立取证，不以结构绿代替外部执行。
- **维护边界 / 续跑**：Dependabot 不扫描 Rust 模板字面量，相关升级需同步生成器；实际 bot PR 与新目标 CI 尚未证明，父债不转 Done。第三轮全本地链 `check:ci-local` 原生 0 / 744.91 s，28 项收尾字节匹配；最后验证文字另补文档 ratchet。文档中英/台账/入口/本计划同轮同步，待父 CI 终态再推送，不取消已有验证或为结果造证据专用提交。没有发布、真实模型/语音或 CP-INT 身份消费。

### DCL-20260929-03 · 重复债务行改为权威引用

- **记录者 / 类型**：主控 Codex；治理实施＋读取层更正。base `245d6ca98edfa8ebb354e2609d556a455336cc73`、开场干净，M 级合同见 [ROUND-02-PLAN](ROUND-02-PLAN.md#d-debt-ledger-01--重复状态改为显式引用2026-09-29)，逐项 owner 与证据仅在 [台账 Wave](waves/WAVE-20260929-DEBT-REFERENCES.md) 维护。
- **范围 / before → after**：D-DEBT-LEDGER-01 `OPEN · governance slice added → Partial · duplicate references normalized`。18 个重复 ID 取得明确权威行，20 处重复行改为引用/历史引用；已有 Minimal/Full、Observe/Deferred、冻结及结案裁定不变，优先级、QUEUE 与机器计划均未调整。长 Verification 迁移、全表状态词规范化及持续自动冲突检查仍缺，不升父债 Done。
- **依据 / 自主修正**：初次临时 ID 正则漏掉 O-1/O-2，完整格式复核后补齐；首次批量补丁因历史行文字与上下文不符被原子拒绝，原台账 hash 未变，再按实际原行生成限定补丁。18 个权威状态/解冻字段原样、132 个未触及 ID 行原样、20 段 Verification 原样；只吸收三项既有触发说明并将 D-PORT-03 的过时上游解冻引用指回现行冻结条件，不作架构决策。
- **关联 / 出口**：主台账、接手协议、COVERAGE、计划与入口同轮更新；独立静态审计、适用文档/债结构/编码/diff 逐项留原生日志。没有新增 CI 门禁、改生产源码或重跑真实业务；本机忽略证据不可取得时仍 `needs-evidence-access`，不能把本轮静态结果当全债验收。
- **接续 / CI**：等前批 Actions exact SHA CI 终态后再推送本轮文档，两个目标分别取证；新文档不借父绿声称当前 HEAD 已绿。下一步可逐类迁移长历史或明确持续检查范围，不能默认解冻全部债务。只读审计与文档检查可另名重跑，旧证据不改。

### DCL-20260929-04 · 七月 Verification 历史迁移

- **记录者 / 类型**：主控 Codex；限定文档迁移。base `5cd50d346f5fab351357af388f8e43f110b65696`、开场干净，上一引用化切片已本地提交；M 级范围见 [第二轮计划](ROUND-02-PLAN.md#d-debt-ledger-01--七月-verification-迁移2026-09-29)，细节复用 [台账 Wave](waves/WAVE-20260929-DEBT-REFERENCES.md#七月-verification-迁移2026-09-29)。
- **范围 / before → after**：主台账首部 20 段七月 Verification 改为单一历史链接；归档按原顺序保留文字、旧 SHA/CI 与范围，仅重定位三类相对链接的五次出现到原目标。D-DEBT-LEDGER-01 保持 Partial 并补进展；其它状态/条件、ID/锚点/引用、QUEUE 和机器计划不变。首部其它长快照、§5 和状态词规范化未迁，不把旧绿灯扩成当前验收。
- **证据 / 出口**：六份原文冻结于独立忽略根，机械迁移审计另核所有历史正文、外部 URL、相对目标与主表未触及行；适用七篇改文链接、路径/登记/债结构/编码/diff。没有生产代码或自动门禁改变，不重复工程/业务链。原始证据取不到仍 `needs-evidence-access`，旧日志、候选与 DB 不改。
- **接续**：先冻结本轮文档身份、本地分类提交，再在前批 Actions CI 终态后一次同步里程碑。持续冲突门禁另定真正的解析合同和反例，不将本机一次性脚本默认为新工程门禁；父债不转 Done。

### DCL-20260929-05 · 首部旧结论标明历史范围

- **记录者 / 类型**：主控 Codex；S 级文案澄清。base `d725a3af50d7f6ab532980eca66a58b20960fe79`、开场干净。首部产品冻结、综合评分和 R18 下一动作仍是旧快照，容易误读为本轮质量调查或当前排期；仅改三个标签为显式历史，全部正文、表格状态/条件及七月归档原字节保留。
- **出口 / 边界**：本机独立根 `.cursor/plans/debt-ledger-labels-20260929-r0/` 冻结原文，限定标签替换比对、默认/两篇改文链接、路径/登记/编码/diff 适用；不重跑业务或调整架构，D-DEBT-LEDGER-01 保持 Partial。本地等待 Actions CI 的工具达到 600 s 预算后仅结束自身，远端 run 仍在执行，不能把等待工具的非零记为 CI 测试失败。
- **同步节奏**：前批 main CI 在终态前不被新 main push 取消；文档里程碑可按既有授权保全到无 PR 的独立分支，main 同步及其新 SHA 验证随后分开办理。不声称本轮或旧历史证据已覆盖全部债务。

### DCL-20260929-06 · 独立 ID 行与引用的持续结构门禁

- **记录者 / 类型**：主控 Codex；M 级治理实施。base `b3b2fef5774ff84985b8ccea705c9b302b55e41e`，开场干净；合同与失败/修补事实唯一见 [台账 Wave](waves/WAVE-20260929-DEBT-REFERENCES.md#持续登记结构门禁2026-09-29)。新检查函数由独立 CLI、债计划入口与反例共用，Dimension 5 原债计划步骤加入回归；主台账仍是唯一状态 owner。
- **before → after**：一次性静态保全检查变为持续登记结构检查；当前独立 ID 重复、锚点/引用错配、历史标签错位和直接状态另写会被拒绝。历史-only 记录、现行与历史引用明确分开，既有组合标签只作为有限例外。补齐 17 行表头/占位几何，不重判旧单元格裁定；D-DEBT-LEDGER-01 留 Partial，自由文本语义、其余长历史和状态词另定范围。
- **自主处理 / 影响链**：首轮正例失败暴露旧表列数不一，失败日志保留后只补位置，不删规则；调用点审查发现马拉松测试临时复制缺少新库/台账，补真实依赖和启动拒绝负例。实际旧队列与机器计划不动，无新增领取/调度授权；未改产品、生产数据库、旧 CP-INT 证据或架构。
- **出口 / 接续**：本机忽略根 `.cursor/plans/debt-ledger-check-20260929-r0/` 保留原文、原生 stdio/退出与状态/历史不变核对；适用定向反例、Dimension 5、文档/编码和差量检查。冻结后分类提交，父 Actions CI 终态前不取消其运行；新提交的远端结论单独绑定，不借父绿也不以本地结构绿关闭父债。
- **独立调度诊断**：`--assert-no-runnable` native 1，列出已有 `K-PLUGIN-SEC-01:s3`，不是新门禁或产品失败。QUEUE `implemented` 与计划 `ready/currentStage=3` 使其形式上 runnable，但台账仍 Partial 且 Windows 原生与可信身份绑定未完成。本切片不擅自改机器计划或激活 Stage；后续先按原计划与台账复核其可执行范围，再决定调度。此前概括“没有 runnable”以这次实际命令为准更正。

### DCL-20260929-07 · Actions 固定引用目标 CI 终态入账

- **记录者 / 类型**：主控 Codex；前一实质供应链切片的远端证据接续，详情只见 [Actions Wave](waves/WAVE-20260929-ACTIONS-PINS.md#验收出口)，当前状态仍由 [主台账](../TECHNICAL_DEBT_INVENTORY.md) 的 K-SUPPLY-10 行唯一维护。
- **目标绑定**：固定实现 `245d6ca98edfa8ebb354e2609d556a455336cc73` 的 [ci.yml run 36528350305](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36528350305) 正式 `completed/success`，17/17 job 含 `ci-gate` 全部成功；原始只读 JSON 和哈希在 Actions Wave。后续整理与结构检查没有改该固定引用实现，但仍需各自新 SHA 的远端结果，不借旧绿。
- **范围 / 保留**：K-SUPPLY-10 从本地已验的 Partial 更新为“固定引用远端已验、维护待证”的 Partial。Dependabot 实际 PR 与 CLI Rust 模板的同步闭环未证，父债不标 Done；不扩大为 runner、Action 传递依赖、发布或未来更新时间点全部可信。

### DCL-20260929-08 · 八月工程收口快照移出当前台账首部

- **记录者 / 类型**：主控 Codex；D-DEBT-LEDGER-01 的限定历史迁移。base `2852b285f6a0b33dbd73aba63037ac6c5f85680a`，开场干净；M 级范围见 [第二轮计划](ROUND-02-PLAN.md#d-debt-ledger-01--八月工程快照归档2026-09-29)，迁移映射和审计见 [台账 Wave](waves/WAVE-20260929-DEBT-REFERENCES.md#八月工程快照归档2026-09-29)。
- **范围 / before → after**：首部连续八月工程、产品、评分与旧下一动作块移入 [历史归档](../archive/TECHNICAL_DEBT_CLOSEOUT_SNAPSHOTS_202608.md)，原位仅留非当前 truth 的入口。旧 SHA/CI、产品冻结、结论与排期原文保留，三个相对链接只按目录深度重定位到原目标。§1–§5 的其它内容、唯一权威状态、QUEUE、机器计划和产品代码不因此改判；D-DEBT-LEDGER-01 保持 Partial。
- **出口 / 保留**：独立 S0 保存七份原文；限定逆向重定位逐字节比较、相对目标和外部 URL、主台账其余行、适用链接/路径/登记/债结构/编码/staged diff 逐项核。旧七月归档不动，不重跑业务。父 `2852b285` 的 Actions CI 终态和本轮新目标分别取证，不借旧绿；归档不证明当前 HEAD 产品验收。

### DCL-20260930-01 · K-PLUGIN-SEC-01 Windows 原生隔离取证与测试判据修正

- **记录者 / 类型**：主控 Codex；Stage 3 限定原生取证与回归修正。base `44d952b3711c646b903333d65a116ed2a7db6366`，开场工作树干净。原始 attempt、二进制身份、通过与失败边界只见 [Stage 3 Wave](waves/WAVE-20260930-K-PLUGIN-SEC-01-s3.md)。
- **before → after**：Windows 原生 WebDriver session 未建立，不得计作产品失败；同源码测试变体的 WebView2 CDP 已验证宿主/另一插件 DOM 不可读、直接 IPC 被拒与 broker 正向引导。旧“插件 iframe 中 `__TAURI_INTERNALS__` 必须不存在”判据与 Windows 实况不符，现改为实际授权拒绝。生产权限/协议未变，K-PLUGIN-SEC-01 仍 Partial，Stage 3 未越级，Stage 4 身份绑定仍待 K-SUPPLY-09。
- **出口 / 下一步**：本地适用门禁与目标 SHA 远端原生回归分别核；后者未终态前不写 Done/Stage 4。前一 `44d952b` 主 CI 也须等独立终态，不能拿其结果验本轮未来提交。

### DCL-20260930-02 · Stage 3 远端验证完成，Stage 4 按维护者决策暂缓

- **记录者 / 类型**：主控 Codex；K-PLUGIN-SEC-01 阶段证据与执行准入同步。`1153dc2cc3c26d9e004fa08ffca624dbb15d087f` 已推送 main；该精确 SHA 的 [主 CI 36605083928](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36605083928) `completed/success`，17/17 job 含 `ci-gate` 成功；[Nightly 36605066991](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36605066991) `e2e-tauri` 与原生 WebDriver step 成功，更新后的插件隔离用例实际 1 passed。原生平台细节与 Windows CDP 限定证据只见 [Stage 3 Wave](waves/WAVE-20260930-K-PLUGIN-SEC-01-s3.md)。
- **before → after**：Stage 3 从“本机补充已验、目标远端待验”进入目标 SHA 限定远端验证完成；父债仍 Partial。维护者明确选择暂缓 K-SUPPLY-09 签名架构、先推进其它债务；机器计划移到 Stage 4，`planStatus=blocked`，队列设为 `blocked:signing-policy-deferred`，避免自动领取。解除条件是维护者恢复可信发布者身份、生产验签、轮换/撤销和开发 opt-out 的决策；不借 Stage 3 绿灯修改信任根或宣布 Full Done。
- **接续 / 保留**：此次状态同步是新的文档提交，其目标 CI 与 `1153dc2c` 的实质测试提交分列；后者成功不自动证明新 HEAD。Windows `tauri-driver` session 未建立、Windows CDP 仅限定取证；旧失败、原始 attempt 与其它债务裁定不变。签名暂停不阻断独立债务，马拉松若无 runnable auto 则按队列纪律停止自动领取。

### DCL-20260930-03 · K-RESILIENCE-01 Minimal 历史计划对账收口

- **记录者 / 类型**：主控 Codex；历史 Minimal 计划与机器队列对账，不实施新 ResilienceLayer。起点 `1d390bc5fbee256721b421e94ceb3c4b679402cc`，工作树干净；旧 [Stage 3 Wave](waves/WAVE-20260716-K-RESILIENCE-01-s3.md) 保持历史原文。
- **依据**：[PR #126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 于 2026-07-16 合入，merge commit `23e4e1843ddc2c3ddf3c4cfd727131950fd50c66` 是当前 HEAD 祖先；`REMOTE_PLUGIN_PROTOCOL` 的调用点清单、`prompt_http` 使用 `call_with_builtin_fallback` 的示范接线与 adapter 回退测试仍在当前源码。`cargo test --locked -p oclive_kernel_host remote_plugin -j 1` 本机 exit 0：29 passed，0 failed，未运行 ignored 场景。
- **before → after**：本册 `planStatus=blocked`、队列 `blocked:needs-reconcile` → `planStatus=closed`、队列 `done`，只表示历史 **Minimal 计划**已完成且无需再次领取。主台账 K-RESILIENCE-01 仍为 **Partial**，Full ResilienceLayer 仍 OPEN；不以队列 `done` 代替父债 Done，不启动 Full 的 RFC、设计或写集。
- **出口 / 保留**：产品源码、既有 Wave 与主台账事实均未改；仅同步计划、队列和本事件。文档提交须按自身目标 SHA 单独取得门禁结果，不能借既有代码或父提交的 CI 绿灯。

### DCL-20260930-04 · K-CROSS-01 按声明能力划分宿主验收

- **记录者 / 类型**：主控 Codex；维护者确认跨平台宿主接入按**已声明能力**验收。起点 `b3ec76d24bd712ca4421def24c5ecbed0bc3a4aa`，工作树干净；[PR #126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 的历史 Minimal 文档与 Wave 原文不改。
- **范围更正**：旧的“Windows/Linux/macOS 各跑完整内置 ASR→chat→TTS”不再是 K-CROSS-01 的统一结案前置。Windows 已声明支持的内置路径需正向设备 smoke；Linux/macOS 当前标为 `unsupported` 的内置路径需宿主/界面明确拒绝或禁用、无误启动的符合性证据。`HostProfile` 的发行版模块配置与 OS 语音支持矩阵仍分开。用户自建外部语音端点不等于内置能力交付。
- **归属**：K-VOICE-03 仅负责 Linux/macOS bundled CosyVoice **TTS** 产品化；同平台 sherpa **ASR** 仍须独立界定产品范围，不借 K-VOICE-03 或 K-CROSS-01 假结案。现行平台能力见[语音轨道](../../human-docs/team/TRACK_VOICE_RECOGNITION.md#平台--asr--tts--webview差异声明)；新符合性测试尚未执行。
- **before → after**：历史 K-CROSS Minimal 计划 `blocked:needs-reconcile` → `closed` / 队列 `done`，只表示已合入的文档里程碑无需重领；主台账父债仍 **Partial**，新范围的真实宿主证据未齐。`npm run test:distro-profile-mirror` 本机 exit 0 只证明 VS Code/desktop profile 镜像，不证明 OS 语音。旧 Wave 和原始实机缺口叙述保留为当时判断，本事件是现行范围裁定；后续执行需另冻新计划、身份与预算。

### DCL-20261001-01 · 文档声明支撑调查首批入账

- **记录者 / 类型**：主控 Codex；按 [Document → Code 预算与停止线](../AI_VERIFICATION_PROTOCOL.md#doc-code-support-audit) 完成首批只读分类。起点 `60249dd9a953432661805d5bc67b3777dbb4f5bc`，工作树干净；本批候选池 6 条，入选 4 条、已查 4 条、`Unknown` 0 条、未入选 2 条（K-UID-DEFAULT-02、K-AGENT-MERGE-01 已有明确 OPEN 行，本批不重复证明）。仅选活跃能力/门禁声明及一个已发生的 CI 反例；Full 目标、历史快照与其余台账事项未枚举，不计作已审。D1 每条未超 15 分钟；CI 因原生失败进入 D2，未开启 D3。未重跑业务、实机语音或 CI。

| 当前声明与责任层 | 实现入口、直接证据与调查深度 | 分类及下一动作 |
|----------------|----------------------------|----------------|
| [K-CROSS-01 平台能力](../../creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md)；Chat Pro Host / 官方语音插件 | `asr_profiles.json` 将 Linux/macOS 内置 ASR 标为 `unsupported`，`rpc_server.mjs::resolvePlatformProfile` 返回 `unsupported_platform`；已有 profile 镜像测试只证配置镜像，缺 Windows 正向及 Linux/macOS 宿主/界面实机符合性。D1。 | **Evidence gap**；归既有 K-CROSS-01，不把静态拒绝代码写成三 OS 已验。 |
| [K-RESILIENCE-01 Minimal](../../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md#宿主弹性代码锚点minimal)；Kernel Host | `remote_plugin/adapter.rs::call_with_builtin_fallback`、`prompt_http.rs` 调用与开/关闸单测对应；此前 `remote_plugin` 定向 29 passed，当前 HEAD 的 [主 CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36741793963) 17/17 success，但本批未重跑定向测试。D1。 | **Supported（仅 Minimal）**；Full 被文档明确标 OPEN，仍归既有 K-RESILIENCE-01 实现债，不能由本分类转 Done。 |
| [K-SUPPLY-10 Actions 维护](../../creator-docs/security/SUPPLY_CHAIN.md)；仓库 workflow / CLI 模板 | `.github/dependabot.yml` 仅扫描 workflow；`oclive-cli/src/ci_cmd.rs` 模板固定独立 SHA，源文已明示人工同步与实际 bot PR 待验。D1。 | **Evidence gap（维护闭环）**；归既有 K-SUPPLY-10。v7 仓库 workflow 与 v4 CLI 模板是原冻结时即存在的不同版本，不能仅凭版本号判新漂移。 |
| [ci-gate 计划证据门禁](../AI_VERIFICATION_PROTOCOL.md)；CI control plane | `ci.yml` 上传/下载证据名都包含 `github.run_attempt`，而 `ci-rerun-flake.yml` 调 `gh run rerun --failed`。真实 [run 36713345514 attempt 2](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36713345514/attempts/2) 中 `ci-impact-plan=success`、责任组成功，但 `ci-gate` 下载 `...-2` 报 artifact not found；同 SHA 全量 attempt 3 成功。D2（具名重跑反例）。 | **Semantic drift / 已发生的 CI 语义缺陷**；归 K-CI-IMPACT-01 的 gate 证据路径，单独修复切片须确保失败作业重跑时仍绑定同 run、同 SHA 的有效计划，不以宽松匹配或跳过计划校验换绿。 |

- **入账规则与止点**：文档领先代码的 `Implementation gap`、代码有而缺证据的 `Evidence gap`、行为不符的 `Semantic drift` 均先归现有 owner；新债须按协议取得 L3 入账证据并去重，不把 `Unknown` 升为 OPEN。已明确为未来目标的 Full/产品化范围仍保留原债，不伪装为当前能力。下一工程切片优先处理已发生且不需架构裁定的 CI 重跑证据缺陷；K-CROSS 实机条件、K-RESILIENCE Full 语义和 K-SUPPLY-10 bot 更新各守自己的准入条件。此事件是调查分类与归属，主台账现行状态、优先级和 QUEUE 均未改变。

### DCL-20261001-02 · K-CI-IMPACT-01 失败作业重跑证据绑定修补

- **记录者 / 类型**：主控 Codex；主分支 CI 门禁切片，尺寸 L。base `e0afbf4b6f0ea35368114207e492248c3ae60494`、开场工作树干净；方案与止点见 [第二轮计划](ROUND-02-PLAN.md#k-ci-impact-01--失败作业重跑的计划证据绑定2026-10-01)。只改 `.github/workflows/ci.yml` 与 `oclive_ci_plan` 仓库合同测试，不改规划器、选择规则、责任组、权限或受信基线。
- **问题 → 修改**：原 `ci-gate` 用自己的 `github.run_attempt` 找计划 artifact；失败作业重跑不一定重跑已成功的计划作业，因而误找不存在的当前 attempt 产物。现由 `ci-impact-plan` 把其实际 `GITHUB_RUN_ATTEMPT` 写入 job output，gate 以 `needs.ci-impact-plan.outputs.artifact_attempt` 精确下载同 run 的计划。计划失败、输出缺失或产物缺失仍 fail closed；Compare 的当前 attempt 采样口径未被改写。
- **验证**：新增合同回归先对旧工作流失败，再对修补后工作流通过；`cargo test --locked -p oclive_ci_plan --test repository_contract` 6 passed。CI execution-policy 与 Compare collector 自检通过；`cargo fmt --all -- --check`、`dimension5 --ci` **29/29**、`npm run check:ci-local` 均 exit 0。首次 Dimension 5 失败系本机无 `py` 启动器；用已存在的 Python 3.12 可执行文件仅在该命令子进程设置 `OCLIVE_VOICE_PYTHON` 后，语音 ratchet 与完整本地链通过；未改语音代码或门禁。YAML 解析确认计划输出、上传名与 gate 下载名按预期连接，`git diff --check` 通过。
- **现行结论 / 待证**：本地 **Locally verified**，K-CI-IMPACT-01 父债仍 In progress；目标 SHA 的正式 `ci.yml` 与“只重跑 gate、计划作业不重跑”的原生路径尚未取得，不能据此称重跑缺陷 Done。前一文档提交的主 CI 未终态前不推新 main，以免取消原运行；旧 attempt 与 artifact 不动。若目标运行可对成功 gate 作单作业重跑，再核计划 output 仍指原 attempt、下载成功、门禁成功和 Compare 限定口径；否则保留原生证据缺口，不构造假通过。

### DCL-20261001-03 · K-CI-IMPACT-01 重跑绑定取得原生证据

- **记录者 / 类型**：主控 Codex；接续 DCL-20261001-02 的目标 SHA 远端验收，不改该事件的本地待证快照。修复提交 `2a9c02d059c7b6421d2e76078a4aff92d3e83afa` 的 [主 CI run 36758789418](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36758789418) 首次运行 `completed/success`，17/17 job 成功，计划产物为 `oclive-ci-impact-plan-36758789418-1`。
- **单作业重跑**：对首次成功的 `ci-gate` job `110058351439` 执行 `gh run rerun 36758789418 --job 110058351439`，命令 exit 0；[attempt 2](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36758789418/attempts/2) `completed/success`。新 gate job `110058842004` 的原始日志显示 `needs.ci-impact-plan.outputs.artifact_attempt = "1"`，下载的恰是 `oclive-ci-impact-plan-36758789418-1`（artifact `11117743543`），下载、责任组核验、Compare 收集与上传步骤均 success。计划 job 在 attempt 2 的 API 视图沿用首次时间戳，未见第二份 `...-2` 计划 artifact；本次关键结论依据 gate 的实际下载日志与输出，不依据复制 job 对象的 attempt 标签。
- **结论 / 边界**：已发生的“仅重跑 gate 会误找当前 attempt 计划产物”缺陷在目标 SHA 的原生路径验证通过；K-CI-IMPACT-01 父债仍 **In progress**，Stage 3 选择性范围、Compare 权威性和其它失败形态不因这一子问题自动完成。本事件是新的文档提交，其目标 SHA CI 须与 `2a9c02d0` 的实质修复 CI 分列，不借旧绿。

### DCL-20261001-04 · K-SUPPLY-10 rust-toolchain 同步合同

- **记录者 / 类型**：主控 Codex 接续 Luna 草稿；base `c2dd6112c60f0aed6e0687b5d3654f863e8266f8`，开场 `main` 干净。重风险规划已裁定限定 Rust Action 同步语义；维护者明确自适应流程不限模型，主控完成收口与复核。K-SUPPLY-09 签名暂缓，不恢复 main、不升级依赖、不改生产 workflow 或 CI 编排。
- **实现 / 结果**：仅在 `kernel/crates/oclive-cli/src/ci_cmd.rs` 的 `#[cfg(test)]` 复用 `serde_yaml_ng` 与真实 `render_ci_yaml`，从 `CARGO_MANIFEST_DIR` 向上三层扫描全部 `.github/workflows/*.yml/*.yaml`；仓库 Rust Action 引用须为唯一 40 hex pin、每处显式 `stable`，Library/Kernel 两种真实模板须含同一 pin。共享 collector 的内存负控按预期拒绝 workflow-only PR #184 新 SHA、单处仓库 pin、模板漂移、缺 stable、短 SHA 与缺 Action。
- **本地证据**：定向 `ci_cmd::tests` native 0、5 passed；完整 `cargo test --locked -p oclive-cli -- --test-threads=1` native 0、16 套 132 passed；最终 `cargo fmt --all -- --check`、`cargo clippy --locked -p oclive-cli --all-targets -- -D warnings`、Dimension 5 `--ci` 29/29、默认及六份改文链接、docs-only stale paths、doc registry、doc mirror、debt-marathon 均 native 0。首次格式检查与 Clippy 对新增测试辅助代码报错，原日志保留，格式与类型简化后复测转绿。生产代码前缀、workflow、Cargo.lock 未改；完整 CLI 测试在临时项目构建时访问 crates.io 索引，未改仓库锁文件。K-SUPPLY-10 保持 **Partial**；本轮目标 SHA 远端 CI 与实际升级模板同步仍待证，不以既有 `36765997872` 绿灯宣称本轮完成。

### DCL-20261001-05 · K-SUPPLY-10 实际 Rust Action 更新配对模板

- **记录者 / 类型**：主控 Codex；接续 DCL-20261001-04 的防漂移合同与主线 `e3030a9f3d04726381e47c810bed5e39f883904f`。真实 [Dependabot PR #184](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/184) 仅升级四份 workflow 的 14 处 Rust Action pin；在当前主线应用相同更新，并同步 CLI 生成器生产模板的五处引用。旧 PR 仅作来源，不直接合并或改动其分支。K-SUPPLY-09 签名决策仍暂缓。
- **来源 / 语义**：所选原上游 `02cb101ec7c40f2c49e1d9714d64511d8e1b74de` 相对旧 pin 的 Action 差量是在两条 rustup 安装/默认命令增加 `--force-non-host`，不是完全等价提交；仓库所有 19 处仍显式 `toolchain: stable`，没有新增非宿主工具链输入。只替换 40 位 SHA，不改 runner、权限、触发、命令、失败策略、其它 Action 或锁文件。独立 nightly/release、非宿主工具链与传递供应链范围未据此验收。
- **验证 / 边界**：既有 CLI 生成器与仓库同步正负合同 5/5、fmt 原生 exit 0，独立只读差量复核通过。`check:ci-local` 首轮仅因把语音 Python 环境变量误指向 `.cmd` 包装器而 exit 1，原日志保留；仅改子进程为已核 `python.exe` 后第二轮 native exit 0（Dimension 5 29/29，前端构建及 Rust 工作区/CLI 集成均按原链通过）。四份改文链接、stale docs、登记、镜像、债结构、diff 检查全 exit 0；来源与细节见 [Actions Wave](waves/WAVE-20260929-ACTIONS-PINS.md#2026-10-01--pr-184-的实际-action-更新与-cli-模板同步)。目标正式 CI 待证。K-SUPPLY-10 保持 **Partial**，本次证明单个实际升级的人工配对，不宣称所有模板可自动维护。

### DCL-20261001-06 · K-SUPPLY-05 reqwest 范围与安全止点

- **记录者 / 范围**：主控 Codex；基线 `053ebdadf990c9c278bfc601c1e9b1640258c962`、开场干净。单族 D1 只读对账，结论与精确命令见 [Full 计划](long-plans/K-SUPPLY-05-Full.md#2026-10-01--reqwest-单族重对账d1实施未启动)。38 条 skip 仍须保留；tree 顶层未显示不代表 deny 的全目标依赖不存在，删除 17 条的临时负控已全部被拒，未动正式成员集。
- **事实 / 处置**：reqwest 0.12 是 CLI/Host/桌面的直接客户端；0.13 来自 Tauri 的移动目标，不是当前 Windows 运行图的第二份客户端。升级同时涉及 webpki→平台证书验证与 crypto provider 特性变化，不能按普通版本消重执行。仅修正 `deny.toml` 中 reqwest 的过期理由，未改变门禁、依赖或网络行为；维护者于 2026-10-02 选择暂缓此族并继续其他债务。机器计划/QUEUE 继续 blocked:needs-reconcile，父债 Full Partial 不变。没有升级编译或真实 HTTPS 验收。
- **前轮证据随本次实质文档入账**：K-SUPPLY-10 实际更新提交 `053ebdadf990c9c278bfc601c1e9b1640258c962` 的 [CI 36823868603](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36823868603) 为 completed/success，17/17 jobs，含正式 ci-gate；这是该 SHA 的已声明路径证据，不关闭所有模板未来维护债，也不替代本轮新提交的远端验证。
- **本地验证**：正式 `cargo deny --offline check bans`、默认入口和四份改文链接、docs-only stale paths、doc registry、债务结构、diff 与四份中文文档 UTF-8 检查通过。只改说明与接手记录，无需重跑产品场景；本轮未推送，不声称新提交远端已验证。

### DCL-20261002-01 · D-DEBT-LEDGER-01 六月轮次历史表迁移

- **记录者 / 范围**：主控 Codex；base `53479d8cbba6bffe92213dc65fde61fefac24882`，开场干净。依 [第二轮计划](ROUND-02-PLAN.md#d-debt-ledger-01--六月轮次-1619-历史表归档2026-10-02) 仅迁主台账 §5 的轮次 16–19，保留原位带历史边界的链接；新 [六月归档](../archive/TECHNICAL_DEBT_ROUNDS_202606.md) 不定义现行债务状态。
- **保真 / 结果**：迁前主台账原文 SHA256 `49DCAD75909783AD1BB749031CD3639100208AE514F12F2B293F59E3A8A2BA05` 留本机忽略证据根；四个轮次标题、原表格顺序与文字保持，九处相对债务锚点改为指向同一主台账目标。九处链接逆向重定位并还原归档末尾的一个段间空行后，表格正文逐字比较通过；当前登记结构检查仍过，历史-only 行 5→0、主表历史引用 9→0 是内容移出造成的结构变化，不是债务关闭或测试丢失。其它当前状态与旧归档不改，D-DEBT-LEDGER-01 保持 Partial；未处理全表自由文本冲突或其它长快照。

### DCL-20261002-02 · K-ENCODING-01 活跃文档自动防回退

- **记录者 / 类型**：主控 Codex；base `d7040c37cc95966764318a3f8aa6481c9a831b1e`、开场干净。按[本轮计划](ROUND-02-PLAN.md#k-encoding-01--活跃文档编码防回退2026-10-02)处理已发生过的中文静默损坏，事实与限制见[编码 Wave](waves/WAVE-20261002-K-ENCODING-01.md)。不改变旧事故原件或产品语义。
- **before → after**：此前只有人工写后红线；现在对三类活跃 Markdown 全量执行 fatal UTF-8、BOM、替换字符和连续三问号硬检查，并在 Dimension 5 中运行真实文件负控。开工前 307 份、本轮新增 Wave 后 308 份均通过；历史归档及非 Markdown 不在默认扫描内。主台账 K-ENCODING-01 保持 OPEN，单个/两个问号或语义损坏仍须人工对照可信来源。
- **本地证据 / 接续**：定向测试 6/6、默认检查、Dimension 5 30/30 及完整 `check:ci-local` 通过。首次 Dimension 5 唯一失败为本机缺 `py`，仅为随后子进程指定已安装 Python 3.12 可执行文件，未更改全局环境或语音门禁。目标 SHA 的远端 CI 仍须另验；不以父提交 CI 替代新提交证据。

### DCL-20261002-03 · D-DEBT-LEDGER-01 Event Stream 长历史出权威行

- **记录者 / 类型**：主控 Codex；base `5f2836ce67a8103882a383765d5a01913ccbb365`、开场干净。该 SHA 的[主 CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36971687198) 已 17/17 成功。按[本轮计划](ROUND-02-PLAN.md#d-debt-ledger-01--event-stream-长历史出权威行2026-10-02)只整理主台账入口；事实及保留条件见[本轮 Wave](waves/WAVE-20261002-DEBT-EVENT-STREAM-HISTORY.md)。
- **before → after**：`K-EVENT-STREAM-01` 原 4,340 B 状态长行[逐字归档](../archive/TECHNICAL_DEBT_EVENT_STREAM_STATUS_20261002.md)，主表保留 ID、问题、优先级、完成条件及 OPEN，只把状态格缩成当前能力、未交付边界和追溯链接。原 R1–R4 有限真实样本、R5/R6 合成证据及 B0 Trace-only 不再与 Production 缺口混读；没有改变 RFC 或启动 Stage C。
- **保真 / 出口**：原行 SHA256 `3C421D9F2DFCC11C7D68CFFE0BEE0741DCBAD4DC8B4E3A1D3B845B51BA575E9C`，归档与起点逐字相等；123 个独立 ID 行中只变 K-EVENT-STREAM-01 和治理父债行，前者前四格不变。默认/改文链接、旧路径、登记、债结构及活跃编码均通过；暂存差量与目标 SHA 远端结果另核。D-DEBT-LEDGER-01 保持 Partial，K-EVENT-STREAM-01 保持 OPEN，其它长历史和全表语义冲突不在本轮。

### DCL-20261002-04 · K-LLM-ENV-02 可控读取交错

- **记录者 / 类型**：主控 Codex；base `19293fe8c66dbbdccb6f9d823d22736e16bb67a6`，独立于上一文档切片。计划见[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--db-读取交错的隔离回归2026-10-02)，场景和边界见[本轮 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md)。不改生产设置应用、锁或用户凭据。
- **before → after**：原有验证只证明版本/dirty 两个原子结果；新增独立进程的内存设置集成测试，强制旧 DB 快照停在读取处，32 个新调用同时等待，验证旧调用放行前新调用不可越过读取、最终环境值属于新设置。没有把局部交错测试写成跨进程/真实 DB 压力或产品调用链全覆盖。
- **本地证据 / 剩余**：定向用例最终字节连续 5 次 exit 0，Host lib 634 passed，定向 Clippy、fmt、分层和综合 `check:ci-local` 均 exit 0。第一次 fmt 非零仅对应新测试三处格式，定点修正后通过。新目标远端 CI 待提交后独立核验；K-LLM-ENV-02 仍保留更长时间和 save/chat/theater/canonical sync 等剩余证据门，不升 Done。

### DCL-20261003-01 · K-LLM-ENV-02 AppState 刷新入口的内存 DB 回归

- **记录者 / 类型**：主控 Codex；基线 `b912237324fd9521968084cbf25f6c7dd267cc6e`，工作树干净。上一切片的[目标 CI](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37009520138) 17/17 success，Linux/Windows 原始日志均显示 `old_db_snapshot_cannot_overwrite_newer_concurrent_apply ... ok`。本切片依据[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--appstate-刷新入口的真实内存-db-回归2026-10-03)与[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--完整-appstate-的内存-sqlite-刷新边界)，不改生产实现。
- **before → after**：替身交错证据之外，新增完整 `AppState` + 真实内存 SQLite 的刷新边界测试。直接改 DB 但不标 dirty 时生产应用函数保留旧环境；调用刷新入口、或显式标脏后，环境才对应新 DB 值。这个负例定位了各调用者的标脏责任，不宣称它们已经全部履行。
- **验证 / 剩余**：定向测试 1 passed、Host lib 634 passed、定向 Clippy exit 0；初次 fmt 仅一处新测试换行，定点修正后复核通过。`npm run check:ci-local` exit 0，覆盖 Dimension 5、前端、Rust 格式/Clippy/lib/工作区集成与 CLI 集成。新提交远端 CI 待按目标 SHA 独立核验。真实持久库、完整 AppState 并发版本交错及 save/chat/theater/canonical sync 仍未覆盖；K-LLM-ENV-02 不升 Done。

### DCL-20261003-02 · K-LLM-ENV-02 保存设置入口的标脏/应用对照

- **记录者 / 类型**：主控 Codex；基线 `8e39f6b6269e0a7aba3d305c9353d1fa1e1936ed`。依据[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--保存设置主路径接入刷新2026-10-03)和[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--保存设置主路径的标脏责任)，限于本地 provider 的保存设置主路径。
- **before → after**：前一用例只证明直接 DB 写入后的 dirty 快路径；现加入最小合成角色，真实调用 `save_llm_user_settings_impl`。它保存第三个 URL 后，DB 和进程环境均对应新值，从而把保存入口的标脏责任与直接写库负例区别开。生产实现、模型/网络和用户数据均未改变。
- **验证 / 剩余**：定向集成测试 1 passed；fmt、定向 Clippy、`npm run check:ci-local` 均 exit 0，目标远端 CI 待新 SHA 单独核验。只覆盖 local provider；chat/theater/canonical sync、真实持久库、完整 AppState 并发版本、长时间压力和云端/LoRA 分支仍在剩余范围。父债不升 Done。

### DCL-20261003-03 · K-LLM-ENV-02 chat 主入口的待应用设置

- **记录者 / 类型**：主控 Codex；基线 `1a0acca1`。依据[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--对话主入口消费待应用配置2026-10-03)和[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--对话主入口应用待刷新-db-值)，复用已建立的内存 DB/合成角色/Mock LLM 夹具。
- **before → after**：保存设置入口已证明会标脏并应用，但 chat 自身只见源码调用；现把第四个 DB URL 标脏后走真实 `process_message`，模拟回复成功且环境变为该 URL。这样补上一个正常对话主路径的入口证据，不要求真实模型来证明环境刷新。
- **验证 / 剩余**：定向测试 1 passed；第一次 fmt 仅新增 import 顺序不符，修正后通过；定向 Clippy 和 `npm run check:ci-local` 均 exit 0。目标远端 CI 待新 SHA 单独核验。流式/断流、Theater/canonical sync、持久 DB、完整 AppState 并发版本和长期压力仍缺；K-LLM-ENV-02 不升 Done。

### DCL-20261003-04 · K-CI-IMPACT-01 全量 CI 节奏与责任边界登记

- **记录者 / 类型**：主控 Codex；维护者提出控制全量 CI 的使用频率，并把 Kernel/Host 责任分层作为后续 CI 设计债，而非本轮重构。归入既有 K-CI-IMPACT-01；已完成执行去重的 D-CI-EXECUTION-02 不重开。
- **触发事实 / 例子**：同一 K-LLM-ENV-02 主题的阶段提交 `8e39f6b6` 与最终 `9114cc01` 分别取得[主 CI 37047235967](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37047235967)和[主 CI 37053114491](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37053114491)的 17/17；两个 SHA 各自的成功证据有效，但这组切片也说明“每个小步骤都先结案再继续”会重复付出全量运行成本。不能仅由次数推定工作流设计有缺陷；开发/验收节奏与验证器归属分开处理。
- **本轮收敛 / 后续止点**：[`AI_VERIFICATION_PROTOCOL.md`](../AI_VERIFICATION_PROTOCOL.md#ci-推送节奏与证据绑定)明确同主题低/中风险切片先定向验证、批次最终 HEAD 再完整验收，真实高风险独立验收点例外；[`SOMEDAY_TOOLCHAIN_CI.md`](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#后续责任分层的调查输入未实施)登记 Kernel、通用 Host 接入、具体发行版与发布组合四层候选归属。现行 Push/ready 全量、草稿选择、正式 gate 均不改。下一步仅有界盘点现有 validator/job 的责任与 Compare 数据，足以分类即停；没有证据前不改路由、删门禁或把某个 Host 的全套测试升为 Kernel 公共契约。

### DCL-20261003-05 · K-LLM-ENV-02 Theater 入口的待应用设置

- **记录者 / 范围**：主控 Codex；承接本地 CI 批次节奏登记，基线 `5eec7b42e706d28d78eb4c63ae087020c562716c`。按[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--theater-主入口消费待应用配置2026-10-03)只扩现有独立 Host 集成测试及同债记录，生产代码和公共契约零改动。
- **before → after**：此前完整 `AppState` 的内存 SQLite 测试已覆盖刷新、保存与 chat，但 Theater 仅见 `generate_scene` 源码调用。现在第五个 URL 写库并标脏后，调用有效 Theater 场景；模拟 LLM 在实际标签生成时记录新 URL，返回结构化台词。生成结果、调用次数与调用时环境均被断言，不能仅凭返回后的环境推断生成前配置。
- **验证 / 剩余**：最终字节定向测试 1 passed；首次 fmt 因新测试缩进非零，定点修正后 fmt、定向 Clippy 与分层均 exit 0。与 canonical seed 切片合批运行的 `check:ci-local` 最终 exit 0，目标 SHA 的远端 CI 仍待核，不借前一个 SHA 的绿色结果；父债保持未结案。完整 AppState 并发版本、canonical sync 其它路径、持久 DB、流式/断流、真实 provider 和长时压力仍是独立缺口，见[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--theater-主入口消费待刷新-db-值)。

### DCL-20261003-06 · K-LLM-ENV-02 桌面 canonical seed 的文件库对照

- **记录者 / 范围**：主控 Codex；接续同债 Theater 测试，仅新增桌面集成测试与债务记录。按[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--桌面-canonical-seed-的隔离文件库回归2026-10-03)核生产 `seed_shell_llm_from_canonical`，没有改生产同步、DB schema、公共 DTO 或真实用户数据。
- **before → after**：此前 canonical seed 只见源码中的复制、标脏与应用调用。现在以临时 canonical `app.db` 的最小 `app_settings` 表为输入，先让内存 shell 应用旧 URL，再调用生产 seed；shell DB 和进程环境均变为 canonical 新 URL。测试进程保存并恢复环境、显式关闭文件库 pool。
- **验证 / 边界**：定向桌面测试 1 passed，fmt、定向 Clippy 与分层 exit 0。首轮 `check:ci-local` 只因新测试夹具触发旧目录名检查而止步，定点改名后第二轮完整本地链 exit 0；目标 SHA 远端 CI 另核，父债不升 Done。此证据只覆盖 local-provider seed，不证明完整桌面启动、全部迁移、双向同步、云端 token、完整 AppState 并发版本或长期压力，详见[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--桌面-canonical-seed-的隔离文件库回归)。

### DCL-20261003-07 · K-LLM-ENV-02 流式生成调用时的待应用设置

- **记录者 / 范围**：主控 Codex；基线 `1e9cd8e0`。按[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--流式对话调用时配置观察2026-10-03)复用既有隔离 AppState 夹具，只改测试与债务记录，不改生产流式或设置算法。
- **before → after**：此前 chat 普通回合与 Theater 已有调用入口证据，但流式生成仍只见源码共用入口。现在第六个 URL 入内存 DB 并标脏后，生产 `process_message_stream` 调用模拟流式生成器；生成器在被调用时记录新 URL、发 token，回合返回独立正文。测试把环境更新与真实流式调用绑定。
- **验证 / 剩余**：定向测试 1 passed，fmt、定向 Clippy exit 0；与文件库重建切片合批的 `check:ci-local` 原生 exit 0，目标 SHA 远端 CI 待核。该证据不覆盖 HTTP SSE/断流/fallback、完整 AppState 并发版本、真实 provider 或长时压力，父债仍未结案，见[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--流式对话生成调用前应用待刷新设置)。

### DCL-20261003-08 · K-LLM-ENV-02 文件库重建后的设置应用

- **记录者 / 范围**：主控 Codex；接续同债流式切片，仅新增隔离 Host 集成测试和债务记录。按[第二轮计划](ROUND-02-PLAN.md#k-llm-env-02--文件库重建后的本地设置刷新2026-10-03)使用生产 AppStateBuilder、临时文件库和模拟 LLM，不改生产构造或迁移实现。
- **before → after**：此前完整 Host AppState 的刷新证据来自内存 SQLite，桌面 canonical seed 只有最小手建表。现在让生产构造建立并迁移临时 `app.db`，第一份状态写入 local 设置并关闭 pool；第二份状态由同一路径重建，读回 URL 且生产 reload 将其应用到进程环境。
- **验证 / 边界**：定向测试 1 passed，fmt、定向 Clippy exit 0；与流式切片合批的 `check:ci-local` 原生 exit 0，目标 SHA 远端另核。该测试不证明崩溃、跨进程、全部迁移、并发版本竞争、真实 provider 或长时压力，父债不升 Done，详见[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--迁移后文件库的-appstate-重建)。

### DCL-20261003-09 · 停止扩展补证，回到具体债务处理

- **来源 / 原因**：维护者指出当前补证已过细，目标是偿债而非持续研究证明。此前 K-LLM-ENV-02 各切片列出的未覆盖场景，被当成继续开工的理由，造成验收范围不断扩大。
- **调整**：按[实施停止规则](../AI_VERIFICATION_PROTOCOL.md#210-技术债实施足以修复与验收即停止补证)停止该债自动扩展补证，接续只办理原问题的有限验收；未覆盖范围不自动入债、不自动阻塞。保留已有测试与历史结论，未取得的目标 SHA CI 仍待核，不以流程纠偏冒充新的测试结果或直接升 Done。
- **接续**：下一项优先处理原因与改法明确的实现债；具体决策/外部阻塞跳过后统一报告。维护者进一步明确：验证收益下降即停，是否深入由维护者确认；因此不自行续预算或换名继续补证。当前为四份文档的规则与安排校准，仅做适用文档/债结构检查，不跑全量工程链。

### DCL-20261003-10 · D-DEBT-LEDGER-01 收敛 K-LLM-ENV-02 长状态行

- **起点 / 改动**：base `7b0b41a145847bfefcedd642f29b1e16d3ad52cc`，工作树干净；按[第二轮计划](ROUND-02-PLAN.md#d-debt-ledger-01--k-llm-env-02-长状态行收敛2026-10-03)，将 K-LLM-ENV-02 的完整原状态行[转为历史快照](../archive/TECHNICAL_DEBT_LLM_ENV_STATUS_20261003.md)，权威行只保留当前修复状态、有限证据与待验收事项。
- **不变量 / 边界**：原问题、优先级、完成条件、历史文字和链接均保留；K-LLM-ENV-02 与 D-DEBT-LEDGER-01 仍为 Partial。双核 NULL 回滚是冻结决策门，发行版默认身份也是语义选择，本轮均跳过。纯文档迁移只做原行保全及适用文档检查，不重跑业务或全量 CI。

### DCL-20261003-11 · K-ENCODING-01 跟踪角色 JSON 防回退

- **起点 / 原因**：base `1b3680b9d4084357deeee0ceba0b40100503f05e`，工作树干净；活跃 Markdown 已有自动字节检查，官方角色 JSON 仍依赖人工写后检查。按[本轮计划](ROUND-02-PLAN.md#k-encoding-01--官方角色-json-编码防回退2026-10-03)将 Git 跟踪角色 JSON 加入同一只读检查，未跟踪的本机聊天记录明确排除。
- **before → after**：默认检查从当前 310 份活跃 Markdown 扩为 310 份 Markdown + 123 份跟踪角色 JSON；复用无效 UTF-8、BOM、U+FFFD 与连续三问号判据，不读取运行期 JSON，也不改角色数据。隔离测试证明入选/排除与坏 JSON 拒绝。
- **验证 / 限制**：测试 8/8、默认扫描 433 份及 Dimension 5 `--ci` 30/30 通过。Dimension 5 首轮仅因本机语音检查缺直接可执行 Python 而失败；第二轮命令局部指定已安装 Python 后通过，未改全局环境。其它 JSON、单/双问号、语义损坏与历史恢复不在本切片；K-ENCODING-01 维持 OPEN，目标 SHA 远端 CI 待核，详见[编码 Wave](waves/WAVE-20261002-K-ENCODING-01.md#2026-10-03--官方角色-json-的受控扩面)。

### DCL-20261003-12 · D-CLI-BLUEPRINT-05 显式最小角色文件校验

- **起点 / 原因**：基线 `86cbb2d5a6651da8a0caba594933f8c8a67850b5`，工作树干净；维护者选择最小角色包接入为下一主线。现有共享加载器能准备定义与本地资产，但 CLI 只能按参考宿主蓝图/legacy 格式校验目录。
- **本片**：按[第二轮计划](ROUND-02-PLAN.md#d-cli-blueprint-05--显式最小角色本地校验入口2026-10-03)，CLI 新增 `pack validate-minimal-local`，直接复用同一个只读加载器；调用方选择资产根与定义文件，预算有界。无蓝图的一图加 prompt 正向通过，缺资产与定义路径越界拒绝；旧 `pack validate` 行为不变。定向集成测试 2/2、Clippy、分层、债结构、中英镜像、链接、登记、旧路径及编码检查均 exit 0；fmt 首次只发现新测试一处换行，定点修正后 exit 0。本片仅形成本地提交，后续同主线里程碑再跑完整本地与目标 SHA CI。
- **边界**：这只把既有准备能力暴露给 CLI；没有统一磁盘包名、生成器、媒体解码或 Host 生命周期。公开 `PromptInput` 与 `Arc<Role>` 缓存的迁移是下一决策门，不用产品默认值冒充完成。主台账保持 Partial。

### DCL-20261003-13 · D-CLI-BLUEPRINT-05 最小快照进入 Prompt Base

- **起点 / 原因**：接续本地提交 `e2ad3066`。维护者选择保留旧接口增量接入；CLI 能校验最小本地快照，但参考 Host 的 Prompt 入口与缓存仍依赖完整 `Role`。
- **本片**：新增借用 `LocalMinimalRoleSnapshot` 的 `LocalMinimalRolePrompt`，经现有 `PromptBase` 能力调用。人设来自已验证快照，材料保持顺序；复用现有文本连接核心，非空附加要求返回 `Unsupported`。一图加 prompt 的外部 crate 测试通过；不构造产品默认关系、人格向量、角色名或蓝图。
- **边界 / 后续**：这是单个能力实现的接入，未注册到参考 Host 的 `AppState`、角色加载或回合流程，也未解码视觉资产。旧 Prompt 接口不变，D-CLI-BLUEPRINT-05 仍 Partial；下一片须明确 Host 如何拥有快照、绑定技术身份并治理资源，不能把本片写成角色已激活。
