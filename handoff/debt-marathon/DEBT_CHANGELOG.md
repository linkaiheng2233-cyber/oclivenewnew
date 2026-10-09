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
| D-CLI-BLUEPRINT-05 | **已裁定逻辑契约 / 宿主接入缺口**：保留参考 Host 旧接口、增量接入最小逻辑定义；跨发行版不要求统一磁盘封装 | [ROLE_PACK_BOUNDARY §0](../ROLE_PACK_BOUNDARY.md)、主台账 §1；共享定义、本地快照、CLI 显式文件校验、Prompt Base 与双来源独立 Host 案例已存在。关系归发行版；`content.json` 只是本地适配示例 | 继续核参考 Host 内部增量接入点与真实发行版映射；不切换 CLI 默认生成目标，不用产品默认值填充旧 `Role`，不把示例误记为参考 Host 激活 |
| K-CORE-BOUNDARY-01、V-EMBED-01、V-PORTABLE-01 | **互补验收面**：逻辑 Kernel、完整参考运行时嵌入、跨发行版映射；物理拆分与 Full 实机条件分别是**决策门 / 外部条件** | [MODULE_MAP](../MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities)、主台账；已验 B1/B2、进程内门面及 [ChatPro 案例](../CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md) 只在所列范围复用 | 按选定 Kernel 延伸合同核 Host；不把跨发行版 UI Full 或物理小 core 交付作为工具链、CLI 或嵌入研究的统一前置；2026-10-09 有限公共兼容审阅规则已获维护者采纳，其它验收面保持原范围，不因规则已有自动完成 |
| K-UID-DEFAULT-02 | **已裁定并收口**：有效显式选择优先，发行版默认 A 高于角色默认 B，A 不存在则回退合法 B | 主台账 §1、[DCL-45](#dcl-20261006-45--发行版默认身份兑现用户确认的优先级)；`58be11fa` 精确正式 CI 已通过，loader／global／per-scene／恢复默认与消费者已同步 | 默认选择合同 Done；不因字段名或此合同去迁移全部 legacy 历史关系 |
| K-AGENT-MERGE-01、V-FUSED-01 | **独立范围 / 决策门**：工具 composite 与多槽实例融合不能混为同一实现 | 主台账前瞻风险及 [融合 stub](long-plans/V-FUSED-01.md)；前者须定工具冲突、权限、顺序/短路、超时/隔离与 trace，后者仍受其 Phase 3 条件约束 | 核单 Agent 行为与诊断声明；不从多 ID 推断已合并，也不以此阻断单 Agent/MCP 使用 |
| K-DUAL-ROLLBACK-02、DUAL-CORE-FREEZE | **维护冻结 / 未解决**：2026-10-07 维护者选择暂停 Beta 继续维护；解冻后才选择 NULL 恢复或收窄实验写入 | 主台账 §1/§2及 [冻结范围与解冻条件](long-plans/DUAL-CORE-FREEZE.md)；补偿不是数据库事务 | 当前仅保留事实与接手方向，继续其他可开工债务；不自动启用、扩证或修补 Beta，也不要求 Stable 等待其全部修复 |
| D-HOST-RECOVERY-01、D-HOST-ERROR-CONTEXT-01 | **独立残留窗口 / 证据门**：当前源码、确切可达路径和新故障范围；旧历史推断不是当前复现 | 主台账 §4、[Host 检查单](../CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md)；同 ID 恢复与取消已有所列限定证据，提交/收据原子性等另定 | 复核残留路径并保留历史；不重跑旧业务身份，不把所有崩溃窗口设为已收口接入案例的再验收前置 |

### 事件、资源与开发工具

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-EVENT-STREAM-01 | Stage A 的 Session/wire/checkpoint、存储/隐私/ACK及未定恢复策略是 Stage C consumer runtime 的**硬前置 / 决策门** | [RFC_RUNTIME_EVENT_STREAM](../../creator-docs/rfc/RFC_RUNTIME_EVENT_STREAM.md)；Trace/DTO/探针不是 Production，A.2.2.2 的宿主密钥恢复、多宿主协调尚不能当已冻结 | 在获准 A 切片中取证与固化合同；不启生产读取/消费循环、不真实发送或撤回外部消息；2026-10-09 选择 2A，生产化继续暂缓，既有原型保留 |
| K-PROACTIVE-01 | **共用能力**：现有 Event Ring、可信 origin 与一次性 permit；不要求 Production Stream 整债 Done 才能核现有非用户回合 | 主台账前瞻风险、[Event Ring](../../creator-docs/plugin-and-architecture/EVENT_RING.md)；生产持久消费属于上行独立范围 | 核现有主动链与拒绝/零副作用合同；不建立第二套回合入口或把 Trace 接到行为反馈 |
| K-RESOURCE-COORD-01、D-SCAFFOLD-RESOURCE-01 | **契约能力硬前置 / 协同**：CLI 复用已实现 types/Host Plan Compiler；目录自动装配、真实渲染和 soak 是资源父债的独立剩余范围 | [资源 RFC](../../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)、主台账；工具界面需要具体可用字段/原因码，不要求资源父债所有硬件条件先 Done | 核只读配置/诊断、round-trip 与无 GPU/observe-only 负例；不发明新 schema、设备探测或自动跨适配器授权 |
| D-SCAFFOLD-EVOLUTION-04 | **决策门**：使用问题、一个有界目标、迁移及来源/权限/沙箱 | [脚手架 RFC](../../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)、主台账；Stage 2C 仍需单独取舍，资源工具并不自动解冻组合/namespace/离线生命周期 | 收集实际使用问题；不把依赖解析、市场/联网安装和第三方 CI 编排权并进首批；2026-10-09 选择 4A，继续观察，Stage 2C 不启动 |
| K-CI-IMPACT-01、D-CI-AI-REVIEW-03 | **数据硬前置**：AI 评测复用 Shadow/Compare 的版本化真实数据；不要求 selective 全部开放才开始采集 | 主台账前瞻风险；漏选/过选与全量结果对应同事件，训练/评测隔离。AI 不拥有降级确定性门禁、Runner 或 Secret 的权力 | 核已有数据/目录边界；无冻结评测与运行预算不训练，模拟语料不代替真实 CI 漏选证据 |
| K-LLM-ENV-02 | **原缺陷已有限结案 / 新反例另立范围**：旧 DB 读取覆盖新环境并误清 dirty 的可控交错与目标 SHA CI 已验收；真实 provider、跨进程与长期压力不随之转绿 | 主台账 §1、[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md)；`86cbb2d5` 的 CI 17/17，不把原先 Partial 阶段倒写成完成 | 无新的可复现反例不继续扩张矩阵；若出现独立故障，另定隔离与预算，不碰用户凭据或真实 provider 配置 |

### 模型质量、语音与生态产品

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-EMO-01、03 | **协同**：词表与显式降级路径；按实际语言/重复实现边界区分 | 主台账 §1；英文扩充先核中文词表条件，降级出口收拢须保留现有主 LLM 与安全回退语义 | 固定文本回归与调用点盘点；词表完善不等于复杂语义或角色质量已通过 |
| K-EMO-05、07 | 趋势需 M2 历史数据，复杂情绪优化需固定角色/场景评测：分别是**数据硬前置 / 证据门** | 主台账 §1；05 不因 07 研究自动获准；07 仅 shadow/advisor，对照通过后才可有界进入 Prompt | 准备可审计评测输入和数据字段；真实 7B 运行须预算，不让规则/CHS/VAD 成为第二个决策内核。历史 S01 FAIL 不因新文档变绿 |
| K-EMO-06 | **已确认 Deferred / 可选扩展**：2026-10-09 维护者选择 3A；高级情绪驱动长期记忆不列入核心开发目标 | [决定、边界与重评条件](../../creator-docs/architecture/DESIGN_DECISIONS.md#emotion-memory-extension-deferred)；须有具体可复现接入案例证明现有契约阻碍明确需求，才评估公共调整；评估不自动授权实施 | 记录第三方模块/发行版可选探索；继续修基础能力的真实缺陷，不新增高级策略或 Base 强制接口，不以研究价值自行解冻 |
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
- **并行分支合流先对事件身份**：同一个编号若指向不同事件，不能直接并成同一条或丢掉一方。保留两个原提交和原记录；组合文档为新导入事件分配当前唯一编号，登记原编号、来源完整 SHA 与改号理由。正文事实和证据范围保留；旧冻结分支不回改，旧 CI 不改绑组合 SHA。冲突只处理相应登记，不重做无关业务验证或停住其它无重叠工作。
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

### DCL-20261003-14 · D-CLI-BLUEPRINT-05 独立最小 Host 装配

- **起点 / 原因**：基线 `c51f8415f702fa8b90da666827cd125cddb41220`。维护者选择先从现有 Base 能力做独立最小 Host 案例，旧参考 Host 接口与主链继续稳定；此前只有 CLI 快照准备与单个 Prompt 能力调用，没有 Host 对身份、快照所有权和失败止点的组合。
- **本片**：新增可运行的 `minimal_role_host` 示例：Host 自定技术 ID、持有有界本地快照、调用现有 Prompt Base 和内存 Echo LLM。正常、错身份、额外要求拒绝及缺资产的 3 项示例测试通过；原生示例运行 exit 0。错身份与 Prompt 拒绝均未到达 LLM。
- **边界 / 后续**：例子无参考 `AppState`、DB、网络、真实模型、视觉渲染与跨平台发行版结论；即时 future 驱动仅服务本例。独立 Host 的最小装配案例已形成，但统一磁盘载体/生成、产品 Host 生命周期与真实分发仍缺，父债保持 Partial。

### DCL-20261003-15 · K-LLM-ENV-02 原并发缺陷有限结案

- **原因 / 依据**：此前刻意停止追加低收益的环境排列组合，只等待既定目标 SHA 的正式 CI。`86cbb2d5a6651da8a0caba594933f8c8a67850b5` 的 `ci.yml` [run 37100060646](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37100060646) 原生 `success`，17/17 作业成功；同目标本地综合链 exit 0。
- **裁定**：原先“旧 DB 读取覆盖新环境并误清 dirty”的事务修复、可控交错和有限 save/chat/Theater/canonical 主路径完成条件已满足，主台账 K-LLM-ENV-02 转 `Done · bounded original defect`。原因、测试与界限见[同债 Wave 收口](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--原缺陷的有限收口)。
- **不外推**：真实 provider、云端 token、跨进程环境、所有崩溃/失败交错和压力未被本结案证明；出现具体反例再独立登记。此前各切片的 Partial 是当时状态，原文保留，不倒填成“当时已结案”。

### DCL-20261003-16 · D-CLI-BLUEPRINT-05 Prompt 能力输入去文件适配耦合

- **原因**：上一增量片的 `LocalMinimalRolePrompt` 借用了 `LocalMinimalRoleSnapshot`。虽然能力调用本身无 I/O，但类型上要求所有发行版先走本地文件适配，削弱最小逻辑定义的跨 Host 用途。
- **修正**：同一 Prompt Base 实现改为 `MinimalRolePrompt`，只借用 `MinimalRoleDefinition` 并复用共享逻辑校验；独立 Host 仍持有本地快照、给能力传定义。外部 crate 增加纯内存定义调用，原本的一图文件案例、三项 Host 示例测试及原生运行继续通过，非空附加要求仍拒绝。
- **边界**：构造成功只证明逻辑字段合格，不能证明资产存在或可显示；本地 Host 的加载器另证明受控字节快照。没有迁移旧 `PromptInput`、参考 Host 生命周期或产品回合，父债仍 Partial。

### DCL-20261004-17 · 依赖图对齐最小角色与环境并发的现行裁定

- **原因**：本文件顶部调度表仍将 D-CLI-BLUEPRINT-05 的旧接口取舍写成未决，并将 K-LLM-ENV-02 原缺陷写成仍等待压力证明；两者都落后于已登记的实现与验收，会让接手者重复开启错误前置。
- **修正**：只更新这两行的**当前调度口径**。最小角色继续保留旧接口并走独立 Host 增量路径，统一磁盘格式仍是后续取舍；环境并发原缺陷按 `86cbb2d5` 的限定证据结案，真实 provider/跨进程/压力不随之验收。
- **边界**：历史事件与原始证据不改，技术债主体状态不升格；本次不修改产品源码、角色包格式或 CLI 默认行为。

### DCL-20261004-18 · D-CLI-BLUEPRINT-05 双来源 Host 与磁盘边界

- **原因 / 取舍**：仅有本地快照的 Host 示例仍容易让接手者以为 `content.json` 是跨发行版标准。维护者认可保留最小逻辑契约、发行版自行适配磁盘来源；最终二次审核前，按该方向实施可逆的独立示例和文档，不改旧参考 Host API。
- **本片**：示例 Host 内部内容来源接口同时接收本地快照与纯内存定义/资产，同一技术 ID 选择、Prompt Base 与内存 LLM 路径；Host 在能力调用前核逻辑字段和资产数、顺序、非空字节。角色边界与主台账不再把统一磁盘封装/生成器写成 Kernel 合规前置。
- **验收 / 边界**：五项示例测试、原生示例运行与定向 Clippy/fmt 通过后形成小提交；纯内存来源仍是合成夹具，不代表第二发行版已实接。不解码视觉资产、不建参考 `AppState`、不触碰 CLI 默认格式。`D-CLI-BLUEPRINT-05` 仍 Partial；格式取舍会在本轮汇报中单列供维护者二次审核。

### DCL-20261004-19 · 参考 Host 能力拒绝前的角色状态发布

- **原因 / 依赖**：从 `10e4d4c1` 的最小角色装配继续核参考 Host 接入点时，发现旧 `load_role_impl` 在 required extension 拒绝前已写当前选角与插件状态。新增有限实现债 D-HOST-ROLE-ACTIVATION-01；复用现有 RoleStorage、ExecutionPlan 和插件设置能力，不依赖最小角色全接入，也不恢复已冻结的恢复/错误上下文事项。
- **原生反例 / 修复**：合成包及内存 SQLite 的旧实现回归 exit 101，当前标记从 `previous` 变成 `blocked`。把校验移到发布前；因解析还要看未迁移的 legacy 禁用列表，抽取同一初始状态选择并做内部只读预览，沿用原 Provider 权限/可用性计算，保留已有角色记录及 global 合并规则。没有新公共 DTO、端口、权限或错误码；HTTP/Tauri 共用角色服务无需改线。
- **定向验证 / 后续**：四项 execution-plan 回归通过，包含拒绝保留磁盘字节/内存选角/DB/缓存、正常激活以及 legacy/global/已有记录优先级；Host lib 635 项、all-targets/all-features Clippy、fmt、分层、module-compat 及适用文档/债务门禁均 exit 0。红/绿日志分存在 `.cursor/plans/debt-role-activation-20261004-r0/`。首轮 fmt 仅发现本片换行，规范化后通过。主控自查不冒充独立审查；同主题冻结点才跑全量并推送，当前父债不升 Done。
- **调查止点 / 未决**：能力拒绝这一明确顺序错误处理后停止扩证；不追所有后续 DB/并发/崩溃窗口。参考 Host 若直接运行纯最小角色，缺失关系、人格等产品功能如何表现是另一产品范围选择；已向维护者提出澄清，等待期间只推进本片，不用产品默认值填旧 `Role` 或自行创建第二条参考回合流水线。

### DCL-20261004-20 · 最小角色基础互通与开发者转换器

- **维护者裁定 / 原因**：对上一事件的产品范围问题，维护者明确开发者为扩展格式准备转换器；最小角色须能在发行版跑基础闭环，扩展缺失明确不可用。核现有角色边界、角色规范及 HostProfile，已有“自有格式分别映射内容与能力”的分工与此一致；缺的是基础运行路径的实现，不能继续把格式取舍当 blocker。
- **本片**：在角色边界 §0.1 明确转换器、基础能力和扩展不可用的责任；纠正图中将生命周期归给小 Kernel 的用词。角色规范中英修正“CLI 未接”的过期说法，并只摘要链接边界。第二轮计划把下一施工点限于一个参考发行版的基础路径与旧丰富路径兼容；不新增公共转换器协议或假设所有发行版已验证。
- **债务 / 止点**：D-CLI-BLUEPRINT-05 仍 Partial，后续只沿现有入口处理内容/基础结果/产品扩展的真实耦合，不继续扩大示例和数据证明。缺模型/资源不能伪装成功；调用方明确要求的未支持扩展仍须拒绝，不靠丢弃要求通过。不重跑业务用例或全量 CI来验证本次文档裁定。

### DCL-20261004-21 · 参考 Rust Host 的最小角色基础文本接入

- **原因 / 实际变化**：数据面和独立示例已经足够，继续补示例不能偿还“参考 Host 仍要求完整 Role”的实现债。本片在原 OcliveKernel 增加 `process_minimal_message`，由既定 `process_message.rs` 编排共享 Prompt 与 Host 已装配模型客户端；开发者转换后构造不可变 `PreparedMinimalRole`，也可适配已读取的本地快照。技术 ID、资产快照由 Host 持有，不创建旧 Role、蓝图或产品默认状态。
- **基础结果 / 兼容**：增量 DTO 只有角色技术 ID、模型原样 reply 和 `product_extensions: unavailable`；仅表示本路径不执行产品扩展。非空额外要求在模型前经 typed Unsupported 停止；空消息、模型和设置错误保留对应原错误，无本编排 fallback/重试。正常空文本原样保留。旧丰富角色服务、RoleCache、HTTP/Tauri、收据和 SendMessageResponse 无修改；额外要求不被新请求反序列化静默吞掉。公开 facade 薄转发，错误放在既有共用 message_error，不从领域实现反向依赖 facade。
- **本地证据 / 复核**：新增外部 crate 五项回归和既有 rich facade 一项回归通过；Host/types all-targets/all-features Clippy、workspace doctest、分层与 module-compat 均 exit 0，文档镜像与债务结构检查通过。控制方按实际 diff 做语义自查，不冒充独立审查。公共 API 故使用 doctest；本主题合并先前能力拒绝修复与方向文档，在冻结提交上只跑一次完整本地链后推送。前一个 `10e4d4c1` 的正式 run `37139976177` 已核为 success、17/17，但不替代本片目标 SHA 的 CI。
- **自行修正 / 保留日志**：首轮编译把已有 LlmGenerateOutcome 的字段写错，随后测试实现漏了 generate_tag 并误用了 SQLx shim 未导出的 Connection；均按真实契约修正，没有改生产契约迁就测试。首轮运行四项通过、一项仅在 TempDir 清理时报 Windows 文件占用；测试加入最多两秒的独占打开释放检查，清理仍须成功。Clippy 又检出测试显式 drop 的锁仍跨 await，被词法作用域修正。所有失败/成功日志分存在 `.cursor/plans/debt-minimal-role-host-20261004-r0/`。删除旧失败夹具 `.tmpQdYRu3` 的安全检查与清理命令被工具自动审批整体拒绝（blocked by policy），没有重试或绕过，目录留待后续授权清理；不据此宣称旧失败清理成功。
- **范围 / 状态**：本片是参考 Rust Host 的单次文本入口，不是 ChatPro UI、传输、渲染、真实模型/语音、多轮记忆或持久化恢复验收；也不关闭原债全部生命周期和跨发行版适配。D-CLI-BLUEPRINT-05 仍 Partial，完成基础主路径后停止扩证。K-SUPPLY-09 签名、双核及 Event Stream 等冻结项不变。
- **批次门禁返修**：`87c4debf` 的第一次完整本地链在 Dimension 5 路径门禁 exit 1：测试的临时根写成 `join("roles")`，触发现有旧路径禁用规则；这是夹具命名问题，不是真实角色目录选择错误。改为明确 `fixture-role-root`，同步未建丰富角色目录的断言，不增加规则白名单；全路径检查与五项新/一项旧门面回归均通过。原失败日志 `08-ci-local.log(.err)` 保留，新增 `09-fixture-path.log(.err)`；规范化本次长断言的纯格式换行后，在新冻结点重跑完整链，而非无代码 rerun。

### DCL-20261004-22 · 生成 library 直接消费最小角色入口

- **原因 / 实际偿还**：上一批已实现基础 Host，生成库却仍只在根部提供丰富角色类型与用法；开发者需要深入模块路径猜接入方式。已链接 library 的模板现在直接重导出五个既有最小角色类型，生成 README 与 rustdoc 分列最小基础和旧丰富入口。没有新增包装调用、DTO、转换器协议、角色默认状态或运行依赖；转换器和资产策略仍由开发者负责，默认 init 格式不切换。
- **实现 / 兼容**：`lib.rs.hbs` 和 `README.generated.hbs` 为生产差量；现有 library 模板合同测试新增类型与用法断言。新的 CLI E2E 真实生成独立 library，测试消费方只通过该库的公开路径装配 Host + 内存模型，核一次原样 reply、显式 unavailable 及 typed Unsupported 的零新增生成；旧丰富入口同库编译，原模板七项回归通过。隔离项目沿用 workspace lock 的已解析版本，不依赖本机碰巧存在的最新缓存。原尝试将 optional Host 同时作为 dev-dependency，后被默认依赖树回归拒绝并撤回；最终只通过已有 `diagnostics-host` 显式启动消费测，不新增依赖、不改根 lock。未链接的 serde stub、kernel-server、Host 主编排、官方包、HTTP/Tauri/UI 无修改。
- **已取得证据 / 自查**：新增生成合同先红 exit 101（旧模板缺 PreparedMinimalRole），改后通过；CLI 单元 85 项、模板集成 7 项、生成消费一项与生成 rustdoc 一项、CLI all-targets/all-features Clippy、workspace doctest 及适用文档/债务检查通过。使用生产 builder 和隔离文件 SQLite，但模型是内存替身，不外推真实模型、媒体或产品发行验收。主控按最终 diff 自查，不声称独立复核。冻结批次只跑一次完整本地链，目标远端结论另核。
- **自行修正 / 原始失败**：首次生成消费的子 Cargo 已 exit 0，外层却把 Cargo 的 doctest 标题误当 stdout 而失败；修为检查真实 stderr，同时保存两段原始输出，并要求消费与 rustdoc 各一项成功，防止零 doctest 冒充编译证据。新增文件中重复的无角色目录断言触发旧路径 ratchet，移除该重复断言，原 library-embed 模板回归仍验证不生成角色包；没有加入白名单或改路径规则。失败与成功日志分别保存在 `.cursor/plans/debt-minimal-role-library-20261004-r0/`；消费子进程退出后显式关闭临时项目，未遗留用户态资源。
- **状态 / 止点**：D-CLI-BLUEPRINT-05 继续 Partial，本片只偿还已链接 library 的可调用性和接手用法缺口，不把生成库当作第二个实际发行版、丰富生命周期或 ChatPro UI 接入。父批 `96ca9d29` 的 CI 不替代本片新目标；没有恢复 K-SUPPLY-09、双核或 Event Stream 冻结工作。完成生成消费主路径后停止扩证，后续只选实际使用断点或另一个已具备实施条件的债务。
- **冻结后门禁返修**：`b629931d` 的 `10-ci-local` 先被缺 Windows py 启动器拒绝；按已有脚本配置，仅在命令进程指定已安装 Python 3.12 原生解释器。首轮用 .NET 值比较复核环境恢复，结果为 false；改按 Env provider 的存在态显式恢复，`12-python-restoration` 确认原缺失态恢复 true。`13-ci-local-configured` 的 Dimension 5、前端、workspace Clippy/库/集成测试通过，随后 CLI 单元 84/85，默认依赖树用例拒绝 Host dev-dependency 引入 SQLite；该提交不标绿、不推送。撤回 dev-dependency、保留旧回归不变，并为特性消费测在原 CI cli job 增加独立命令；这是必要测试接线，不是 CI 分层重构。新冻结点重跑受影响窄测与完整链，失败日志原样保留。返修后 `14-cli-default-restored` 85+7 项、`15-opt-in-consumer` 消费+rustdoc 各一项通过；结构化 YAML 对比确认除新增步骤外全工作流相等，原 CI execution-policy 自检通过。默认测试不执行该特性用例，不把零测试算作消费通过。

### DCL-20261004-23 · 必需能力拒绝前发布修复的有限结案

- **证据 / 状态变动**：目标 `96ca9d29832c9560e1427843db76154dbb63d915` 的正式 [CI 37144457163](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37144457163) conclusion=success，17/17（含 Windows/Linux Rust 与 ci-gate）；同冻结点的完整本地链和四项执行计划回归已通过。D-HOST-ROLE-ACTIVATION-01 从 Partial · Locally verified 转为 Done，随这次生成库实质返修提交入账，不为单独写绿灯另造证据提交。
- **有限范围**：只关闭 required capability 拒绝前污染选角/插件 UI 状态的原缺陷，保持已有/全局/未迁移 legacy 禁用设置语义；不包含 DB 发布失败、并发切换或崩溃原子性，也不把该修复当最小角色接入证据。D-CLI-BLUEPRINT-05 仍 Partial，本轮新 SHA 仍须独立核 CI。签名、双核、Event Stream 等暂缓项不变。

### DCL-20261004-24 · 最小角色状态行与 CLI 用法去除过期表述

- **原因 / 已完成**：最小角色的台账状态格逐轮累加，早期“Host 未接”与近期基础入口并列；CLI `create` 指南也未区分生成器和显式文件准备，容易让接手者重新领取已完成的工作。按[本片计划](ROUND-02-PLAN.md#d-debt-ledger-01--最小角色当前状态与接手用法收敛2026-10-04)保全起点 `5dc3d1d0` 整行，当前行缩为已有基础调用与剩余发行版适配，细节见[台账 Wave](waves/WAVE-20260929-DEBT-REFERENCES.md#最小角色长状态与当前接手入口2026-10-04)。中英指南只纠正当前能力分类并链接现有 SSOT，没有缩减产品承诺来假装实现。
- **证据 / 当前接续**：工程冻结点 `5dc3d1d0d7e43d6ee30c3ca3cbb194e36786cae2` 的 `17-ci-local-final` 原生 exit 0、前后 SHA 相同、临时 Python 配置恢复 true；已按既有授权推送，正式 CI 另核，不用父 SHA 的绿灯替代。文档迁移机械核原行/前四列/其它权威行，适用门禁完成后只建本地提交，随下一实质批次统一推送。
- **状态 / 停止**：两项债仍 Partial；未修改 QUEUE、机器计划、产品源码、模板、门禁、旧证据或运行数据。本片不重新跑 Rust/真实模型/媒体/业务身份，整理这一处后即停，不让历史未覆盖范围变成无限补证队列。下一施工仍从选定发行版的实际基础接入断点出发，公共边界改变时再列具体决策；暂缓的签名、双核、Event Stream 等不变。

### DCL-20261004-25 · 生成消费测试在 CI 彩色日志下误拒绝

- **实际失败 / 归因**：`5dc3d1d0` 的[正式 run 37149237736](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37149237736)中原 CLI clippy/测试步骤通过，新增消费步骤 native 101。作业 `111279751539` 的原始日志显示子 Cargo 的消费和 rustdoc 各一项成功，但 `CARGO_TERM_COLOR=always` 在 `Doc-tests` 与 crate 名之间插入 ANSI 控制字节，外层纯文本标题断言拒绝；这是我新增测试的日志合同缺陷，不是生产入口失败，也不能因此忽略正式红灯。原日志保存在 `.cursor/plans/debt-minimal-role-library-20261004-r0/18-remote-cli.log`。
- **修复 / 有效回归**：只在该测试的子 Cargo 参数中指定 `--color never`，保留 native exit、命名消费和两项成功计数断言，不新建解析器或修改工作流。按[返修计划](ROUND-02-PLAN.md#生成库消费回归--cargo-彩色日志合同修补2026-10-04)在同一进程级 `CARGO_TERM_COLOR=always` 下，旧测试 `19-colored-consumer-red` 原生 101，修后 `20-colored-consumer-green` 原生 0；两次均恢复原环境。定向 Clippy/fmt 和文档门禁后冻结，main 修复的完整本地链与新 SHA 远端单独核。
- **状态 / 节奏**：D-CLI-BLUEPRINT-05 保持 Partial，`5dc3d1d0` 不标作远端通过；文档迁移的本地提交 `4f98d34e` 与这次必要修复合批，不为中间片单独触发全量。未改模板、产品 API、依赖、角色、模型、用户数据或暂停项；确认这一实际失败修正后停止扩证。CI 红灯归因与失败原件保留，不以无变化 rerun 或删断言掩盖。

### DCL-20261004-26 · CI 公共契约与模型特化的渐进分责

- **维护者确认 / 原因**：Kernel CI 与面向特定 7B 模型的工程应按责任分开，模块化能为选路提供实际依据；无需完整重设计。复用 [K-CI-IMPACT-01 原计划](ROUND-02-PLAN.md#k-ci-impact-01--公共契约与模型特化的渐进归属2026-10-04)和 [CI 设计 §2.4](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#24-主工作流的执行所有权)，不新建债或第二份架构表。
- **实际变化 / 有限出口**：设计补明公共接口与具体模型质量/资源承诺分列；先整理实际接口、依赖、责任和独立测试，需要时才拆代码。下一次实施只选一个有实际重复/过选依据的范围，足以决定路由即停；公共依赖、组合回归、真实 Compare 与未知保守策略保留，不通过改名或删影响边冒充模块化。
- **基线 / 状态 / 验收范围**：起点 `5b3153ae3bbe0a0bf0b3359202d22c36a0f62b89` 干净，仅设计、计划和本事件三文件；K-CI-IMPACT-01 状态及当前 workflow、模块描述、validation catalog、影响图、选择范围、ci-gate 和暂停项不变。按该计划完成适用文档/债结构、编码及 diff 自查后本地保留，随下次实质批次推送；不为设计片重复 Rust、业务验证或全量 CI，不声称分层代码已实施。
- **本地复核**：默认及本片链接、docs-only 旧路径、文档登记/镜像、债计划结构和编码均原生 exit 0，三份改文逐文件核有汉字、无 BOM 和连续问号；`git diff --check` exit 0（Git 换行提示另列）。日志在 `.cursor/plans/debt-ci-boundaries-20261004-r0/`，主控按最终 diff 自查，不声称独立审查。完成方向落点即停止扩证；最小角色的产品入口问题由下一事件接续。

### DCL-20261004-27 · 最小角色发行版入口需要产品范围裁定

- **有界调用链核对 / 真实断点**：`PreparedMinimalRole` 和基础结果不激活丰富角色、不承诺落库/恢复；ChatPro 的 `distros/shared/src/api/role.ts` 仍要求丰富 `RoleData` / `RoleInfo`，`roleStore.mapRoleInfo` 应用关系/人格等产品状态，`chatStoreSend` 消费丰富回复并更新状态。已有 Rust 入口不能靠补默认字段或前端强转接入这些消费者；达到这一断点即停，不扩查全部聊天/媒体路径。
- **待决 / 保护 / 下一步**：已询问独立基础会话与主聊天统一接入的产品范围，区别在角色生命周期、历史与恢复要求；[待决计划](ROUND-02-PLAN.md#d-cli-blueprint-05--参考发行版入口的待决边界2026-10-04)记录接续位置。回答前不改变生产契约、传输、加载缓存、选角或 UI；回答后列所选有限写集与必要回归再施工。原债保持 Partial，已有共享契约/转换器责任和暂停项不重开。只读核对可安全重做，原业务身份和冻结证据不得复用或改写。

### DCL-20261004-28 · 最小角色准备与 Prompt 实现解除固定绑定

- **维护者确认 / 原因**：先在 Host 与能力之间提供最小角色的共享消费者，Host 仍经原六槽契约对接。已有 `MinimalRolePrompt` 将人设/材料准备与固定拼接绑在一起，另一个 Host 不能复用准备去选择自己的 Prompt。按[本片计划](ROUND-02-PLAN.md#d-cli-blueprint-05--可选择-prompt-的最小角色共享消费者2026-10-04)先处理这一个实际缺口，DCL-27 的 ChatPro 产品入口留待后续选择，不阻塞共享切片；不重新调查全部六槽或建通用调度框架。
- **实现 / 例子 / 自行决定**：新增 `MinimalRolePromptConsumer`，借用最小定义与 Host 自选 Prompt，使用唯一准备函数保留原人设、每片材料、次序/边界及额外要求，一次调用后原样交还结果/完整错误。例如 Host 可选择能处理“保留主题”要求的 Prompt，而不是为了使用旧拼接实现偷偷清空要求。旧实现仍选择 Literal，保留原输出和 Unsupported 说明；自查发现直接持有通用消费者会改变旧类型的 auto traits，改为保留原定义引用并共享准备函数，编译断言确认原 Send/Sync。无新 DTO/trait/依赖、状态或权限，也没有填关系/人格默认值；模块权责与角色使用分别更新各自 SSOT。
- **已取得证据 / 边界**：新外部调用先在旧源码因模块不存在原生 101，实现后四项通过，覆盖自选 Prompt、全部五类 typed 错误/正常空输出、一次调用及真实 Pending 后仍可读的借用；旧最小 Prompt 两项、runtime lib 273 项、独立 Host 示例五项与参考 Host 最小入口五项通过。日志在 `.cursor/plans/debt-minimal-role-consumer-20261004-r0/`，后续适用门禁与冻结批次完整本地链分开记录。主控直接实施和 diff 自查，不声称独立审查或 runtime doctest 已执行；新公开调用面由真实外部集成测试编译。原债仍 Partial，真实媒体、产品生命周期、HTTP/Tauri/UI 与其它暂停项不扩大。
- **适用复核 / 批次节奏**：runtime all-targets/all-features Clippy、最终 fmt、workspace doctest（46 项，runtime 既有 doctest=false）、分层、模块兼容、默认及四份改文链接、文档登记/镜像、债计划结构、编码及 diff 均通过。首次 fmt 只拒绝我新增测试的一处换行，原日志保留，规范化后通过。逐文件汉字/BOM/连续问号检查与最终 diff 自查完成。上一生成库冻结点 `5b3153ae3bbe0a0bf0b3359202d22c36a0f62b89` 的正式 [CI 37153013370](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37153013370) 已 success、17/17；不替代当前新增源码的验收。此片与已核设计提交合成一个里程碑，冻结后一次完整本地链并推送，目标远端单独核，不用原债 Partial 混称未取得的全发行版成功。

### DCL-20261004-29 · Host 按次选择 Prompt 而不改变默认调用

- **原因 / 实际接续**：公共消费者已在 `6784db4cc1b7daa5fbcaefe3720f26c235e558ce` 实现，但参考 Host 的基础方法还只能选择原拼接实现。按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--host-显式选择共享-prompt-消费者2026-10-04)补上 Host 的真实调用和独立示例用法，达到这个断点后停止，不为其它五槽加同形包装。原债继续 Partial；ChatPro 产品入口选择、暂停项和已冻结的转换责任不重开。
- **改动 / 例子 / 自行选择**：增量 `process_minimal_message_with_prompt` 薄转发到与旧方法共用的基础编排，由消费者准备一次再调用 Host 选定的 Prompt。旧签名、默认要求协议、模型设置和基础结果保持。选择只属于本次调用，不写全局注册或 Host 状态；空输入与 Prompt 失败均在模型前停止，完整错误与正常空结果保留。例如独立 Host 可显式选“逐片 JSON 引用”，解码后材料逐片相同，而旧 Literal 仍拒绝它未承诺的非空要求；引用格式仅属示例，不代表 Kernel 统一要求语言或防注入保证。没有新 trait/DTO/依赖、资产读取或伪造产品状态。
- **必要证据 / 真实边界**：两个调用面先在旧源码各因新方法/实现不存在编译红（native 101），修后参考 Host 九项外部测试包含原五项兼容、非 Send 本地异步、五类完整 Prompt 错误、空输入零调用、正常空输出及原模型失败；同一 Host 再走默认入口的回归证明确实未记住前次选择。Host lib 635 项、独立示例六项与 native 运行通过；定向两个 crate 的 all-targets/all-features Clippy 及 workspace doctest 47 项通过，新增 Host `no_run` 仅证编译，runtime 的 doctest=false 保持。首次 fmt 拒绝我新增代码的纯格式，原日志保留，规范化后继续。日志在 `.cursor/plans/debt-minimal-role-host-choice-20261004-r0/`，没有真实模型、用户数据或业务身份。
- **复核 / 验收节奏**：控制方按生产者、消费者、兼容、完整错误与有限写集自行复核，不声称独立审查。角色使用、模块接线和主台账状态格按各自 SSOT 同步，原债前四列保护不变。最终 fmt、分层、错误码漂移、模块兼容、默认及五份改文链接、docs-only 旧路径、文档登记/镜像、债结构、编码与 diff 均原生 exit 0；五份改文逐文件核汉字、无 BOM 和连续问号。两片合为一次实质提交并只跑一次冻结完整本地链；父批远端与新 SHA 分开核，不推送去取消父批。剩余丰富生命周期、实际发行版、HTTP/Tauri/UI 和媒体范围不据此转绿。
- **2026-10-04 目标结果收口**：实际代码提交 `8a642fa004cb62fb9364b66bc2bc33eed4b9552a` 的 `npm run check:ci-local` 原生 exit 0（790.518 秒，Dimension 5 30 项；本机 `22-ci-local.result.json` 确认前后同 SHA、工作树空、临时环境恢复）。[CI 37160662341](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37160662341) 已 completed/success、17 个作业全部 success，不再沿用父 SHA 的通过来代替本片。共享消费者与两个 Host 的有限接线里程碑通过，D-CLI-BLUEPRINT-05 仍 Partial；更新这三个现行文档后只核适用文档门禁，本地保留并随下一实质批次推送。ChatPro 产品范围待维护者明确，基础 Rust 接入成功不自动决定它的历史、恢复或角色生命周期。

### DCL-20261004-30 · 主流程目标与核心外共享适配已确认

- **维护者选择**：最小角色进入发行版主流程；通用适配由项目共同维护的共享运行库承担，保留小 Kernel 边界，在外层提供方便复用的一层。原独立基础会话/小 Kernel 直接拥有回合的选择不再待答，不能把主流程解释为承诺所有产品扩展、历史或恢复。
- **已核事实 / 有限施工**：六槽 Base 请求已不依赖完整 Role；公共最小消费者当前只准备 Prompt，独立 Host 的 Prompt→LLM 衔接仍自行组织。按[下一片计划](ROUND-02-PLAN.md#d-cli-blueprint-05--核心外共享基础文本消费2026-10-04)复用已绑定两种 Base，提供可选的基础文本操作与一个实际消费者接线。Host 仍负责材料、能力/资源绑定及结果应用；依契约合规且满足本次基础用途的实现可替换，不承诺任意实现、不可用资源或不支持要求都能成功。其它槽按需使用，不固定六阶段。
- **保护 / 状态**：只沿已确认方向实施实际复用断点，不迁移核心职责或丰富生命周期，不填产品默认值，不暗增能力调用/权限。父债保持 Partial，主流程传输与 UI 接线尚未完成；签名、双核、Event Stream 等暂停项不变。

### DCL-20261004-31 · 核心外共用最小角色的基础文本操作

- **实际问题 / 实现**：按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--核心外共享基础文本消费2026-10-04)增加 `MinimalRoleTextConsumer`，复用已有最小 Prompt 消费者，再将输出原样交给绑定的 LlmBase，正常正文直接返回。独立 Host 的显式选择分支已复用这一操作，原默认路径和角色/资产/技术身份校验保持。公共错误分辨 Prompt/LLM 阶段且包裹完整 BaseCallError；无新六槽 trait、Host 数据要求、依赖、状态或授权，也不将两个实际依赖步骤变成所有 Host 的固定流程。
- **原因 / 例子 / 自行处理**：Host 不必各自重新写“准备人设→组装→生成”的基础衔接，可将自己的 Prompt 与 LLM 绑定给共同实现。例如示例的 JSON 引用 Prompt 与 Echo LLM 用这条共用操作，而默认入口仍保留原 Literal；两者不依赖 ChatPro 关系/人格字段。示例继续使用既有窄错误载体，公共操作保留完整阶段与错误，不为了兼容旧 LlmClient 改写 AppError 或强制迁移参考 Host。模型/资源不可用仍如实失败，不能以可替换接口承诺任意实现都能完成任意用途。
- **必要证据 / 自纠**：新外部调用先因两个公开符号不存在编译红（101），实现后八项外部回归通过（原四项＋新四项），包含两阶段各五类错误、Prompt 失败模型零调用、正常空值、逻辑无效零调用及两个真实 Pending 边界。runtime lib 273 项、旧最小 Prompt 两项与独立示例六项/native 通过。自己新增测试曾在完成后仍持有 future 时移动被借用原文，编译器拒绝；改为只复制断言预期。Clippy 又拒绝单元素预期 clone，改为 from_ref 借用比较；两次失败、首次 fmt 差异和修后结果各自保留在 `.cursor/plans/debt-minimal-role-text-consumer-20261004-r0/`，没有放宽 lint 或实现语义。
- **复核 / 范围**：控制方按材料/要求、一次调用、阶段与完整错误、借用及默认兼容自行复核，不声称独立审查。适用门禁和冻结批次的完整本地链分开记录；runtime doctest=false 不改，公开操作由真实外部集成调用编译。小 Kernel、主流程传输/UI、持久化恢复和其它槽未改，父债仍 Partial；这片不是 ChatPro 主流程完成或所有发行版/真实模型已验收。
- **2026-10-04 局部验收**：最终源码的 runtime all-targets/all-features Clippy、fmt、workspace doctest 47 项、参考 Host 九项兼容及示例 native 运行均 exit 0；分层、模块兼容、默认/已改文链接、docs-only 旧路径、文档登记/镜像、债结构、编码与 diff 检查通过。生产及测试 diff 已在有限写集内自行总审，未增加状态、副作用或核心契约。接下来冻结本批提交，只跑一次完整本地链，目标远端与父 SHA 分开判定；全量结果尚不由这些局部门禁代替。

### DCL-20261004-32 · 共享基础文本操作的冻结点已通过

- **目标证据**：`692aa7f4c51fcd89c9282dd1745f569a203a3516` 的一次完整本地 `check:ci-local` 原生 exit 0（1091.399 秒，同进程核前后 SHA / 空工作树 / 临时环境恢复）；[正式 CI 37184869027](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37184869027) completed/success，17/17 包含 ci-gate。推送后本地 / origin/main 相同且工作树干净；原始 receipt 在 `.cursor/plans/debt-minimal-role-text-consumer-20261004-r0/`。监听首尝试因 GitHub API EOF exit 1，重新连接同一个 run 最终 exit 0；没有将它归为产品 / CI 作业失败，没有 rerun 或为回写绿灯再造提交。
- **范围 / 接续**：只收口 DCL-31 的共同基础操作和独立消费者，不把父债改 Done。核心外共享层、发行版主流程方向已明确；在下一实质传输片一并入账这份结果，避免证据回写触发全矩阵。选角 / UI、丰富生命周期与媒体范围仍各自保持边界，签名、双核、Event Stream 等暂停项不变。

### DCL-20261004-33 · 最小角色的受保护基础传输

- **原因 / 实际接线**：主流程的丰富角色 / 回复不能靠填零状态承接最小角色。按[有限传输计划](ROUND-02-PLAN.md#d-cli-blueprint-05--主流程接线的最小文本传输适配2026-10-04)增加来源与消息的共享封装，复用唯一逻辑 JSON / 本地快照加载器与 canonical 基础编排，再经受保护 `/chat/minimal`、已鉴权桌面 Rust 桥、注册 IPC 和 shared API 输出现有基础 DTO。例如开发者将自己的包转换成一个显式指定文件与资产根，传输无需它提供关系 / 人格 / 蓝图，回复也不捏造这些状态；本片尚未把选角 / composer 接到这一调用。
- **自主实施决定**：默认传输绑定既有 Literal Prompt，非空要求先按本端点输入规则拒绝，不清空或做通用 Base-error wire 映射；旧进程内可选 Prompt 的 typed 错误保持。文件 / 局部非 Send 调用由 Host worker 和当前 Tokio Handle 承担，无 runtime 返回错误，不将 Send 限制加给六槽。加载预算是参考 Host 的固定本地策略，不是作者通用要求。桌面 JSON 调用复用既有带令牌客户端，成人路由的 URL / 错误标签保持；不在渲染层暴露令牌、放宽 Host 鉴权或新增重试。
- **已有窄测 / 自纠**：新 Host 外部测试先因 DTO 缺失 native 101，实现后五项通过，包含真实 Router 鉴权、一次模型 / 原文结果、坏来源 / 越界 / 三类字节超限 / 要求拒绝、原模型错误与无 runtime；九项旧基础 API 回归通过。shared 三项新测试与旧 stream / send 合计 27 项通过，桌面二项 DTO / 既有错误合同通过，typecheck / ESLint 已通过。初次 shared 命令误用根目录路径而未找到用例，正确 cwd 后实际执行；首次 fmt 命令使用了不存在的 package，纠正后记录纯格式差异并只格式化写集；新增测试一处箭头括号 lint 已修，没有放宽门禁。失败与成功各自保存于 `.cursor/plans/debt-minimal-role-transport-20261004-r0/`。
- **验收边界 / 状态**：控制方直接实施并做语义自查，不声称独立审查；后续适用门禁另记实际结果。Router 用内存模型且不监听端口，shared 用 IPC 替身，桌面合同测不算真实 IPC / TCP / webview 通过。旧丰富角色、发送 / 恢复 / SSE、六槽 trait、核心职责与用户数据未改；未取得选角 / UI 接入、收据 / 历史 / 恢复或媒体证据。父债仍 Partial，这片仅本地保留，主流程关联切片完成后再对预定冻结点跑一次完整本地与目标远端 CI。
- **2026-10-04 本地总审**：最终两端 all-targets/all-features Clippy、fmt、workspace doctest 48 项、typecheck、ESLint、layering、错误码漂移、module-compat、默认 / 五份改文链接、docs-only 旧路径、文档登记、债结构、编码与 diff 均原生 exit 0。typed 输入、鉴权挂载、原文与单次模型、旧成人 URL / 标签、worker / 非 Send 边界及所有状态声明已按真实 diff 自查；父债前四列机器比对不变。预期注入错误 stderr 与 Git 换行提示未写成“零错误日志”。无新依赖 / 核心 trait / 官方角色 / 业务身份或真实外部调用，不为此开发切片重复全量；结论限于 Locally verified，下一片直接做主聊天的最小角色状态和基础结果消费。

### DCL-20261004-34 · 主聊天接线前的临时最小会话消费

- **原因 / 实现 / 例子**：按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--主聊天接线前的临时最小会话消费2026-10-04)新增 shared 的临时最小会话 store。旧 roleStore 将选角解释为完整关系 / 人格状态，旧 chatStore 的删除 / 编辑又会调度历史缓存；本片让主聊天后续消费独立的基础来源与气泡，不伪造 RoleInfo、不顺带写历史。例如“取消后模型仍返回正文”只丢弃旧客户端结果，不转发普通发送或 recover，更不解释为 Host 已取消生成。
- **自主处理 / 关联闭环**：来源是复制冻结的临时绑定，Host 每次发送仍执行唯一文件 / 资产校验。发送原文与正常空回复保留，返回身份 / unavailable / 正文类型错误明确拒绝；完成气泡只用于当前展示，不回传作历史。真实 bus 的提交 / 最终事件标 text-only；直接发现语音提交消费者还会加载配置与预热，故让其尊重 skip_auto_tts，旧未标记行为保持。取消 / 新发送 / 重绑定按代际失效，旧 finally 不清新加载状态；不添加后端取消、收据或扩展状态。
- **验证 / 自纠**：新 store 在源码不存在时测试导入失败，语音负例在旧实现上实测加载配置一次，修后 store 十三项与语音六项通过；最终 shared 全套 52 文件 / 260 项通过，typecheck / ESLint / module-compat exit 0。首次 red 日志的相对根计算错，重定向失败且未启动测试；改绝对日志路径并启用 Stop，真实 red 另留。首次 typecheck 错选 distro 自身 tsconfig，产生非本片诊断；改用仓库既定 `npm run typecheck` 后通过。自身新增对象列表换行 lint 14 项由限定文件的 ESLint fix 修正，无 lint / 类型规则放宽。全部失败与成功保留在 `.cursor/plans/debt-minimal-role-chat-state-20261004-r0/`；预期注入 stderr 不作“零错误”声明。
- **复核 / 止点**：控制方直接实施并自行语义审查，不称独立复核。本片是主流程所需的复用状态与结果消费，尚未开放选角 / composer，不增第二个会话产品；未执行真实 IPC / webview / 文件 / 模型 / 音频，也未改 Kernel / Host / 依赖 / 角色包或历史证据。父债 Partial，选角与扩展不可用 UI 下一片完成；关联批次收口时再做一次完整链，不为这个前端片重新跑 Rust / doctest 或全量 CI。
- **2026-10-04 局部收口**：module-compat、默认 / 四份改文链接、docs-only 旧路径、文档登记 / 镜像、债结构、UTF-8 与 diff 检查均 exit 0；债行前四列保持原文，新增文件和已有两份语音文件均在预先写集内。取消 / 原文 / 状态代际 / unavailable 的直接分支与旧兼容已足以支撑下一片施工，停止扩证；此回滚点为 Locally verified，尚未推送或用新 SHA 全量 CI，不宣称父债 Done。

### DCL-20261004-35 · 最小角色接入 ChatPro 基础主流程

- **原因 / 实际结果 / 例子**：按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--两套主聊天界面的最小角色接线2026-10-04)在 Fluent / Tool 的现有选角区增加共同来源控件，复用主 composer / 列表和共享临时状态。开发者转换后的定义可指定任意相对文件名与绝对资产根，不补关系 / 人格 / 蓝图；用户在原聊天位置发送和看到基础回复，扩展明确不可用，可返回原完整角色。详细现行用法见[角色边界 §0.13](../ROLE_PACK_BOUNDARY.md#013-chatpro-两套主界面的基础接线)。父债仍 Partial，不以界面接线替代全部发行版 / CLI 默认生成器的收敛。
- **自主选择 / 关联收尾**：保留丰富角色上下文，用临时选择而非伪造 RoleInfo / Host 激活；绑定前验证、取消客户端和成人队列，取消失败不绑定，过渡期间拒绝新发送。屏蔽丰富工具并对轮询 / 插件角色变更 / ASR / 语音预热 / 热键加范围守卫，模型管理保留。发现旧按键注册器在 disabled 后不会消费 keyup，补停用时释放 pressedHold；MainShell 只为自身已开始的录音发一次 stop，返回后可重启。旧包内联主题随范围清除 / 恢复，不迁 Kernel 或六槽职责。
- **自纠 / 必要反例**：初次主流程测试在缺少新控件时导入红；jsdom 缺 ResizeObserver / matchMedia 造成夹具失败，补环境替身而不替换实际列表 / 角色 store。自行复核发现历史分割误用 -1 会隐藏较早气泡，改为 0，并用实际 DOM 的连续两轮可见性核对。新增返回旧角色用例第一次填错 pure_chat 桶（default 而非 home），改用唯一 effectiveChatSceneId 后核原气泡回显。真实按键注册器在旧源码上停用时 onStop 为零（red），修后核一次收尾及返回重启；新增代码的导入 / 换行 lint 只对写集用 ESLint fix，无规则放宽。各尝试原始日志保留在 `.cursor/plans/debt-minimal-role-main-flow-20261004-r0/`。
- **证据 / 复核 / 止点**：主控直接实施和语义自查，不称独立审查。主流程七项涵盖真实控件发送、原文 / 本地连续气泡、停止等待晚结果、原错误、取消失败、过渡门控与等待绑定的失效；scope 五项核轮询在途 / 事件停用与返回 / 两层热键 / 主题，原 ChatInput / 语音消费者增加相应回归。原丰富 API / stream / send 和其它前端测试随全套执行；其实际计数与最终适用门禁另附收口。没有新 Rust / 依赖 / 权限 / 蓝图 / 官方角色 / 业务身份，也未执行真实产品 / 模型 / 音频 / 用户库。此前传输、临时状态与本 UI 组成同一冻结批次，只在冻结点跑一次完整本地链再按既有授权推送，目标远端单独核，不沿用父 SHA 绿灯。基本主流程完成即停止扩证，媒体 / 历史恢复与所有发行版边界不自动转验收。
- **2026-10-04 局部收口**：最终前端全套 native exit 0（shared 54 文件 / 274 项；ChatPro 测试配置 24 文件 / 92 项，后者含现有 Theater 用例，不把 92 全计为 ChatPro 自身），typecheck / 全前端 ESLint / i18n / module-compat 全 exit 0；两套壳实际构建产物存在。默认 / 四份改文链接、docs-only 旧路径、登记 / 镜像、债结构、编码及 diff exit 0；父债前四列原文不变。真实 diff 自查已覆盖来源 / 显式范围 / 默认兼容 / 关联停用和返回，当前没有遗留阻塞 finding。随后对冻结后的确切 SHA 做整批完整链和远端验收，结果保存为原始 receipt，不为回写绿灯制造第二轮 CI。

### DCL-20261004-36 · 同一最小角色的六槽可替换消费

- **已有基线 / 目标修正**：上一基础主流程批次已在 `85a9df845dd2b912817745485ec9bd2c43229662` 完成一次 `check:ci-local`（native exit 0，1137.041 秒）并推送，[目标远端 37202649802](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37202649802) completed/success、17/17 含 ci-gate；本地 / tracking / remote 相同且干净，原始 closeout 在 `.cursor/plans/debt-minimal-role-main-flow-20261004-r0/33-closeout.json`。两次 watch 的 API EOF 原件保留，最终重连同一 run exit 0，未 rerun 工作流。维护者随后明确六槽目标：同一最小定义可配合发行版不同的 Memory / Emotion 等实现，不能只做文本回复；完整角色可返回的上下文继续保留。
- **原因 / 实际改动 / 例子**：按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--最小角色的六槽可替换消费2026-10-04)在核心外共享运行库增加 `MinimalRoleBaseBindings` / `MinimalRoleBaseConsumer`。原先只有 Prompt / LLM 消费，现在沿六个原 Base traits 各自接一次绑定调用。Prompt 复用唯一人设准备，其余槽原请求、结果与错误直通。例如一份“耐心向导”最小定义，使用关键词 Memory / Emotion 或更换检索排序和情绪材料分析实现，都能将实际检索 / 分析材料交给最终 Prompt / LLM，不补七维人格、关系或蓝图。
- **自主实施 / 有限证据**：六槽并非公共固定流程；外部测试在两种合法依赖下实际消费六槽，LLM 可直接接受已经准备的输入，Agent / Event 不隐式触发其它能力。新增 API 先因公开符号不存在编译红（101），实现后五项新回归与八项旧消费者测试通过；新测试另核请求字节 / None 与空背景 / 材料顺序、正常空值与五类完整错误、无效内容零调用、局部非 Send 的 Pending 借用与无重入。原始日志按次保存在 `.cursor/plans/debt-minimal-role-six-slots-20261004-r0/`。没有重试、状态投射、角色存储、权限框架、新依赖或六槽契约改动。
- **当前边界 / 下一片**：控制方直接实现并按 diff 自查，不称独立审查；局部门禁结果另记，不用上一 SHA 绿灯替代本片。共享消费面存在不代表当前参考 Host / ChatPro 已装配生产六槽；下一片接真实能力，不用四槽空值或扩展 unavailable 冒充运行。Host 仍负责材料来源 / 命名空间、任务授权、实际依赖、异步调度及结果应用，小 Kernel 保持边界。父债 Partial，真实模型 / 媒体 / 发行版全覆盖与暂停债务不扩查；关联批次冻结时再做一次完整链，不为每片跑全量 CI。
- **2026-10-04 局部收口 / 自行处理**：最终五项六槽与八项旧消费者回归、runtime all-targets/all-features Clippy、fmt、workspace doctest 48 项及适用分层 / 模块 / 文档 / 债结构 / 编码 / diff 检查均 exit 0。Dimension 5 首次因本机未指定可执行 Python 而失败，原输出保留；使用已有 Python 3.12.14，仅在该检查进程临时绑定 OCLIVE_VOICE_PYTHON 后重跑 30 checks 通过，finally 复核环境存在性 / 原值已恢复，未改产品配置或安装依赖。自查把内存情绪替身收紧为实际字面线索并不推断主体状态，随后定向测试 / Clippy 的修后日志另存。真实 diff 已自行核对六个独立入口、唯一准备、完整错误与原请求保持，父债前四列机器比对不变；本地保留这一可回滚片，不把这些局部结果称为新 SHA 全量验收。

### DCL-20261004-37 · 独立 Host 接入原生六槽

- **原因 / 实际接线**：按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--独立-host-的原生六槽装配案例2026-10-04)在既有 `minimal_role_host` 追加一条 Host 明确选择的六槽操作。上一片 `f48095a1` 已是干净本地回滚点；这一片让内容来源 / 资产和六槽共享入口成为可以直接运行的装配用法，保留原基础文本与自选 Prompt。参考生产 Event 依赖真实人格 / 情绪 / 历史，不为最小包捏造这些值；案例改选满足其有限用途的既有 LlmEventAnalyzer，未替换现有产品配置或数值分支。
- **例子 / 自主选择**：同一“好奇的向导”最小定义，从本地文件或内存内容进入同一个 Host 泛型。两组选择原生 KeywordMemoryBase / QueryMemoryRetrieval，以及 KeywordEmotionBase / BuiltinUserEmotionAnalyzer 的 Base 入口（后两者共用同一分析 core）；Prompt 为原 BuiltinPromptAssembler Base 视图，Agent 明确委托已有 ScalarCountAgent 支持的 Unicode 标量值计数。检索与分析 / 委托报告逐项进入最终模型输入，人设一次；候选材料、查询和合法任务由这个 Host 自己提供，不增加作者字段或公共六槽封装。
- **有限证据 / 账本**：新三个案例用例先在旧代码因缺符号编译红（101），实现后原六 + 新三共九项通过，涵盖两种实际内容来源及原生装配、技术身份拒绝零生成、Event 协议 / Agent 任务 / Prompt 要求的真实失败不达正文模型。每条六槽路径 Event 假生成一次 + 正文假生成一次，原 native 演示另有三次基础调用，整个程序预期 7 次假生成；没有真实模型 / 工具 / 网络 / 音频或历史身份。后续拒绝不抹去已完成的 Event 生成，不把 fail-fast 当零调用或回滚。完整错误仍由共享消费者保留，案例的既有窄 HostError 只显示 kind。修正三份 native Base rustdoc 的过时“任何 Host 尚未接入”措辞，生产 AppState 未接仍明确保留。
- **复核 / 止点**：controller 直接实现与语义自查，不称独立审查；原始命令 / native 退出与局部门禁保存在 `.cursor/plans/debt-minimal-role-six-slot-host-example-20261004-r0/`，实际收口另记。这是一个实际独立 Host 消费者，仍不等于 ChatPro / 参考生产 Host 全六槽迁移或两种真实发行版验收；共同适配维持核心外，旧丰富角色上下文 / 接口 / 生命周期不改。父债 Partial，后续生产绑定只处理已定位依赖，不继续在共同消费者上扩证，也不把签名、双核、Event Stream 等暂停项带入本片。
- **2026-10-05 局部收口**：案例九项 + 共享六槽五项 + 旧消费者八项，共 22 项通过；native 示例 exit 0，runtime all-targets/all-features Clippy 与全仓 fmt exit 0。分层仍为 3 imports / 1 FQ refs，module-compat 10 slots / 9 manifests / 7 UI、6 plugins；改文五份及默认 52 份链接、注册表、旧路径、镜像、债务结构 147 行 / 12 auto plans、编码 433 份及 diff 检查均通过。局部复核只核本片九个文件，前四历史债务列保持不变；没有新增产品能力调用、权限、默认值或作者字段。验证已足够支持这片实际装配，停止增加共同消费者的反例调查。
- **关联冻结点**：将已本地保留的共享六槽入口和本片原生装配合成一个批次，在最终干净 SHA 上只跑一次 `npm run check:ci-local`，再按既有推送授权一次推送并核目标 SHA 正式 CI。适用 workspace doctest 已在公开 API 那一片实际通过 48 项；本片只添私有案例与范围注释。完整链 / 远端结果先写交付与本机证据，随下一实质改动入账，不追加证据专用提交；当前父债继续 Partial，完整链也不代替生产六槽接线、真实模型或全发行版的专项验收。

### DCL-20261005-38 · 共享消费者复用参考 Host 的实际正文客户端

- **上批精确基线**：共享六槽与独立 Host 案例已在 `b4d0cc17cfb731a2be0fab1e5b5cf1b30fc18616` 合批收口；一次完整本地链 native exit 0（914.9449802 秒），一次推送后[正式 CI 37220653380](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37220653380) completed/success、17/17 含 ci-gate，tracking / remote 同 SHA 且干净。原始 receipt 位于 `.cursor/plans/debt-minimal-role-six-slot-host-example-20261004-r0/30-closeout.json`（本机忽略产物，不当作 Git 携带证据）。这是上批事实，不证明本片新 HEAD 已全量通过。
- **原因 / 实际接线 / 例子**：按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--参考-host-已装配正文模型的-base-接口2026-10-05)，把参考 Host 实际 `AppState.llm` 的正文调用开放为借用的 `text_generation_base()`。集成方可把它交给共同消费者，而不为最小角色另建绕开已装配资源 / 授权包装的 Ollama 客户端。与旧基础入口复用唯一正文 helper，模型设置 / options None / 输入及正常空结果不改；原接口保留原 AppError，只有新 Base 边界按 typed 事实投射，不解析超时 / 取消文字。settings 同步与环境更新沿旧机制，未承诺零 I/O 或全进程环境隔离。
- **自主实施 / 有限验证**：公共 API 先因方法不存在编译红；同时发现新增测试误把两种 analyze 同名方法写成 analyze_event，改用 trait 明确限定，未改原六槽 API。三项新 Host 回归使用生产 builder + 内存客户端，原生六槽实际消费一份准备内容并把真实材料交给正文；Event 与正文在测试里显式调用同一模型两次，产品默认未变。原九项公共入口回归保持。初次 fmt 与 Clippy 报新增代码格式 / 测试对非 Drop 值的冗余 drop，均只修本片写集，不放宽规则；原始失败、修后验证与复核存 `.cursor/plans/debt-minimal-role-host-llm-base-20261005-r0/`，最终适用门禁数字另附。
- **归属 / 止点 / 下一片**：controller 直接实现与语义自查，不称独立审查。该 borrowed view 位于参考 Host，公共消费者仍在小 Kernel 外；未增加作者字段、全局六阶段、产品记忆来源、工具授权、真实模型 / 网络 / 音频 / 用户数据、HTTP / Tauri / UI wire 或旧角色生命周期。新入口能实际复用当前模型，不等于 ChatPro 已默认消费四个其余槽。达到实际客户端复用即停止扩证；父债 Partial，下一片的产品材料范围按维护者选择处理，关联批次稳定后才一次全量与目标远端，不为这个局部入口单独推送。
- **本片局部出口**：Host lib 637 项、原 HTTP 五项与公共 API 十二项全部通过；workspace doctest 实际 49 项（含新 facade 示例）、Host all-targets/all-features Clippy / fmt 与分层、模块、默认 / 五份改文链接、路径、登记、镜像、债务、编码、diff 及 Dimension 5 30 checks 均 native exit 0。检查 Python 只用已有安装并在本进程恢复原值 / 缺失态。当前无自查阻塞 finding，本地保留；维护者已选下一片只使用当前最小会话对话材料，切换不跨角色共享，不读取旧丰富记忆，不设计持久身份 / 历史恢复。

### DCL-20261005-39 · 当前最小会话对话材料接入 Memory

- **原因 / 用户选择 / 实际结果**：维护者明确先用当前绑定的临时对话，不接旧丰富记忆、不新增持久身份。按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--最小会话的当前对话材料接入-memory2026-10-05)，主 store 发送前快照已完成对话对，增量 DTO / 现有鉴权 IPC-HTTP 薄转发将材料交给 canonical 入口的 QueryMemoryRetrieval Base，选中的原引用文本进入共享 Prompt 准备与原正文调用。例如“她说她不喜欢咖啡”的前轮引用，下一轮询问咖啡会保留否定与原发言人，不把它改写成当前用户偏好；无命中沿原 Prompt，不制造记忆。
- **自主实现 / 兼容边界**：最近八个完整对话对、合计原文 UTF-8 64 KiB 是参考 Host 的有界策略，不是最小作者字段。前端只取连续后缀，不裁文字 / 跳过过大的最新对；Host 超限在加载资产 / 生成前拒绝。取消 / 失败 / 在途不算完成对，正常空回复保留，重新绑定清空材料；快照先于同步提交监听器，晚结果不能写回候选。原无会话载荷和进程内文本 / 自选 Prompt 保持，旧 Host 拒绝新字段时不静默丢材料或换 rich 路径。复核后撤回直接扩充旧 Rust DTO 的实现，新增 conversation 封装 / From / canonical 入口，保留旧两字段 literal、原入口及 backend / HTTP 方法；原 HTTP 路由和 Tauri 命令的薄接线接受新类型，鉴权 / ACL 不变。没有新增 provider 调用、工具权限、作者格式、存储 / 收据或状态字段，也没有假 rich Role，不改历史 B6 证据；相关产品提示中英同步“当前会话”与“持久历史”的区别。
- **真实失败 / 修补记录**：新增 Host HTTP 用例先在旧 DTO 因未知 conversation 而真实失败（原五项过 / 新两项败，101），补增量封装与绑定后七项过。前端最初命令用了错误 workspace 名，实际未执行测试，不计 red 业务证据；更正为 @oclive/desktop-shared。新增主流程请求断言误填了另一用例的文本（26 过 / 1 败），按该用例的 first user / first reply 更正；lint 报新增测试每行两条语句，拆行修复，不放宽规则；首次格式差异与错写 check:types 命令均保留，正确命令为 npm run typecheck。patch 校验失败均未产生源码修改；不把实施者失误归因产品。各次日志原样位于 `.cursor/plans/debt-minimal-role-conversation-memory-20261005-r0/`。
- **复核 / 证据止点**：controller 实施和语义自查，不称独立审查。实际 HTTP 路由核检索材料 / 未命中 / 原载荷 / 预算和 typed 拒绝；真实 store / composer / 主列表与 API 用例核同一 IPC 载荷 / 第二轮 / 原文 / 取消失败 / 换绑定 / 晚结果 / 快照 / UTF-8 最近后缀。模型与 IPC 为内存替身，不声称真实模型质量或桌面进程运行。本片实际门禁与关联批次收口另记；父债 Partial，Memory / Prompt / LLM 的有限生产连接不等于六槽全接线。其它槽按合法材料 / 任务继续处理，签名、双核、Event Stream、真实媒体和持久记忆不自动扩面。
- **本地结果**：Host lib 637 项、当前 HTTP 7 项、公开入口 12 项、桌面桥纯合同 3 项、workspace doctest 实际执行 49 项通过；兼容调整后的 Host / 桌面 all-targets / all-features Clippy 与 fmt exit 0。共享前端全套 278 项、ChatPro 配置 92 项、针对三文件 27 项通过，后者配置包含其它发行版示例，不能当 92 项均为 ChatPro 独占；typecheck / scoped ESLint / i18n 及适用分层、模块、文档、债务门禁通过。与前一片 `3ae400bd` 合批的最终完整本地链和精确 SHA 远端结果按冻结点核对，不沿用父批绿灯。

### DCL-20261005-40 · 当前用户材料的 Emotion Base 线索消费

- **上批精确收口**：正文 Base 与临时会话 Memory 两片已在 `c83e501259ab5793d19b27767ab27d97d2b5f77d` 冻结；一次 `check:ci-local` native exit 0（1089.3560021 秒），一次推送后[正式 CI 37229093997](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37229093997) completed/success、17/17 含 ci-gate，工作树与 remote/tracking 同 SHA。完整链 runner 收尾错误如实保留：本来缺失的 Process 环境变量被 .NET API 恢复成空值，原 receipt 的 environment_restored=false 未改；子进程退出后原调用者无残留，三种缺失 / 有值 / 空值有限对照确认缺失态应以 Remove-Item Env: 恢复。GH watch 因读取 EOF exit 1，后续只读 metadata 核正式成功，未重跑 CI。原始 closeout 见本机 `.cursor/plans/debt-minimal-role-conversation-memory-20261005-r0/47-closeout.json`，不冒充 Git 携带证据或本片新 SHA 绿灯。
- **原因 / 有限行为 / 例子**：按[既有计划](ROUND-02-PLAN.md#d-cli-blueprint-05--当前输入的-emotion-base-线索消费2026-10-05)，让本地最小入口真正消费当前用户材料，而非仅宣称 Emotion trait 可调用。明确选择 BuiltinUserEmotionAnalyzer 的无模型 Base 视图，context=None；完整词表线索及其限制原样进入 Prompt 参考分区。例如“她说我不开心”保留引述和否定，并注明未判定主体 / 引述归属 / 条件，不认定当前用户或角色的情绪状态，也不反推七维。只分析本轮原文，不以角色人设、检索历史或旧 rich 状态作隐含输入；正常 None 不插中性占位、不缓存。正文依然一次已装配客户端调用。
- **兼容 / 自主选择 / 关联面**：canonical 入口组织材料，接口 / 角色作者格式 / 六槽合同 / DTO / HTTP 与 IPC 鉴权 / 回复 wire / 主流程临时状态无需改变；原直接进程内基础文本和自选 Prompt 不自动启用这个入口策略，旧 rich 配置与七维生命周期保持。正常无线索且其它材料相同时保留原 Prompt 字节；有线索才新增明确的参考材料。选择具体 builtin 是参考 Host 的有限策略，不将其词表规则、标签或 context 范围升为通用 Base 要求，不代表已装配的任意 rich provider。固定实现真实词表错误保持诊断并在正文前拒绝，不吞空值或额外重试；未新增通用 Base-wire 映射。只修当前 base_emotion rustdoc 的过时产品消费范围，原分析与词表不变。
- **验证 / 停止**：真实鉴权 HTTP 的两项新用例先在旧代码红（原七项过 / 新两项败，101），实现后 HTTP 九项与原公开入口十二项通过。用内存记录模型证明完整原报告进入最终输入、主体 / 否定限制保持、历史情绪不被当本次分析、无线索的后轮不残留、一次正文与零 rich 激活；原模型错误用有线索输入回归仍保留原错误且不重试。controller 自查，不称独立审查；原始命令 / 日志 / 结果保存在 `.cursor/plans/debt-minimal-role-emotion-material-20261005-r0/`，其它 applicable 结果与本地收口另记。父债 Partial，当前最小生产接线为 Memory / Emotion / Prompt / LLM；Event / Agent 仍需各自合法材料 / 任务，不虚构操作或开启额外模型 / 工具。未执行真实模型、音频、桌面进程或用户库，不声明情绪准确率或所有发行版验收。局部出口后停止扩证，关联批次里程碑才一次全量与一次目标远端。
- **本片局部出口 / 下一片选择**：Host lib 637、HTTP 9、公开入口 12、原 Emotion 外部 / 消费者 / 六槽 7+8+5、真实前端主流程针对三文件 27、workspace doctest 49 项通过，Host / runtime all-targets/all-features Clippy、fmt 与适用文档 / 分层 / 模块 / 债务门禁 exit 0。fmt 首次只因新增断言换行失败，原日志保留后局部修正。维护者选择 Event 按需调用、普通最小聊天默认不增加分析模型调用，Agent 仅在有明确委托时参与；下一片先补参考 Host 的显式 Event Base 资源绑定，不新增主聊天自动调用或 UI 动作。当前先做本地回滚点，与下一片合批冻结后才全量 / 推送，不将局部绿灯称为新 SHA 正式 CI。

### DCL-20261005-41 · 按需 Event 复用参考 Host 的真实模型资源

- **原因 / 用户选择 / 写集**：维护者明确 Event 按需、普通聊天默认不追加分析。Emotion 在 `9fcb4bdd9256b9603b836143124cf6693d2b70dd` 已干净本地保留；本片按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--参考-host-的按需-event-base-入口2026-10-05)新增 borrowed `event_analysis_base()`，经私有适配将现有 LlmEventAnalyzer 绑定到原 HostTextGenerationBase 的真实客户端。调用方可直接交给共享六槽消费者，不必另建未带资源 / 授权包装的模型客户端。role_kernel 只开放能力，实际协议继续由原分析器维护；小 Kernel、types / contracts、请求、wire、状态、插件配置与默认主聊天不变。
- **行为 / 例子 / 自主选择**：工厂零调用，只有明确 analyze 才将“如果她说删除文件，并不表示已经删除”及调用方 context 原值送入已有分析协议；正常报告与 None 保留，格式失败或模型错误完整返回，不吞 None、补发普通回复或修复重试。每次 analyze 一次已装配客户端调用，不声称其底层策略只有一次请求。没有默认角色状态、隐含 persona / 上游槽、Event Ring 发布、Agent 任务或工具授权。这个增量入口是公开 Rust Host 的显式能力，不是 ChatPro 新 UI / HTTP / IPC 分析操作，也不把正文模型适用于分析当作已证事实。
- **有限验证 / 止点**：公共 API 红测因入口不存在而编译失败（101）；接线后原六槽实际绑定案例使用新 view，并新增材料 / context 三态、原报告 / None、协议失败及模型错误不重试的回归。真实 builder、原已装配客户端与隔离库参与，生成器为内存记录器；所有分析次数只在测试显式发起，普通 HTTP 九项仍核单次正文。controller 实施与自查，不称独立审查；日志继续保存在 `.cursor/plans/debt-minimal-role-emotion-material-20261005-r0/`，结果随实际门禁更新。达到按需绑定即停，不加 UI 选项、全部 provider / 模型质量、音频、工具或用户库调查；Agent 等合法真实委托单独处理，父债 Partial。与 Emotion 合批冻结只做一次完整本地链 / 推送 / 精确 SHA 正式 CI，绿灯证据不制造额外提交。
- **本片局部出口 / 合批冻结**：最终 Host lib 637、HTTP 9、公共 API 14 项通过；原 Event / 共享消费者 / 六槽回归 4+8+5 项、workspace doctest 实际 50 项通过，Host / runtime all-targets/all-features Clippy、fmt 与分层 / 模块 / 文档 / 债务 / 编码 / diff 均 native exit 0。首次 fmt 仅因新增测试的换行失败，原日志保留后限定修正。普通 HTTP 仍只调一次正文客户端，显式 Event 的额外分析只发生在主动调用测试；无需改变前端或 wire。适用证据已足以支撑本片，停止扩证；与 Emotion 两片形成同一批次的干净提交，随后仅在该最终 SHA 跑一次完整本地链、推送并核正式 CI。这里记录局部结果，不预写尚未发生的全量 / 远端绿灯。

### DCL-20261005-42 · 已有真实 Agent 向外开放 Base 借用

- **上批收口**：Emotion / Event 两片冻结于 `fb26a15c2338c7dc3daecc62ce931b8b4fc0a663`，一次完整本地链 native exit 0（946.442798 秒、环境恢复 / HEAD 未变 / 工作树干净）、一次推送、[正式 CI 37266206907](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37266206907) 同 SHA 17/17 success 含 ci-gate，零 rerun。本机原件在 `.cursor/plans/debt-minimal-role-emotion-material-20261005-r0/55-closeout.json`；不冒充 Git 携带证据，随本次实质实现补记，不另造绿灯提交。
- **原因 / 有界实现 / 例子**：按[有限计划](ROUND-02-PLAN.md#d-cli-blueprint-05--已有真实-agent-的显式-base-借用2026-10-05)，停止于 L1 已找到的真实 builtin 与私有 Base 视图。外部 Host 已可构造 Agent，但无法复用该报告投影；新增 `BuiltinReActAgent::task_execution_base` 借用当前实例和真实模型 / 角色 / 会话，复用已有 core / parser / bridge，不加默认计数任务、第二 Agent、模型轮次或权限。例如明确委托读取调用方自己的记录，由原模型发现 / 调用工具再形成带限制的报告；最小角色消费者保留独立调用，不在普通聊天自动触发它。
- **兼容 / 自主决策 / 关联面**：沿原 context 拒绝、typed 错误、三轮上限和 branch-facts 报告，无丰富 `AgentInput` 默认领域字段。身份只进入原模型 / trace，不授予权限；未来与外部效果沿旧边界。保留旧 `AgentProvider` / configured single provider 及 rich 产品调用，无新的 `OcliveKernel` 选择工厂、state / wire / UI / 请求 / 角色包 / 依赖变化。不在本片定义 ChatPro 任务按钮、工具集合、合并 Agent 或统一授权语义；实际任务 / Host 接线与 CLI 默认路径仍分别处理。controller 自行实施与语义 / 权限自查，不称独立复核。
- **已取得证据 / 停止线**：五项实际外部调用在旧源码因入口缺失编译红（101），实现后 5/5 通过；真实 core / 原 parser，模型和工具为内存资源 seam。核共享消费者明确委托的两轮一工具、实际身份、原人设不暗加、非空 context 零 discovery、空任务 / 无工具如实未承接、未授予 discovery 权限为 Unavailable、模型 failure 完整且不重试。命令与原始日志在本机 `.cursor/plans/debt-minimal-role-agent-binding-20261005-r0/`；其余适用门禁完成后再记，未预写全量 / 远端结果。父债 Partial，达到这个实际外部断点即停止调查，不再次穷尽内部三轮 / 并发矩阵、真实工具或所有发行版。
- **局部出口 / 本批冻结**：Host lib 637、外部 Agent 5、原最小 HTTP 9 / public API 14、runtime 消费者 8 / 六槽 5、workspace doctest 实际 51 项通过；Host all-targets / all-features Clippy、fmt 与分层 / module-compat / 文档链接、登记、编码、债结构、docs-only 旧路径、diff 均 native exit 0。首次 fmt 只涉及本人新测试的换行，限定该文件规范化，失败原件保留；没有规则放宽或产品失败。controller 自查本片三份 Rust 文件和五份文档，确认原 core / 投射执行字节不改、旧路径不调新入口、无 state / 权限 / wire / 小 Kernel 变化。足以施工的证据已经取得，停止扩查；本片到达现有实现外部借用的有限里程碑，干净提交后只跑一次完整本地链和一次目标正式 CI，不预写尚未发生的结果。

### DCL-20261006-43 · CLI 市场在替换旧安装前共用基础检查

- **上批已冻结 / 并发安排**：Agent 借用入口冻结于 `f849e2a95e3a5035fc70045e87c13ebe2fbb28c7`，一次完整本地链 native exit 0（707.2131963 秒、环境恢复、HEAD 与干净工作树不变），一次推送后[正式 CI 37329575952](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37329575952) 对同 SHA 17/17 success 含 ci-gate，零 rerun；独立源码复核通过。watch 曾因 GitHub API EOF 非零，早期终态查询仍在运行而拒验，原输出保留，未把它们写成测试失败或终态通过。本机回执在 `.cursor/plans/debt-minimal-role-agent-binding-20261005-r0/55-closeout.json` 与 `56-independent-review.json`，忽略目录不是 Git 携带证据。按维护者并发要求，从这一基线开两个工作树，源码和市场文档写集分离，controller 单写计划及台账；未重启最小角色或 Agent 产品调查。
- **原因 / 实际修补 / 例子**：按[有限计划](ROUND-02-PLAN.md#2026-10-05--冻结后并发接续cli-市场基础安装闭环)补 `V-MARKET-01` 已有基础安装承诺。原 Git 分支直接替换目标，本地依赖图又可能读取旧根；例如新插件缺 manifest 或新声明依赖不存在，旧安装仍会被破坏。现以同一个 crate 私有准备函数复用 JSON 对象 / 依赖 parser 与图遍历，图根来自本次 source；Git 再核非空字符串 manifest.id 与索引精确一致，全部这些拒绝先于目标删除。默认或 canonical 别名 source 就是 target 时保留自身，删除目标错误传播；不更改原显式角色装配与非 Git 分支。
- **自主选择 / 复核纠偏**：不为 CLI 引入默认 Host 依赖或复制完整清单 parser，身份比对仅兑现已有索引一致承诺，不新增 ID 语法。实施中的复制方案会意外改变 Git 移动语义，controller 在复核前发现后撤回，最终仍为临时克隆根 / 子目录的原 rename；暂态测试不是最终证据。文档支线修正默认 awesome URL、两处同步路径和重复 OPEN 表述，controller 按实际实现区分 CLI 基础检查与桌面完整清单校验。任务 UI、实际工具授权 / 模型选择 / 结果进入记忆仍是 ChatPro 的待决项，不在此片擅自接入。
- **有限验证 / 独立审查**：实施者在新 TempDir 与本地 file:// Git 夹具上先红测 native 101（12 项中 11 失败），最终 13/13、CLI bin/tests Clippy、两文件 fmt / diff 均 native 0。独立 reviewer 对最终两源码 hash、同一个准备函数、source 根、验证顺序、保留移动与同目录保护复核 PASS，仅读最终日志并区分实施者 native 退出记录，未重复 Cargo；不声称 reviewer 自行跑测。本机原件归于 `.cursor/plans/debt-market-install-20261005-r0/`，controller 合流适用门禁和最终冻结结果随实际更新。未访问线上索引、真实插件进程或用户安装；不扩 IO 回滚、全部重解析点 / 并发安全矩阵、签名或社区发布。父债仍 Partial，已足以行动则停。
- **合流出口 / 本批冻结**：两个独立工作树的精确写集已经保留并合流，共两份 Rust 源和六份文档；controller 在主树实际复跑 13/13，CLI bin/tests Clippy、两文件 fmt、module-compat、默认 / 六份改文链接、文档登记、docs-only 旧路径、债结构 / 计划、六份编码与批次 diff 均 native 0。审查过的源码 Git blob 与合流一致，磁盘 LF / CRLF 的 hash 区别单独记录，未冒充逐字节一致。controller 的首次编码调用误用位置参数，usage 拒绝 native 1，改用既有 `--file` 后通过，原件保留；D-DEBT-LEDGER-01 只追加本次快照引用，原有历史链接保留。没有因为工具使用错误改产品 / 放宽规则。最终相关批次干净冻结后仅跑一次完整本地链与一次推送 / 精确 SHA 正式 CI，正式结果留原始回执并随下次实质工作入账，不再制造纯绿灯提交。

### DCL-20261006-44 · 当前最小角色状态凝缩，历史原行保全

- **原因 / 例子**：`D-DEBT-LEDGER-01` 的既有规则用于本轮实际触及的 `D-CLI-BLUEPRINT-05`，避免每次接手先读数千字的重复验收串。将冻结基线原行完整保全到[2026-10-05 历史快照](../archive/TECHNICAL_DEBT_MINIMAL_ROLE_STATUS_20261005.md)，现行第五列只留 Partial、已有六槽 / 真实 Host 绑定、Event 按需 / Agent 委托及剩余产品边界，逐次证据链向既有 SSOT 与 DCL。
- **边界 / 检查**：前四单元格不改；旧快照不改、历史正文逐字保留、新快照不是第二个活跃状态来源，父债不转 Done。仅处理这一已触及长行，不重数全仓未偿还项或穷尽自由文本冲突；准确登记 raw 行包含、原前四列相等及适用文档门禁。源码修补与机械文档支线均按各自有限复核方式收口，controller 统一验收后再保留本批。

### DCL-20261006-45 · 发行版默认身份兑现用户确认的优先级

- **上批实际终态**：市场安装／文档批次冻结于 `cfcfc7343063d4eba7364523e59117fe68dc4b1e`，一次完整本地链 native 0（987.4727047 秒，环境恢复、HEAD 与干净工作树保持），一次推送后[正式 CI 37341693585](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37341693585) 同 SHA 17/17 success，含 ci-gate，零 rerun；独立源码复核已取得。本机终态原件 `.cursor/plans/debt-market-install-20261005-r0/55-closeout.json`，GitHub watch 的 API EOF 原件保留，不当测试失败。随本次实际修补入账，不制造纯绿灯提交；三个父债仍 Partial。
- **原因 / 用户决定 / 例子**：按[有限计划](ROUND-02-PLAN.md#k-uid-default-02--发行版默认身份的实际优先级2026-10-06)处理已登记的代码／文档差异，不重新扩大支撑调查。维护者明确：合法显式选择优先；未选择时取发行版默认 A；A 不在当前角色 catalog 时回退合法角色默认 B。例如 profile 指定同学、角色包默认朋友时，初始与恢复默认应使用同学模板；当前包没有同学则保留朋友模板，不能变成缺身份的旧关系提示。
- **实现 / 关联闭环**：Host resolver 与身份 service 共用一个 crate 私有默认 helper，global / per-scene / 恢复默认及 DTO 使用同一规则。per-scene 跟随默认标记只取该 scene 覆盖是否存在，不被另一 global 显式状态影响；旧 wire 字段名、sentinel、DB 字段和显式选择 allowed_ids 约束保留。shared 消费者原本已读 DTO 的有效 default_identity_id，只补回归与“跟随默认身份”两语种文案；中英文角色规范／profile 同步实际优先级，已有 DTO 的 rustdoc 修正 pack-only 措辞，公共类型与值不变。没有小 Kernel、六槽 SessionCache、API shape、角色包格式、成人授权或 DB 迁移变化。
- **有限取证 / 自主纠偏**：新 TempDir、真实角色 loader／service／模板 resolver、内存 SQLite 与禁止模型调用的 seam。最初 fixture 未显式覆写 exporter 默认的 per-scene，暂态 5/6 被 controller 查明为夹具错误而非产品关系同步缺陷；修正并断言实际加载模式后，在原源码重新红测 native 101（1 过 / 5 败），最终实现六项全过。旧输出保留，不把第一次夹具结果当正确 global 证据。后续 applicable 原始日志保存在本机 `.cursor/plans/debt-user-identity-default-20261006-r0/`，忽略目录不是 Git 携带证据。controller 实施与语义自查，不称取得独立 agent 审核。
- **停止线 / 状态**：`K-UID-DEFAULT-02` 当前 Partial，待实际本地相关门禁与批次最终 SHA 正式 CI 再收口，不预写完整／远端通过。达到默认规则、直连 API／实际模板和消费者闭环即停；不重查全部身份历史迁移、legacy relation-only 接口、真实用户 DB、模型／音频或全部发行版。上一批仍冻结在 `cfcfc7343063d4eba7364523e59117fe68dc4b1e` 等正式终态，本片独立工作树开发，不覆盖或借用旧结果。
- **局部出口 / 自查**：六项直接身份测试、Host lib 637（含已有 HostProfile 配置解析）、shared 身份消费者 3 项（新增 2、原成人切换次序 1）全部通过；Host all-targets / all-features Clippy、五文件 fmt、前端 ESLint / typecheck、分层 / module-compat、两语种镜像、七份改文链接、文档登记、编码／中文及债结构／计划、diff 均 native exit 0。工作树复用主树既有 node_modules 的精确 Junction，无安装；Vitest 日志 root 与 alias 指向该工作树自己的 shared 实现。台账前四列逐字未改，controller 对实际 source / tests / docs 复核，公共 shape、关系 sentinel 和授权分支未变；日志保留初始 fmt 与 fixture 失败。当前已足以支持该实现，停止扩证；干净批次最终 SHA 再取得一次完整本地链及正式 CI。
- **首次完整门禁的实际拒绝**：原冻结 `ee1b0939b2ccab79217fcd1abf56d91810dfc553` 的完整本地链 native 1（93.6814357 秒），Dimension 5 拒绝本人新测试的 `root.path().join("roles")` 写法；测试使用新 TempDir，但不是检查器已有的 `dir.path()` 夹具写法。仅将 TempDir 局部变量改为 `dir`，路径／隔离／执行语义不变，门禁规则未改；先核 code-only 旧路径检查与直接身份回归，再冻结新提交重跑最终完整链。原 50-full-local 日志、SHA 和环境精确恢复回执保留，不把这次失败算作全链通过。
- **实际收口**：最终 `58be11faba622dc87ed29ef677ea444cf2dd6599` 第二次完整链 native 101（1165.704004 秒），根因是控制端全局导出的 `CARGO_TARGET_DIR` 令嵌套 CLI 生成项目的二进制产到共享目录；三项找不到自己 target 下产物，不是业务源码回退。取消这一覆盖后同源码 CLI monolith 8/8、第三次完整本地链 native 0（1128.6415517 秒），一次推送后的[正式 CI 37353477334](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37353477334) 对同 SHA 17/17 success、含 ci-gate，零 workflow rerun。`55-closeout.json` 核 HEAD / tracking / remote 一致、工作树干净；三个完整尝试原件保留。本债第五列迁为默认选择合同 Done，历史前四列保留；源／测试／中英合同可随 Git 交接，忽略的本机证据不可冒充可携带。随下一实质切片回填，无纯绿灯提交。

### DCL-20261006-46 · CLI 资源预览复用 Host 编译器

- **起点 / 原因 / 风险**：上述已正式验收的 `58be11fa`；`D-SCAFFOLD-RESOURCE-01` 有维护者独立实施授权。现有 `doctor execution-plan` 仍不评估资源，但已有 types、registry validator 和纯候选编译器足够复用；本片停止调查而实施只读诊断，M 实施／最终 main 保留升 L，controller 语义自查，不称独立复核。
- **实际接线**：新增 opt-in `doctor resource-plan`，显式诊断文件、可选发行版 profile／GPU 索引 → canonical 类型 → 原所有者校验与有限意图 validator → 原纯编译器 → 人类档位／估计量／选择／拟议动作／原原因码或原 candidate JSON。输入不改，旧评估不沿用，未知版本／坏类型／重复 ID／非法所有者非零；blocked/degraded 正常诊断 exit 0。没有 controller 实例，空 controller_ids 保留原拒绝原因；managed 描述及无转换 executable 候选均不代表实时执行许可。默认 feature 明确拒绝且不读缺失输入；无新依赖、schema、解析器、设备探测、模型／进程／服务／用户库或执行面。
- **原始验证 / 状态**：首次红测错误使用 `--exact` 未带模块名，native 0 但实际 0 tests，不算证据；纠正过滤后旧命令 native 101／1 failed（子命令不存在），实现后第一版 6/6。新增坏类型／profile／缺文件后继续定向验证。第一次 fmt check native 1，仅本人新增文件格式，按 rustfmt 修正；失败与零测试原件保留于 `.cursor/plans/debt-resource-preview-20261006-r0/`，后续输出各用新身份，不覆盖。父债 Partial，配置编辑／交互 round-trip／自动硬件建议未实现，资源父债不改；达到只读 preview 与适用出口后停止扩证。
- **局部实际出口 / 自主纠偏**：最终实际子进程 CLI preview 7/7，默认 feature 拒绝 1/1，原 Host 资源编译器 8／registry 9／旧 execution-plan 1；CLI all-targets/all-features Clippy／fmt、分层／中英镜像／默认和 8 个改文链接／文档登记／旧路径／债结构／编码与 diff native 0。控制端新窄测批处理的第一次调用在首个 Cargo 之前失败（PowerShell `-Command` 实参不会自动成为所设 `$args`），修正控制脚本的参数引号后全部通过；仅作转录归因件，不冒充 Cargo 原始日志或产品缺陷。controller 复核实际源码与类型／权限链，132 个粗体台账行前四列相等，diff 只动两条当前状态。达到本片停止线，不再补低收益反例；待干净冻结 SHA 的一次完整本地链、一次推送与精确 CI，不能用上述窄测预写通过。
- **必要 CI 关联**：现有 CLI opt-in 测试步骤增加本集成目标，原库消费者仍运行；无新 job、矩阵或门禁责任变更。默认构建只执行 feature 拒绝用例，不能用它替代七项 preview 行为回归；对应组合命令本地实跑 native 0（53.6927036 秒，preview 7／实际生成库消费者 1），两种 feature 的 CLI Clippy 均零诊断。冻结后完整链与正式结果仍以目标 SHA 为准。
- **完整链与首轮中断**：冻结 `9edfe719d71e32edf27bd6a482a0e56a3c52b403` 一次完整本地链 native 0（693.7319829 秒，环境恢复、HEAD／干净工作树保持），一次推送。[正式 CI 37364798271](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37364798271) 首轮 failure，原 Linux 日志为 runner shutdown signal／lost communication，五个取消 job 的注解为未取得 hosted runner，未观察到测试失败，CLI job 成功；runner 关闭的底层原因未确立。原始 API／日志保全后只补跑一次 failed jobs，已成功 job 保留，不改源码或全量重跑；未终态前不称正式验收。实际原件 `.cursor/plans/debt-resource-preview-20261006-r0/44–49-*`；后续终态只随本次实质配置切片回填，不制造绿灯提交。

### DCL-20261006-47 · CLI 非交互资源策略草稿

- **原因 / 有界计划 / 例子**：按[有限计划](ROUND-02-PLAN.md#d-scaffold-resource-01--非交互资源策略草稿2026-10-06)，预览已有而开发者仍须手工改完整 profile，本片停止调查并实现新草稿。例如只改 GPU 保留量和 resident 约束，其余身份／记忆／扩展设置保留；新文件可再次交原 preview，不能改变正在运行的资源。M 实施、最终 main 冻结升 L，controller 实施与语义自查，不称独立复核。
- **实际闭环 / 自主选择**：`config resource-policy` 的显式 profile／原资源节 TOML patch／capture／新输出 → 通用 TOML 值局部覆盖 → 同目录临时草稿 → 原 Host loader、同一个 preview registry／有限意图 validator／候选编译器 → create-new 发布。commands 整组替换／显式清除，缺省键及非资源 TOML 值保持；格式与注释不保留，故禁止原地编辑和覆盖，不另造 schema／typed parser／依赖。有限意图 blocked 拒绝；硬件或权限原因的有效降级照原原因码报告，草稿不提供执行权。默认 feature 在读写前拒绝；既有 config 命令、Host／六槽／角色格式／运行调度不改。
- **原始失败 / 当前状态**：实际新增 CLI 子进程用例先红测 native 101／1 failed（旧子命令未登记），没有零测试假通过。首版 7/8，observe-only 测试仍携带 managed 的 start／unload 能力，被原 registry 正确拒绝；仅修夹具为 observe-only 合法事实与不可选择档位，不放宽产品 validator。初始 fmt native 1 只涉及本人新增／修改的文件，限定 rustfmt；原件保留在 `.cursor/plans/debt-resource-policy-20261006-r0/`，后续实际门禁随结果登记，不预写全量或 CI。父债 Partial，交互向导／自动硬件建议／实时控制和资源父债未关闭。
- **局部实际出口**：CI 同组合的三个集成目标 native 0（31.3954166 秒：新草稿 8／原 preview 7／实际生成库消费者 1）；默认 CLI bin 98、新旧 feature 拒绝各 1，all-targets Clippy 的 default／all-features、fmt 与分层／两语种镜像／默认和五份改文链接／登记／旧路径／债结构均通过。回归通过后停止扩证，最终 self-review、编码／diff 与干净冻结后再一次完整本地链；本机忽略的回执不可冒充 Git 携带证据。原 capture fixture 从旧测试抽出共用，原七项断言未删；同一私有 preview 返回本次重算的 scheduling，不把旧捕获中的 ready 当有效意图。既有 CLI opt-in job 加新目标，default 测试与 job／矩阵／ci-gate 保持。
- **目标 SHA 的真实出口**：`8cd7d5d1da36e9393cb67acc98ae12e107560875` 一次完整本地链 native 0（743.8403995 秒）、一次推送，HEAD／工作树／临时 Python 环境精确保持。[正式 CI 37424944008](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37424944008) 的 npm-audit 被新披露 Vue/source-map-js 高危拒绝，本机完整扫描另有 Tinypool critical；故正式验收不成立，不把资源工具窄测或旧 CI 绿外推。原 job 日志保存在本机 `47-npm-audit-api-original.log`，定点补丁见下一条；没有对该确定性风险盲 rerun。此前预览 `9edfe719` 的一次 failed-only 补跑已实际终态 17/17 success，回执 `31-prior-ci-recovery-accepted.json` 与原中断日志都保留。

### DCL-20261006-48 · 新披露 npm 风险与测试链修复

- **原因 / 例子 / 停止线**：正式 npm-audit 原始日志 high 与本机完整 JSON critical 形成明确施工依据。例如依赖旧 SSR 的属性名校验或测试 worker 的原型污染链，即使不是当前资源工具造成，也会使基线审计拒绝；不开展产品攻击面穷尽证明。遵守[本批计划](ROUND-02-PLAN.md#k-supply-12--2026-10-06-新公告与测试链修复)，只修锁中相关依赖及现有消费者，原 CI 审计策略不改。
- **实施 / 自主取舍**：Vue/runtime/compiler/renderer 统一 3.5.43，source-map-js 1.2.2；Vitest 3 的 Tinypool 1.x 无兼容修复，选择最小已修复 4.1.11，三个 workspace 和根同版固定，未进入 5。原 Vue 3.5 / Vite 6 / Node 22 合同保持，无测试断言或产品代码改写，无 force／override／legacy-peer-deps。Vue 新补丁的同族及 Vitest 的实际传递边同步，锁差量只按这些可达边核对。
- **失败与工具纠偏**：原 npm 10.9.8 两条锁生成命令均抛 Arborist `edgesOut` null，锁未改变；保全日志后不继续原工具盲试，仅临时 npm exec 官方 11.21.0 生成，原 npm 10 `npm ci` 随后 exit 0，完整 `npm ls --all` exit 0。无全局 npm、Node 或 CI 工具链改动。原回执 `.cursor/plans/debt-npm-advisories-20261006-r0/01–09-*`，忽略目录不构成 Git 携带证据；controller 实施与语义自查，不称独立审核。
- **当前证据 / 残余**：生产 JSON 0，full 4 low／1 moderate／0 high／0 critical，两条 high 门禁实际 exit 0；ESLint Markdown→KaTeX 与 postcss-selector-parser 的剩余命中仍登记于中英安全 SSOT，不称 full 0。三个现有 workspace 测试、lint／typecheck／build 与文档门禁按原断言验证，完整链与精确 SHA 正式 CI 待冻结后执行；K-SUPPLY-12 因新公告重开 Partial，本片不能先写 Done。
- **局部实际出口**：Vitest 4.1.11 下 shared 54 文件／280 passed、ChatPro 24／92、Theater 11／53 与两条 Theater 产品 smoke 全部 native 0，未改变生产代码、配置或任何断言。lint、typecheck、生产 build、默认 52 和五份改文链接、中英镜像／登记／旧路径／债结构均 native 0；npm ci 保留两条上游 deprecated 提示，不能称 stderr 空。达到兼容验证即停，冻结后一次完整本地链与一次修复推送，不在本片追低风险残余到零。
- **已发生的冻结出口**：`3eb1ca7cafabedd739075c6a506fa9ecfcb95425` 一次完整本地链 native 0（651.163911 秒），HEAD／干净主树与临时环境保持、一次修复推送；[正式 CI 37427668306](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37427668306) 已进入运行，尚待终态，不提前标验收。旧 `37424944008` 的 npm-audit 已确定 failure、余下两项 Rust 仍占用时先普通取消、未退出再强制取消（两命令 native 0），实际终态 cancelled，原 audit failure 与已消耗的 runner 时间不抹去；不是重跑或取消可通过的批次。本机原始证据在 `.cursor/plans/debt-npm-advisories-20261006-r0/30-full-local.result.json` 与 `31–35-*`，忽略目录不随 Git 转让。后续交互工具在独立工作树开发，避免改动本次受测源。

### DCL-20261006-49 · CLI 有限资源策略交互向导

- **原因 / 例子 / 有限闭环**：按[既有有限计划](ROUND-02-PLAN.md#d-scaffold-resource-01--有限交互策略编辑2026-10-06)，将已能生成资源草稿的 CLI 接上 `--interactive`，与旧 patch 文件严格互斥。例如开发者从捕获中选择 primary、共存／互斥组与 fallback 档位，查看候选和有效保留量，再输入 yes 生成新 profile；不必手工写六类约束。四策略和六类约束直接序列化已有 canonical enum，仍由原 Host loader／registry／compiler 校验；未增加第二套资源语义、硬件建议或控制权。
- **编辑与权限边界**：空行保留，primary 的 `-` 删除该 TOML 键、commands 的 `-` 写空数组，`+` 整组替换；未提问的 TTL／抢占及非资源值保持。已有输出、取消／EOF、未确认、菜单错误或非法／冲突意图均非零且不留下最终草稿，同一临时校验与 `persist_noclobber` 只发布新文件。有效但降级的候选仍如实列原因码，不冒充资源准入；default feature 保持明确拒绝。
- **真实失败 / 自主修补**：写测试后，已有 CLI 可执行文件对新标志实际 native 2 拒绝，原件 `01-existing-cli-red.*`；该现有二进制不是本轮重新编译的精确 SHA，因此不称 Cargo 红测。首次新源码测试 native 101（14 passed／1 failed）：本人把“清除 primary”写成空字符串，原 validator 正确以 `resource_scheduling_adapter_id_invalid` 拒绝。修为结构删除键后，原非交互空字符串规则不变；保留 `02-host-command-regressions.*`，新 `03-*` 实际 15＋7＋1 passed，未弱化校验或断言。日志均在本机独立工作树 `.cursor/plans/debt-resource-wizard-20261006-r0/`，不是 Git 可移交原件。
- **局部出口 / 复核 / 停止线**：default CLI 98＋草稿拒绝 2＋preview 拒绝 1，default/all-features 的 CLI all-targets Clippy、最终 fmt、分层及默认／五份改文链接、镜像／登记／旧路径／债结构／编码均 native 0。首次 fmt native 1 仅要求新模块登记排序，限定修正、原日志 `15-*` 保留。controller 实施及语义自查，不称独立 reviewer。精确九路径只含 CLI 私有入口／测试、中英指南和原台账／计划／DCL；原 CI 已覆盖该目标，无 Host／types／contracts／依赖／公共 Rust API／发行版改动。随后冻结批次取得完整本地链与目标正式 CI，不拿工作树窄测当 main 验收。达到有限交互即停止调查；父债 Partial，自动硬件建议、实际控制及资源父债剩余范围不转绿，真实模型／设备／用户库／旧 CP-INT 身份零使用。
- **隔离运行准备**：工作树原 npm 对同一锁 `npm ci` native 0，未改声明／锁；准备共享构建缓存的脚本因目标已有普通目录 native 1 拒绝覆盖，零删除／移动／Junction 创建，原件 `17-*` 保留。改用原工作树 Cargo 配置的现有外部目录，不导出全局覆盖或回写配置；冻结后在此 cwd 跑一次完整本地链，原主树继续保持供应链 SHA。不同 cwd 与“完整链尚未执行”明确分列，不假装这项准备失败是产品测试失败。

### DCL-20261006-50 · selector parser 单叶补丁预备片

- **原因 / 例子 / 调查止点**：按[有限预备计划](ROUND-02-PLAN.md#k-supply-12--单一-selector-parser-补丁预备片2026-10-06)，原 full audit 的唯一 moderate 指向 ESLint Vue 消费的 `postcss-selector-parser 7.1.5`；[原公告](https://github.com/advisories/GHSA-rj75-hqrm-r3gf)与[7.1.6 发行](https://github.com/postcss/postcss-selector-parser/releases/tag/7.1.6)足够确定同系列补丁。例如 lint 处理选择器时保留原规则/API，换为上游线性解析实现；不调查全仓攻击路径或构造耗尽攻击样本。低级别 KaTeX 链仍单列，不盲信 audit 对 ESLint 配置的降级建议。
- **真实修改 / 边界**：在独立工作树从 `10af367d` 定向 lock update，结构差量只有一个节点的 version/resolved/integrity，**7.1.5 → 7.1.6**；package 声明、peer、CI、测试断言及产品源码零改动。沿已记录的锁生成工具安排仅临时 npm 11.21.0，原 npm 10 实际 `npm ci --ignore-scripts --no-audit --no-fund` exit 0（708 packages，两条 deprecated 提示保留），`npm ls --all` exit 0。无需重试原 npm 10 的已知 Arborist 缺陷，不升级全局工具。
- **当前证据 / 后续坐标**：production 0，full **4 low / 0 moderate / 0 high / 0 critical**，两条 high 审计门禁 native 0；Vue lint、typecheck、shared 54 文件／280 passed、production build native 0。适用镜像／登记／债结构通过；首条编码命令因本人漏 `--file` native 1，只打印 usage，原件保留，修正调用参数后五份改文实际编码 native 0。构建 bridge 仅换行变化，归档原字节、核相同 Git blob 后刷新 index，无语义 diff。最终链接／旧路径／diff 与写集核验随本地冻结回执，不冒充完整/远端验收。原件 `.cursor/plans/debt-postcss-parser-20261006-r0/01–19-*`，忽略目录不随 Git 转让。controller 实施与自查，非独立 reviewer。此小片先干净本地保留，后续相关实质片合批再完整链/推送/目标 SHA CI；当前不推 main 触发第三轮全 CI，K-SUPPLY-12 父债仍 Partial。
- **前片正式终态补记**：供应链 `3eb1ca7cafabedd739075c6a506fa9ecfcb95425` 已完成[37427668306](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37427668306) success、17/17 含 ci-gate；原完整本地 native 0（651.163911 秒），真实 closeout `debt-npm-advisories-20261006-r0/43-closeout.json`。资源向导 `10af367d1ac348b11cb8f6c25620196fcb63f70a` 本地完整链 native 0（2019.8064673 秒），[正式 37434691111](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37434691111) 精确 SHA success、17/17 含 ci-gate；原新工作树缺 Tauri 资源的第一轮 101 保留，按原 CI bundle 准备后同 SHA通过，未改源码/断言。watch 因 API EOF native 1 是读取失败，重新读取同 run 终态，没有 workflow rerun；原完整与终态收口在 `debt-resource-wizard-20261006-r0/34-full-local-attempt2.result.json` 与 `42-closeout.json`。这两片均为有限验收、父债仍 Partial；此处随实质补丁回填，未造 green-only 提交，不从前片借绿给本片。

### DCL-20261007-51 · 兼容 ChaCha20 补丁与供应链合批

- **原因 / 归属 / 限度**：controller；兼容依赖维护。主线 `10af367d` 已验，预备片 `fa579137` 保存 selector parser 7.1.5→7.1.6。本轮处理安全 SSOT 原已登记的 `chacha20 0.10.1` yanked：上游提供同线 0.10.2、MSRV 同为 1.85，修复 SSE2 后端误用 SSE4.1 指令。警告归中英安全滚动表，不把 npm K-SUPPLY-12 或 event-listener K-SUPPLY-11 的定义扩为 Rust 通用债；K-SUPPLY-12 仅登记关联合批坐标并保持 Partial。L1 得到可施工版本和约束即停，无攻击面、CPU 故障复现或无穷跨平台证明。
- **真实差量 / 自主选择**：使用原 `cargo update -p chacha20@0.10.1 --precise 0.10.2`，native 0；Cargo.lock 唯一 package 的 version/checksum 两字段变化，依赖数组、所有 manifest、源码、断言、feature、TLS、CI 和审计策略不改。完整 bot npm 16 项 / Cargo 7 项升级不直接合并；低级 KaTeX 的修复版本超出当前 micromark 声明，不强行 override/降级 ESLint。两片同为已知供应链兼容维护，合成一个里程碑门禁；这是有限工程选择，不是新的产品策略。
- **审计 / 口径**：前置 `cargo audit --json` native 0，DB 1290 / lock 698，漏洞 0、警告 7；stderr 有部分 registry yanked 查询超时，不据 exit 0 宣称查询完整。更新后 `cargo audit --no-fetch --stale --json` native 0、stderr 空、漏洞 0、警告 6（五 unic + glib），同轮 fetched DB，ignore 数组逐字不变。crates.io 元数据另核 0.10.2 未撤回及 checksum。上一 npm production 0 / full 4 low、shared 54 文件 280 passed 等原窄测保留，Cargo 补丁不借其作 Rust 回归证明。
- **验证 / 收口坐标**：controller 自查，independent=false；精确锁差量、第一至四列台账、六路径写集、既有中英镜像/默认及改文链接/登记/旧路径/编码/债结构/diff 实测后冻结干净本地提交。冻结批次一次完整 `check:ci-local`，按既有流程先准备新工作树的 Tauri 资源，随后合流/一次推送/精确 SHA 正式 CI；未取得该证据前只 Locally verified，不称主线或父债结案。证据 `.cursor/plans/debt-chacha20-patch-20261007-r0/`（忽略目录不随 Git 转让）；旧 selector parser 24-local-freeze 与此前源码/账本原字节保留。准备阶段一次 apply_patch 用了截断上下文而拒绝、零改写，改用 .NET UTF-8 append；不归因为测试失败。未启真实模型/TTS/用户库、未复用旧场景身份，未执行暂停的签名/TLS/产品决策。

### DCL-20261007-52 · 资源原因码的 CLI 展示闭环

- **原因 / 闭环 / 例子**：按[有限计划](ROUND-02-PLAN.md#d-scaffold-resource-01--原因码的人类说明预备片2026-10-07)，原 doctor、草稿诊断/拒绝和交互确认仅输出原因码；本片为十八个常见容量、可用性和约束码添加简短英文说明，共用 CLI 私有 formatter。例如缺控制器仍为 Degraded，捕获 GPU 余量不足仍为 Blocked，只帮助开发者理解；未知未来码原样保留、明确暂无内置说明。组冲突措辞按原 producer 收窄为“至少两个适配器同时被要求共存与互斥”，不误称组集合必须完全相等。达到展示闭环即停，不调查全部资源反例，也不生成自动硬件建议。
- **真实红/绿测**：精确新源码的首次 Cargo 红测 native 101，草稿 14 passed/2 failed 后 fail-fast，doctor 未运行；另行 doctor 红测 native 101、6 passed/2 failed，两份原件 `02-cli-red.*`/`03-doctor-red.*` 保留。仅增加说明后，真实 CLI 原回归及新增断言实际草稿 16/16、doctor 8/8，含人类预览前后 canonical JSON 完全相等；未知码私有展示 1 passed，default 拒绝 2＋1 passed。default/all-features CLI all-targets Clippy、最终 fmt native 0，没有削弱原校验或负例。
- **差量 / 权限 / 自查**：精确十路径只含三份 CLI 私有生产入口、两份原集成测试、中英指南和原台账/计划/DCL；Host loader/registry/compiler、公开 DTO/trait、依赖、JSON、退出码、候选排序、临时文件校验与 create-new 发布逻辑保持。controller 实施与语义自查、independent=false；适用文档/债结构/分层门禁须通过才冻结本地提交。说明不赋予控制权、实时准入或执行许可；无真实硬件/模型/TTS/用户库或旧 CP-INT 身份。本机原件 `.cursor/plans/debt-resource-reasons-20261007-r0/` 属忽略目录，不随 Git 转让。
- **基线与验收分列**：从供应链冻结 `67baa952bb7e1ff0ba987609d6fe729ca327f24d` 隔离开发；该供应链完整本地链 native 0（1916.445647 秒）并已推送，但正式 [37505631986](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37505631986) 的 npm-audit 实际因 shell-quote 新收录注入公告报 4 low/2 critical，不能标全绿。concurrently 9.2.4 精确锁定 1.9.0，而修复为 1.11.0；局部 override 例外等待维护者裁定，不在本片暗改依赖。此处只本地预备，不合流受测主线、不触发第二轮全 CI，也不借前片结果关闭资源父债。首轮完整执行会话丢失、无 native verdict 与第一次推送 TLS 握手失败原件保留；持续落盘的新完整执行通过、确认远端未变后同 SHA 第二次推送成功，无源码改写和 workflow rerun。

### DCL-20261007-53 · shell-quote 临时兼容补丁与撤销条件

- **原因 / 已确认决定 / 范围**：controller。前批 `67baa952bb7e1ff0ba987609d6fe729ca327f24d` 完整本地链 native 0（1916.445647 秒），主线 FF 后推送先因 TLS 握手 native 128、确认远端未变后同 SHA 第二次成功。[正式 CI 37505631986](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37505631986) 的 npm-audit 命中新登记 [GHSA-pqg4-j6r4-53mv](https://github.com/advisories/GHSA-pqg4-j6r4-53mv)，最终取消；16 个实际 jobs 的终态为 13 success、1 audit failure、2 Rust cancelled，不能算 ci-gate 通过。原 `concurrently 9.2.4 → shell-quote 1.9.0` 与已验 `10af367d` 相同，不归因前片新引入。维护者明确确认“记录临时补丁，等上游更新后更新依赖链”；本次只实施该单边例外，按[有限计划](ROUND-02-PLAN.md#k-supply-12--shell-quote-临时兼容例外2026-10-07)收口，不重复询问。
- **实际修改 / 自主选择**：根 manifest 添加 `overrides.concurrently.shell-quote = "1.11.0"`，原四条 override、concurrently ^9.2.1 声明及解析 9.2.4 保持；lock 恰好一个节点的 version/resolved/integrity 变化。上游当前 9.2.4/10.0.5 均仍精确依赖 1.9.0，不能仅升级父包或降级隐藏命中。沿已登记工具安排临时官方 npm 11.21.0 生成锁（native 0），随后原 npm 10 `npm ci --ignore-scripts --no-audit --no-fund` 与 `npm ls --all` native 0；708 packages、两条 deprecated 提示保留，不称 stderr 空。无产品源码、测试断言、全局工具链、CI 阈值、Cargo.lock/TLS/权限变动。
- **能识别原问题的有限验证**：前置 production 0／full 4 low + 2 critical，后置 production 0／full 4 low、两条 high 门禁 native 0；critical 两包是同一公告传播计数。内存中四种行分隔符在 comment 后旧版允许、新版 TypeError 拒绝，攻击字符串不送 shell；代表性良性 quote/parse、concurrently 生产参数展开保持。实际良性双子进程成功 CLI 0，子进程 7 时 CLI 1，原 production build 亦 native 0。typecheck 与 shared 54 文件／280 passed native 0；lint 亦 native 0；中英镜像、默认 52／改文 5 份链接、登记、docs-only 旧路径、147 行台账／12 auto plans 结构和编码门禁 native 0。到修复与兼容性足以判断即停，不追 KaTeX low 清零或穷尽产品攻击路径。
- **撤销责任 / 接手动作**：项目依赖维护者。等待上游支持线将这条边采用安全 shell-quote 后，更新 concurrently 和父依赖链，移除本条 override，再核原 npm ci、完整依赖树、production/full audit、实际构建及失败退出。若上游仍固定不安全版则例外保留；不因发布了新版本就自动撤销，也不把本次批准扩到其它 override。
- **证据 / 收口状态 / 边界**：controller 实施与专项语义自查，independent=false。精确七路径及台账所有前四列核对，镜像／链接／登记／旧路径／债结构／编码／diff 通过后冻结本地；一次完整本地链成功后才 FF/push 并取得新 SHA 正式 CI，当前不预写正式通过或父债 Done。原件 `.cursor/plans/debt-shell-quote-patch-20261007-r0/`；前批 `31-full-local-recovery.result.json` / `42-ci-failed-job-raw.*` / `45-final-run-state.log` / `46-unaccepted-closeout.json` 仍在旧供应链工作树逐字保留，忽略目录不随 Git 转让。资源原因说明 `e1e12da6` 独立本地片继续隔离、未 main／push／正式 CI（其 DCL-52 尚不在本主线），不混入本修复。一次 apply_patch 误用英文标题上下文被原子拒绝、零改写；只纠正现有锚点后重做，不改检查规则或产品语义。无真实模型/TTS/用户库/旧场景重放。
- **局部总审纠偏**：本人临时结构核验脚本误把历史三列表也当五列台账，首次 native 1；限定为现行五列并额外断言整个台账除 K-SUPPLY-12 单行外逐字不变，修正核验 native 0，原失败 14 与后续 15 回执分存。production build 的 bridge 只有 LF/CRLF 差异，保全 raw 文件并核同 Git blob 后仅刷新 index，零语义差量。以上为控制端核验与生成换行处理，不是产品修补或门禁放宽。

### DCL-20261007-54 · 资源原因说明合流与已验基线确认

- **原因 / 基线 / 已确认方向**：维护者要求确认基线后继续小批次债收敛。本轮从 main / tracking / 远端干净一致的 `1e81033d6c8efcff8bb9f502c642560e57335cf1` 建新工作树；该补丁一次完整本地 native 0（1491.7835099 秒）、[正式 37513747676](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37513747676) attempt 1、17/17 success 含 ci-gate，无 rerun，原 `38-closeout.json` 保留。当前实质 CLI 合流同时在中英安全记录及台账回填这个终态；不是只回写绿灯的提交，旧失败 37505631986 仍保留 cancelled/audit failure。
- **实际工作 / 例子 / 范围**：合入原干净本地 `e1e12da6d76f59cc1f3989ab675a67db2b4d6e90` 五份 CLI 源码/测试，不改原实现。例如 `resource_plan_controller_unavailable` 继续表示没有实际控制器并保持 Degraded，只附简短说明；JSON 计划不会带展示文本。十八个常见码、未知码不伪造含义，doctor/草稿诊断和拒绝/确认共享 formatter；中英指南同步。原 compiler/registry、校验、状态、排序、create-new、退出码、公开 API、依赖、权限与 CI 均不变。十二路径写集见[合流计划](ROUND-02-PLAN.md#d-scaffold-resource-01--原因说明合流2026-10-07)，父债仍 Partial，自动硬件建议不自动开工。
- **已测 / 待验分列**：新 cwd 原 Host 模式 CLI 集成实际 16＋8 passed、未知码 1 passed、default feature 拒绝 2＋1 passed、原 npm ci native 0；default/all-features CLI Clippy、workspace fmt、分层、中英镜像、登记、147 行台账/12 auto plans 结构、默认 52/改文 7 份链接、旧路径和七份编码均 native 0，精确写集与台账完整文本差量核对通过。只借旧片原红测解释缺口，不借旧 SHA 的绿灯作为本批 main 验收；源码冻结后完整本地链成功才 FF/push，目标正式 CI 仍须取得实际终态，当前不预写 Done。足够支撑展示闭环即停止扩证，不新增真实设备/模型/TTS、用户库、历史 CP-INT 身份或全部硬件故障矩阵。
- **自主合流与纠偏 / 原件**：只有 DCL 和 ROUND 的尾部追加冲突，首次 cherry-pick --no-commit native 1 与两份冲突原字节存档；核双方段落后保留 DCL-52/53 与原计划，显式暂存解决、quit sequencer，不改原冻结 e1 工作树。一次台账定位脚本未允许 CRLF 的 CR，唯一性守卫在写入之前拒绝 native 1；允许 CR 后更新成功，随后还须核整个台账除两个第五列外逐字一致。这是控制端文档定位问题，不是产品测试失败；没有通过改断言或放宽门禁解决。
- **责任 / 证据 / 接手**：controller 实施与最终只读语义自查，independent=false；原预备红/绿原件保留在旧工作树，本轮原件位于 `.cursor/plans/debt-resource-reasons-closeout-20261007-r0/`，被忽略目录不随 Git 转让，tracked 源码/测试/文档可携带。前片 npm 父债 Partial，临时 override 仍待依赖维护者在上游支持线安全后更新链并撤销；本片不重问既有决定，不解冻签名/TLS/EventStream/Agent 产品取舍。

### DCL-20261007-55 · Remote Prompt 既有入口的增量收束

- **原因 / 授权 / 基线**：维护者明确选择逐片复用已有 Remote helper、保留行为，要求整理而非全盘定死。起点 main/tracking/实际远端干净一致 `a679937943b8c079c686f5714ec3aeaa35f08def`；该资源片完整本地 native 0（1310.5125887 秒）、[37526132398](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37526132398) attempt 1、17/17 success 含 ci-gate。随本次实质源码整理只回填资源行第五列真实终态，不改 DCL-54 历史正文、不追加仅为绿灯的提交。Remote 父债保持 Partial，旧 Minimal long-plan 已 Closed 的事实保留，新授权不冒领旧 Stage 或宣称 Full 完成。
- **实际修改 / 例子**：`top_topic_hint` 共用 blocking adapter 的 `call_with_builtin_fallback`，保留 Option 外观；Prompt 移除第二份 AtomicBool 引用，本地 serialization/bad-shape 分支经模块内 `fallback_allowed` 读取同一原开关。比如 HTTP 503 时开关开仍返回 builtin topic、关闭仍 None；HTTP 200 但 `hint:42` 仍 None，不因入口统一而伪造 hint。高风险拒绝仍先于请求与 builtin。原 params、超时、日志、错误及单请求无 retry 保持，未新增公开 API/DTO/依赖/状态应用/统一策略框架。Memory 的空结果处理不同，保留其原实现，留作未来有限片而不顺手扩面。
- **已测 / 待验**：四项生产 HTTP 表征实际先在 a679 未改生产源码 native 0，再在 topic hint 与 gate 两片后各 native 0；包含有效/空 hint、坏形状、运行时开关切换、权限拒绝零请求和准确请求次数。既有 Remote 回归、Clippy/fmt 与适用分层/中英镜像/链接/登记/旧路径/编码/债结构/diff 的原生命令按独立编号保存；源码和文档冻结后一次完整本地链，实际通过才 FF/push 与目标正式 CI，当前不预写父债 Done。足够判断保持行为即停，不寻找所有 timeout/取消/异常组合，不借旧基线 CI 作为本片结果。
- **自主纠偏 / 文档支撑**：fresh worktree 的第一次构建缺 Tauri kernel resource（101）、补包后缺 frontendDist（101），均发生在测试运行之前；按现有构建规则补齐原 bundle/npm/build，再跑表征通过，生产源码当时未改。生成 bridge 原 raw 字节副本保全，Git blob 与 HEAD 一致才刷新 index，不覆写产品。文档修改一次行尾空格使 diff-check 2，修正后 0，原日志保留。中英协议把“失败必然回退”收窄为运行时闸门/各方法语义，明确授权拒绝与 decode 不自动回退，解决文档比代码承诺更宽的问题，不通过扩大代码语义来迁就文字。
- **范围 / 责任 / 接手**：八路径写集与停止线见[本批计划](ROUND-02-PLAN.md#k-resilience-01--remote-prompt-既有入口增量收束2026-10-07)。台账恰为资源/Remote 两行第五列修改，其余文本逐字相同；不改队列、Closed Minimal、签名/依赖等其他架构决策。controller 实施和只读语义自查，independent=false；原件在 `.cursor/plans/debt-remote-prompt-consolidation-20261007-r0/`，tracked 测试/实现可携带，ignored 回执不随 Git 转让。本批仅合成 loopback/grant，零真实模型/TTS/用户 DB 或旧业务身份；后续另按有限单片收敛。


### DCL-20261007-56 · Remote Memory 闸门有限收束与 Prompt 终态回填

- **原因 / 基线 / 上批终态**：维护者已选择增量复用 Remote 既有入口并保留各方法行为。本片从 main / tracking / 实际远端干净一致的 `f4ee767bb92c6e7d0ccb6b6770ee41e1d463373a` 建独立工作树；原 Prompt 批完整本地 native 0（2348.8380943 秒）及 [37579810413](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37579810413) attempt 1、17/17 success 含 ci-gate，八份冻结源字节不漂移、无 rerun。本次实质 Memory 修改一并回填其终态，DCL-55 与旧计划正文保持原字节，不造只写绿灯的提交。
- **修改 / 例子 / 界限**：Memory 只去掉重复 AtomicBool 引用，通过已有 adapter 的模块内 `fallback_allowed` 在原位置读取原开关。保留 `call_plugin_soft`、排序/补尾/limit、方法错误全文、日志、授权、请求/超时/无 retry 与本地 context/search；原 HTTP 503/坏形状在开关开时仍 builtin、关时仍原错误；成功的空 `ordered_ids` 对非空输入仍补原输入，真实空输入在关回退时仍报错。协议登记这个具体分支，英文 §6 残留的“未知形状必回退”按中文和代码收窄；不是扩大实现去满足旧描述。
- **实际验证 / 停止线**：原生产 Memory 源 Git blob 与 HEAD 相同后，复用既有侧车新增四项 Memory 表征，连同原四项 Prompt native 0；去重复字段后用完全相同测试字节再 native 0。检查远端顺序、未知 ID、空 ID 数组、完整 Memory 值、503/坏形状、live 开关、真实空输入、拒绝零请求与准确请求体/次数。既有 Remote 回归及两项 Clippy/fmt 通过，适用分层/镜像/链接/登记/旧路径/编码/债结构/diff 留独立 native 回执；这些证据足以判断本次保持行为，不再追加组合矩阵。
- **节奏 / 风险 / 自主纠偏**：本片为 M、本地开发提交；不立即推第二轮全量 CI，后续有限批冻结时再一次完整/正式验收。父债仍 Partial，Full OPEN；controller 实施与只读自查，independent=false。fresh worktree 按既有 npm/bundle/frontend 构建规则先补前置，生成 bridge 原 raw 保全并证 Git blob 相同才刷新 index。一条测试命令误填 package 为 `oclive`，native 101 在测试启动前拒绝；按 Cargo.toml 的实际 `oclivenewnew-tauri` 修正并以新日志编号通过，旧失败不覆盖。原件位于 `.cursor/plans/debt-remote-memory-consolidation-20261007-r0/`（ignored，不随 Git 携带）。
- **接手 / 未改**：七路径写集与下一冻结线见[有限计划](ROUND-02-PLAN.md#k-resilience-01--remote-memory-闸门的有限收束2026-10-07)。台账只改该父债第五列，其他正文相同；旧 Minimal/队列、签名架构、TLS/reqwest、EventStream、Agent 产品范围及硬件建议不变。零真实模型/TTS/用户 DB/历史 run ID，仅独立 TempDir grant 与 loopback。原 Prompt 已验工作树/main 不承载本片未验字节。

### DCL-20261007-57 · 双核 Beta 维护冻结与已验 Remote 基线接续

- **决定 / 原因 / 例子**：维护者选择“只冻结，指明方向和记录文档，继续”。暂停 v3 双核 Beta / 配套 expert_routing 的继续维护与自动扩证；默认仍关，P2–P5 已实现历史、显式 opt-in 入口和源码保留。例如实验前 emotion 为 NULL、实验写入后失败，现有有限补偿不能恢复 NULL；本次保留问题，不把 Frozen 记为 Done，也不要求 Stable 先等 Beta 修好。
- **真实差量 / 归属**：原 [DUAL-CORE-FREEZE](long-plans/DUAL-CORE-FREEZE.md) 从 skip 提示补为可接手的冻结规则，写参考提交、范围、已知缺口、两条既有未来方向及解冻条件；双核交接和中英指南只加短提示与链接。主台账 K-DUAL-ROLLBACK-02 第五列及 §2 冻结行改维护冻结，当前依赖行同步；其余历史阶段/RFC、源码、配置、feature、队列 skip、依赖与测试不改。若未来选择 nullable restore，先区分读取错误与真正缺值，有限首片覆盖 NULL→实验写入→失败→Stable 前补偿，不扩为通用事务框架；此处没有替维护者选实现路线。
- **已验基线 / 回填**：起点 `90b9cf4d3f9a9a17744323d1177f138c90c686ea` 的 Remote Prompt / Memory 既有入口收束已正式验收：完整本地 native 0、1355.6969896 秒；[37607847085](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37607847085) attempt 1、17/17 success 含 ci-gate；七路径冻结字节无漂移，main / tracking / 实际远端干净一致，没有 workflow rerun。原 `40-closeout.json` / `MILESTONE-20261007.md` 保留在旧工作树的 `.cursor/plans/debt-remote-memory-consolidation-20261007-r0/`；本次随实质冻结决定回填父债第五列，仍 Partial / Full OPEN，不改 DCL-56 历史正文或只造绿灯提交。这是旧 SHA 的终态，不代替本次文档检查。
- **实施 / 适用门禁 / 止线**：按[七路径有限计划](ROUND-02-PLAN.md#dual-core-freeze--暂停-beta-维护并保留解冻方向2026-10-07)，controller 实施与直接自查、independent=false。本片只做默认及改文链接、docs-only 旧路径、登记、镜像、债结构、编码与 diff，以及精确写集/历史保留核验；原始回执在 `.cursor/plans/debt-dual-freeze-20261007-r0/`（ignored，不随 Git 转让）。本地干净 checkpoint 随下次实质批次合流；不为冻结文档重新跑全量 CI，不跑双核场景、不分配业务身份、不启模型/TTS/用户库。冻结规则清楚即停，其他债务按原准入与调查限度继续。
- **本地实际结果**：八项适用文档检查 native 0（默认 52 / 改文 7 份链接、旧路径、登记、镜像、债结构、编码、diff），另有精确写集与保留核验 native 0。七路径仅 Markdown；台账两条五列行只改第五列，§2 冻结行只改状态列，其余文本相同；DCL 历史正文除已声明可变的依赖行外保留，指南/交接实现正文及队列 skip 不变。diff stderr 有 Git LF→CRLF 提示，不称零 stderr；这组结果仅验本片文档冻结，不替代运行时验收。

### DCL-20261007-58 · CLI Rust Action 的五处重复收束为一个步骤

- **原因 / 例子 / 已确认基线**：维护者要求确认基线后继续偿债；main / tracking / 实际远端干净一致 `90b9cf4d`，原完整/正式终态在 DCL-57。本片从干净本地冻结 `3865f5c8` 接续，保留双核 Frozen / Deferred。原 CLI 每次 Rust Action 更新需要人工改五处相同步骤，例如只更新 audit、漏 build 会被已有合同拒绝；现将五个消费位置共用一个私有完整 step，减少漏改入口，生成内容保持。达到原因与改法明确即施工，不把维护片扩为 CI 架构重设计或研究全部升级组合。
- **闭环 / 范围 / 验证**：精确七路径见[有限计划](ROUND-02-PLAN.md#k-supply-10--cli-rust-action-步骤的单点维护2026-10-07)；生产生成器、真实 init/check、原同步正负合同到中英维护指南闭环。原测试正文未改，整理前后 5/5；两类真实 YAML bytes/hash 一致，正常 check 0、独立坏 pin check 1。CLI 全套 20 套 / 150 passed、native 0；完整结果及控制端偏差见[原 Actions Wave 本节](waves/WAVE-20260929-ACTIONS-PINS.md#2026-10-07--cli-rust-action-步骤单点维护)，不复制测试矩阵。实际 `.github`、Action SHA、权限、策略、依赖、公开 API、默认功能和运行时均未改；无真实模型/音频/用户 DB/旧业务身份，仅当前忽略目录内合成项目。
- **执行 / 责任 / 结论**：M 私有整理；controller 实施与只读语义自查，independent=false。applicable CLI Clippy/fmt、分层、文档/镜像/编码/登记/债结构/diff 与精确范围核对须通过才建干净本地提交；本片不单独推远端全量矩阵，相关冻结批次结束后一次完整/正式出口。K-SUPPLY-10 保持 Partial、维护仍人工，不能由单点常量称全部维护自动化；旧 Wave、台账其它行和已冻结双核材料保留，父债第五列只登记真实局部进展。
- **已测出口**：default / all-features CLI all-targets Clippy、fmt、原分层门禁，默认及六份改文链接、旧路径、登记、中英镜像、147 行台账 / 12 auto plans 结构、编码与 diff 均 native 0。原五个同步测试正文逐字保持；两份真实生成字节比较是本次保持行为的停止线，不再为此增加组合测试。CLI native 0 / wrapper 恢复检查 1 分列保留，存在/缺失态控制端复核已通过；失败不归因产品、无测试重跑或规则放宽。

### DCL-20261007-59 · 供应链指南现状引用化与有限维护里程碑

- **原因 / 例子 / 依据**：阅读实际修改的供应链指南时发现两处当前摘要与唯一台账冲突：K-SUPPLY-12 仍称八月全部 audit 0 / Done，而现在有已批准的临时 override 与待撤销条件；插件 HTML 隔离仍被描述为旧共享 origin，而当前已用 opaque iframe + broker，Stage 4 签名架构仍暂缓。只核对应权威行、原安全记录、Stage 3 Wave 与现有 `directoryShellBootstrap.ts` / `pluginFrameBridge.ts` 及 package override，足够确认文案过期即施工；没有全表扫描、audit 新测、实机扩跑或架构变更。
- **范围 / 历史保留**：按[五路径有限计划](ROUND-02-PLAN.md#d-debt-ledger-01--供应链指南的两处过期现状对账2026-10-07)，中英指南两行改为当前范围短述和权威链接；原八月 SHA / CI 与修订历史保留并标作历史，台账 §2 前瞻短段区分历史与现状。D-DEBT-LEDGER-01 仅第五列追加局部进展，其余权威状态不改；不维护第二份当前状态或复制扫描总数，不从结构门禁推断产品事实。
- **责任 / 验证 / CI 节奏**：controller 实施与直接自查，independent=false；适用文档门禁和精确写集 / 历史保留核对后保存本地 checkpoint。与 `3865f5c8` 双核冻结、`5729fe76` CLI 私有步骤整理合成一次有限维护里程碑，随后只跑一次完整本地链与目标 SHA 正式 CI；正式未取得前不称新的 main / remote 已验。双核 Frozen / Deferred，其余父债 Partial / OPEN 与已批准签名暂缓、真实音频 / 数据 / 硬件等前提不变。
- **文档已测出口**：五份改文链接、docs-only 旧路径、登记、镜像、147 权威行 / 12 auto plans 结构、编码与 diff 均 native 0；结构计数不是偿债完成数。历史段、其余现状行和产品源码保持。已有明确反例与正确责任入口，达到可接手即停，不通过新增证明性测试或重跑历史场景来扩大本次结论。

### DCL-20261008-60 · 英文直接情绪词沿既有规则补充

- **原因 / 前置 / 例子**：维护者授权持续偿债，架构取舍先登记跳过。旧自动队列无 runnable，不重开已关闭或冻结 Stage；controller 从主台账选择 K-EMO-01，K-EMO-02 的中文前置已有验收。原英文词表仅 35 个词，真实 `BuiltinUserEmotionAnalyzer` 对 `I feel DELIGHTED!` 返回 neutral 分数，Base 无线索。本片只补十二个直接词，不把有界词汇缺口变成情绪研究项目。基线 `bb229033230f53b7740873d6a421c7bbeba83680` 已完成一次完整本地 native 0（1770.019088 秒）及 [正式 CI 37644388741](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37644388741) attempt 1 / 17/17 success，前片原始失败/观察记录保全。
- **实施 / 权责 / 范围**：按[有限计划](ROUND-02-PLAN.md#k-emo-01--既有英文词表的直接情绪词补充2026-10-08)修改原 JSON 与公开消费回归，再同步本事件和唯一 K-EMO-01 行，共五路径；controller 实施及自查，independent=false。新增批 `en-direct-20261008` 是独立作者词条，每个非 neutral 维度两词，英文权重仍 2；旧 104 条及顺序保持，原边界/否定/归一化/首命中规则与 schema 不变。无外部词库导入或新评分算法，主 LLM 仍负责角色侧权威，Host、UI、Tauri、插件/角色包和双核实现无需改。
- **真实验证 / 状态**：先增加三项外部公开消费者回归；旧数据 native 101（1 passed / 2 failed），首因分别是 `DELIGHTED` 无 joy 线索与 `not delighted` 无可报告词条，原件保留。追加词条后原目标完整 10/10、新三项通过；runtime lib 273/273、runtime all-targets Clippy `-D warnings`、fmt 和分层 native 0。正例用大写与标点，负例覆盖 ASCII 标识符和直接否定；Base 的否定线索仍存在，七维不贡献情绪分数。文档/编码/债结构及最终写集核验后保存本地 checkpoint；K-EMO-01 从 OPEN 记为 Partial，目标 SHA 正式验收待批次出口，不先写 Done。
- **止点 / 原件 / 暂缓**：证据足以施工和判断这十二词后停止扩词或补组合；复杂主体/引述、非相邻否定、反讽、自然度与多轮/真实模型质量仍属各自范围，K-EMO-07 不变。签名、reqwest TLS、双核解冻、Agent 并集、Production Stream 等架构决策跳过，硬件/人工/跨仓项不借本片变绿。日志在本机隔离工作树 `.cursor/plans/debt-english-lexicon-20261008-r0/`，不随 Git 自动转让；开发窄测与本地 checkpoint 后仅在有限批次收口时做一次完整/正式验收，不为小片反复触发全量 CI。


### DCL-20261008-61 · 技术债接手现状的三处过期表述收束

- **原因 / 例子 / 真实依据**：主线和实际远端为已验 `6f11a54f`。接手台账一行同时写“HTTP/Tauri/UI 最小路径尚未接通”和“基础主流程与六槽共享消费已接”，容易让下一位 AI 重复已做接线；CLI 私有 Action 步骤与英文十二词补充的当前状态又仍为正式待验。有限核对 ROLE_PACK_BOUNDARY §0.11–0.19、注册的 `send_minimal_message` 与 ChatPro 绑定入口，以及两批原 closeout 件，足够区分当前事实与旧快照；未重测历史场景或调查所有文档。Remote 现有主路径的有限筛选没有找到新收束点，未为整理而改变 Agent 硬失败 / 工具调用语义。
- **改动 / 历史 / 责任**：按[四路径有限计划](ROUND-02-PLAN.md#d-debt-ledger-01--技术债接手现状的三处过期表述2026-10-08)更新最小角色问题 / 后续列，已有第五列承诺原样保留；K-SUPPLY-10 的长状态改为有限现状和原 Wave / 实施链接，K-EMO-01 同时分清已有直接词与未实施外部词库，并纠正待验阶段；本父债第五列追加进展。三个旧权威行已[逐字保全](../archive/TECHNICAL_DEBT_HANDOFF_ROWS_20261008.md)，旧 DCL / 阶段计划 / 测试 / 原件不回写。controller 实施与直接语义自查，independent=false；这项偿还的是接手语义漂移，不是产品架构变更。
- **既有正式终态 / 不外推**：CLI 步骤随 `bb229033230f53b7740873d6a421c7bbeba83680` 完整本地 native 0（1770.019088 秒）及 [37644388741](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37644388741) attempt 1 / 17/17 success 已收口；英文词表 `6f11a54fef02281f45aef4aeb2301b6b5226d10c` 完整本地 native 0（1324.0502818 秒）及 [37655656311](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37655656311) attempt 1 / 17/17 success 已收口。两份本机 closeout 原件各自保留；正式结果属于这些 SHA，不代表本次文档又跑过完整链。Partial / OPEN 及真实质量、完整生命周期、跨发行版验收边界保持，不以全量绿灯关闭父债。
- **有限验证 / 止点 / CI 节奏**：只做文档适用门禁与文本差量保全检查，结果在本机 ignored `.cursor/plans/debt-ledger-handoff-20261008-r0/`，不随 Git 自动转让；保存干净的本地文档 checkpoint，不为状态维护单独触发全量 CI。已核责任、正确描述与原证据后停止扩查；签名、TLS、Full Resilience、Agent 并集、Production Stream 和双核解冻等决策没有借本片推进，硬件、真实语音 / 模型与姊妹仓条件也未解除。
- **本片实际文档结果**：八项适用检查 native 0（默认 52 份 / 四改文链接、docs-only 旧路径、登记、镜像、债结构、编码、diff）；另有文本保全核验 native 0：归档三行相同、最小角色原第五列相同、其余台账文本相同、两份历史记录仅追加、精确四个 Markdown 路径。结构输出的 147 行 / 12 auto plans 不是已偿还数量，零 runnable 也不代表全部技术债已完成。diff stderr 的 LF→CRLF 提示保留，不称零 stderr。本次没有运行 Rust / 前端 / 全量 CI 或新增业务证据，只有本地文档 checkpoint；main 保持原已验基线。

### DCL-20261008-62 · 构建缓存的有限只读体检入口

- **原因 / 例子 / 范围**：维护者要求无需决策即持续施工；从干净本地 `5c605020` 选择已有 K-BUILD-07 的只读体检片，不重开旧物理分配研究。缓存以往只能读一次性人工盘点，例如“逻辑目录大”不能回答是否可删除；新入口给出可重复观察并显式保留 unknown。精确十一条关联写集与止点见[有限计划](ROUND-02-PLAN.md#k-build-07--有限只读缓存体检2026-10-08)，语义 / 命令 / 原件只在[构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md#只读缓存体检入口2026-10-08)维护，不再复制阶段矩阵。
- **实现 / 能力边界**：新增 metadata-only `inspect:build-cache` / `test:build-cache`，root 必须显式、条目与时间有预算，完整与不完整、逻辑阈值告警分别退出 0 / 2 / 1；按路径计字节，硬链接不去重、物理与可回收量 null、用途 / 保留未判。没有正文读取、删除 / 写 cache / 输出文件、Cargo 运行或默认扫盘；默认门禁只新增真实合成合同。中英贡献指南及 Cargo 的旧删除提示同步收窄，参数值原样保持。K-BUILD-07 OPEN，不把工具或 CI 绿灯当缓存维护策略已验；Kernel / Host / 小 Kernel 边界和其余冻结项不改。
- **实际开发证据 / 自主处理**：原工具不存在时测试 native 1（模块缺失），实现后原十项通过；针对访问失败与最后一次读之后的时间窗口再补两项必要负控，12/12 native 0。真实 CLI 正负退出 / TempDir / Junction / 硬链接 / 坏参数 / 原文件保持均由同一实现执行。一次真实 cache 显式调用 native 0、完整、约 9.29 秒；统计是当前非原子元数据快照，不计算清理收益、不重跑凑样本。一次 apply_patch 格式错误在任何文件写入前被工具拒绝，修正补丁格式后继续；没有把它算成产品失败，也没有覆盖原 native 红测。controller 实施和语义自查，independent=false。
- **批次出口 / 暂缓**：开发窄测与适用门禁之后，和上片 DCL-61 形成有限维护批次，冻结新 SHA 后只跑一次完整本地链，成功才 FF / push 并取精确 SHA 正式 CI；尚未取得前只称 Locally verified。旧二进制、CP-INT 记录 / run ID、原缓存文件、用户 DB / token / 模型 / 音频不动，无业务或外部网络调用。签名 / TLS / Full Resilience / Agent 合并 / Production Stream / 双核解冻等用户决策门继续保留；下一实施需要未决取舍时再问维护者。
- **本片实际窄测出口**：12/12 合同及两处脚本语法、默认 / 六改文链接、旧路径、镜像、登记、债结构、编码与 diff 均 native 0；精确十一条写集和文本保留核验 native 0。package 只有两个维护脚本、Cargo 全部非注释值及原 Dimension 5 步骤相同，主台账其它文本和历史正文保留。147 行 / 12 auto plans 只是结构输出，父债保持 OPEN。diff stderr 保留 LF→CRLF 提示，不称零 stderr；单次真实盘点不作为完整 CI 或可回收量证据。

### DCL-20261008-63 · 单 Agent 保持与多 Agent 合并暂缓

- **维护者决定 / 原因 / 例子**：维护者选择“保持单 Agent，暂缓合并”。例如蓝图写两个 Agent，当前只执行折叠后的一个；直接改成按序接管会改变任务 / 工具副作用语义。有限核对 `wrap_agent_if_merged`、`PluginResolver`、`AgentProvider::process` 与调用者提供的 `AgentInput.tools` 已足够判断不是机械整理，到这里停止扩查；工具并集仍未实现。
- **差量 / 责任 / 止线**：按[三路径计划](ROUND-02-PLAN.md#k-agent-merge-01--保持单-agent暂缓合并2026-10-08)只登记主台账第五列、本计划和本事件；controller 实施 / 自查，independent=false。原权限、单 Agent 行为、接口与配置保持；父债 Deferred / OPEN，解冻需明确恢复并冻结既有语义问题。历史、其它债务与缓存冻结批次不改，没有新场景、身份或模型 / 工具调用。
- **有限出口 / CI 节奏**：只做文档门禁与三路径 / 历史文本核验，回执在新隔离树 `.cursor/plans/debt-agent-defer-20261008-r0/`；通过后建本地 checkpoint，后续实质批次携带。不为优先级记录单独跑全量 CI，也不把上批尚未完成的结果冒充本片验收。
- **实际适用结果**：八项文档检查 native 0（默认 52 / 三改文链接、docs-only 旧路径、镜像、登记、债结构、编码与 diff），另有三路径 / 历史保留核验 native 0。147 行 / 12 auto plans 是结构输出，不是偿债完成数；其它台账文本与原四列相同，计划 / DCL 历史仅追加。编码门首次因 controller 写错脚本名而 MODULE_NOT_FOUND、native 1；使用真实 `check-doc-encoding.mjs` 的新回执通过，旧失败保留，不归因产品。diff 的 LF→CRLF 提示保留，不称零 stderr。

### DCL-20261008-64 · Tauri SDK override 的根声明引用化

- **原因 / 例子 / 实施**：根 SDK 的 dependency 和 override 两处重复同一版本范围；未来只改 dependency 会造成 npm EOVERRIDE。按[四路径有限计划](ROUND-02-PLAN.md#k-supply-12--tauri-sdk-override-引用根声明2026-10-08)，采用 npm 官方 `$name` 根声明引用，保留约束，仅去掉手工同步的第二个事实来源；不删除 override、不升级 SDK。controller 实施与语义自查，independent=false。
- **实际安装 / 停线**：原 npm 10.9.8 ci native 0，锁逐字节相同、完整 npm ls native 0 且依赖树相同；两树 SDK 仍 2.11.1、67 文件集合 / bytes / SHA256 全同。原消费者回归 6/6、typecheck 和 module-compat 已通过；达到保持行为证据后不再扩展调查或新增等价测试。生产 audit native 0 / 漏洞 0；完整 audit native 1 / 4 low、0 moderate / high / critical，原始非零回执保留，不能写成全图零风险。
- **边界 / 出口**：K-SUPPLY-12 仍 Partial，KaTeX low 和 shell-quote 临时兼容补丁的上游撤销条件保持；签名 / reqwest TLS / 多 Agent / 双核冻结不变。后续完成 lint / frontend build、适用文档门禁与精确写集 / 历史保全再保存本地 checkpoint，不触发新的全量矩阵。日志在本片 `.cursor/plans/debt-api-override-ref-20261008-r0/`，不随 Git 自动转让；不跑 Rust / 真实 Tauri / 模型 / 音频 / 用户数据或旧业务场景。一次写入前补丁因 controller 错用标题分隔符被工具拒绝，改用实际文件尾部后继续，没有源码或行为失败。
- **实际适用出口**：lint / typecheck / frontend build / module-compat native 0，原 IPC 消费回归 6/6；八项文档门禁 native 0，147 行 / 12 auto plans 仅为结构输出。构建把旧生成 bridge 的换行重写为 LF；保全原始输出，确认 HEAD / index / 过滤后 working blob 同一身份，刷新该路径 index 后无 staged 内容变化，没有源码回退。精确四路径、单字段引用差量、锁不变、其它台账与历史文本保持再核验后建本地 checkpoint；diff 的 LF→CRLF 提示保留，不称零 stderr。到此停止扩大测试或依赖研究，正式验收不从缓存批次或父 SHA 借用。

### DCL-20261008-65 · 保持 Minimal，并将 Full 韧性暂缓

- **决定 / 依据 / 唯一入口**：维护者确认现有 Minimal 保持，Full 为 Deferred，不是 Done 或永久取消。没有明确故障目标时不发明通用层；[决策 §8](../../creator-docs/architecture/DESIGN_DECISIONS.md#full-resilience-deferred)维护冻结范围、Kernel / Host 权责与复评条件。[原 Minimal 计划](long-plans/K-RESILIENCE-01.md#full-deferred-freeze)保留历史验收，明确其 OPEN 不是当前开工许可。复评不自动解冻，涉及新执行语义 / 架构权力仍须确认。
- **范围 / 责任 / 节奏**：按[六路径计划](ROUND-02-PLAN.md#k-resilience-01--保持-minimalfull-deferred2026-10-08)更新当前台账与上述入口，本计划 / 本事件仅追加；controller 实施和直接语义自查，independent=false。无新实现、重试、熔断、接管或降级；原 Agent / 工具 / 六槽和 Remote 行为保持。只跑适用文档与精确保全门禁，先 local checkpoint，后续有限批次携带，不单独全量。
- **随本次实质决定携带的已验前批**：缓存维护 `c9b08a96a0202c4ebfe61fea9ef4d030a1b147a2` 完整本地 native 0（1482.6169677 秒），[正式 37711541565](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37711541565) attempt 1 / 17/17 success 含 ci-gate；main / tracking / 实际远端同 SHA 且干净、12 冻结路径相符、无 rerun。原只读查询一次 EOF 的 native 1 回执保留，随后读取终态成功；不是测试失败或测试重跑。K-BUILD-07 仍 OPEN；这份终态不证明后续 SDK 或当前冻结文档 SHA 已正式验收。
- **下一片 / 止点**：既有无需架构决策项继续有限施工，先核 K-CROSS-01 unsupported 主路径与直接测试，不穷尽跨平台 / 真实音频。明确问题连续处理；收益下降或无施工点则停止该候选，换其它既有债。签名 / TLS / Agent / 双核冻结、真实模型 / TTS / 硬件与各姊妹仓前提保持。原件与门禁回执在本机 ignored `.cursor/plans/debt-resilience-freeze-20261008-r0/`，不随 Git 自动携带。
- **适用验证**：八项文档 / 登记 / 债结构 / 编码 / diff 检查 native 0；147 行 / 12 auto plans 是结构结果，不是债全部完成。追加出口后仅复验受影响的链接 / diff 和六路径 / 两行 / 历史 JSON 与正文保全。controller 总审不冒称独立审查；不运行 Rust / 模型 / 音频或新业务身份，当前冻结先保存本地 checkpoint。链接检查不核片段锚点，手工总审另修正两个新增 DCL 链接的多余连字符。

### DCL-20261008-66 · TTS 平台拒绝主路径有限修补

- **问题 / 行为 / 依据**：平台声明已把 Linux/macOS bundled CosyVoice 标为 unsupported；原解析器识别结果被 TTS record 丢掉 reason，probe/warm/speak 不核 ok。修后按该声明明确 `unsupported_platform`（未知 profile 为 `profile_not_found`），不再误报 warm skipped 成功或继续引擎发现/探测/启动。范围见[本片计划](ROUND-02-PLAN.md#k-cross-01--tts-unsupported-主路径修补2026-10-08)，契约在[插件 README](../../distros/chat-pro/plugins/com.oclive.voice.asr/README.md)；未新增 OS 支持或调整六槽。
- **有限实证 / 保持行为**：实际原 ES module 和 HTTP dispatcher 的同一回归，IO/平台为替身：原实现 native 1 / 8 passed / 9 failed，修后 native 0 / 17 passed。Linux/darwin 原 ASR 拒绝、Windows 有效引擎发现、local_http/cloud 无 bundled warm、空文本/关闭扩展优先级均保留；不支持的 TTS 探测/预热/发声与协调包装 engine fs/network/spawn 计数为零。默认 TTS 棘轮加入该定向回归，有界 15 秒，不为测试启动实际 socket/引擎。
- **边界 / 状态**：父债 K-CROSS-01 保持 Partial；替代 OS 值不证明 Linux/macOS 实机或 UI，Windows 的真实 ASR→chat→TTS、真实音频仍待相应条件。拒绝回合不伪造资源卸载确认，Host 原无确认保留租约规则与前置准入未改，不从该 Node 控制流外推资源回收。source profile、公共 API、权限与旧 Minimal 记录保持；Full / Agent / 双核 / 签名 / TLS 既有冻结不解。
- **验证 / 节奏**：controller 实施与直接语义总审，independent=false；日志在 ignored `.cursor/plans/debt-voice-platform-guard-20261008-r0/`，首轮真实红灯保留，VM experimental 提示与换行提示不称零 stderr。完成适用专项与文档 / 精确保全后 local checkpoint；与 Agent 暂缓 / SDK 引用化 / Full 冻结共同形成有限批次，才做一次完整门禁与推送验收。证据足够即停止扩展调查。
- **本机工具前提**：首次 TTS 棘轮 native 1，因新执行包装未携带本机 Python 路径，缺 `py` 启动器；不是回归失败。保留原日志，仅在子进程设置已有 `OCLIVE_VOICE_PYTHON` 指向 portable Python 后该棘轮 native 0，父环境未改。原锁离线 npm ci native 0，不升级/刷新依赖；其中缓存 audit 输出不代替在线供应链结论，原完整图 low 边界保留。
- **实际适用出口**：TTS 棘轮、module-compat（含插件索引）、默认 Dimension 5 native 0（31 checks）；文档八项 native 0。追加本段后只复验受影响链接 / diff 和八路径精确保全，原 profile/锁/历史 Minimal 与其它台账保持。源码回归已足够支持该有限修复，停止增加等价证明；批次包含四个逻辑切片，完整本地和目标远端结果在冻结后另取，不将上述窄门禁写成全仓已验。

### DCL-20261009-67 · 注册表复用原 builtin complex emotion provider

- **原因 / 例子 / 有限判断**：K-EMO-03 的 builtin 规则仍在降级与 Fast 路径接线，不能按旧“单引用”印象删除。注册表私有 `BuiltinComplexEmotionArc` 只重复共享运行库原 trait 的同一个 `Ok(resolve_turn_inner(input))`；例如缺少 Remote 地址时仍以原规则返回相同输出，去掉包装不改变为何回退。D1 核到 trait、九处装配和直接消费者即足够，不扩大情绪质量或所有故障组合。
- **实施 / 责任 / 范围**：按[五路径计划](ROUND-02-PLAN.md#k-emo-03--注册表复用原-builtin-provider2026-10-09)移除这一私有包装并直接构造原 provider，旧关键词本体与 Remote/Directory/Fast 不改；主台账第二列用实际符号路径代替已移动行号，第五列登记本片，其它列/行与历史保持。controller 实施与语义自查，independent=false；无新公共 API、六槽/设施归属、权限、超时、重试、模型调用或状态策略。Full/Agent/双核/签名/TLS 暂缓保持。
- **已验 / 停线 / 接手**：原关键词回归修改前后均 5/5，原 Fast 3/3 修改前通过且在修改后 Host lib 637/637 中保持；定向 Host all-targets Clippy、workspace fmt native 0。达到等价整理证据后不新增同构测试；继续适用分层/module-compat、文档/债结构/编码/diff 和五路径总审，再存 local checkpoint，K-EMO-03 仍 OPEN。原件与 native 回执在本机 ignored `.cursor/plans/debt-emo-provider-20261009-r0/`，不随 Git 自动转让。
- **基线 / CI 节奏**：起点 `f78696a20ae143e61c7ba92f0673197db90fbc44` 已完整本地 native 0（1493.6002258 秒），起步时前批正式 37804969337 仍在观察，不能将其冒充本片远端结果。本片不独立推矩阵；相关有限批次收口才一次完整链与目标 SHA 正式 CI。此前观察会话中断与 API EOF 原始失败分别保留，只读续观同一个远端 run，未 rerun。宿主实机/真实音频/模型、用户 DB 与原 CP-INT 证据不碰。
- **实际适用出口**：分层、完整 module-compat（含六个插件索引）、八项文档/债结构/编码/diff 门禁 native 0；147 行/12 auto plans 只为结构输出，不是偿债数量。追加本句后只复验受影响的改文链接/diff，再核五路径、九处精确替换、原规则/adapter/Fast 与锁不变、其它台账行和计划/DCL 历史保持。没有公开 API 变化，不新增 doctest 或重复全量；换行提示保留，不称零 stderr。

### DCL-20261009-68 · Remote result 解码复用原 helper

- **原因 / 例子 / 停线**：Emotion、Agent、Remote/Directory 回复后处理和 Directory 剧场各保留一份相同的 JSON 解码及 `OllamaError` 包装；原 adapter 已有同一实现。例如 `emotion.analyze` 的坏 result 仍返回带原 context 的错误，不因复用 helper 触发 builtin 或重发请求。读到五处与原 helper 完全等价即整理，不开启 Full 设计或调查所有 Remote 故障组合。
- **实际范围 / 自主决定**：按[十路径有限计划](ROUND-02-PLAN.md#k-resilience-01--remote-结果解码复用既有-helper2026-10-09)复用 `decode_serde_value`；原类型、method/params、grant、timeout、回退、Agent 工具循环、后处理映射与剧场校验保持。中英协议文档补 helper 锚点，并将过期 Full OPEN 的复述换为已批准决策链接；台账仅同债第五列追加，本计划与 DCL 历史保持。controller 实施与直接语义自查，independent=false；没有新公共 API、依赖、权限、Kernel/六槽语义、自动重试或降级机制。
- **基线 / 已验前批 / CI 节奏**：本地起点 `abedde6565b8d09640ea2a2597fe26fe9254576a` 承接私有 builtin provider 整理。main 前批 `f78696a20ae143e61c7ba92f0673197db90fbc44` 的完整本地 native 0（1493.6002258 秒）及[正式 37804969337](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37804969337) attempt 1、17/17 success 含 ci-gate 已复核，四片只推一次、未 rerun；观察器会话丢失与续观 API EOF 原件保留，以 run/jobs API 终态而非观察器 native exit 证明正式结果。此处随实质整理回填旧 SHA 终态，不代替本片。两片新整理只存本地 checkpoint，相关有限批次出口才完整验证/推送，不逐个 helper 触发矩阵。
- **已测 / 边界**：原 Remote 29 条在修改前后以同一测试正文通过；纯等价私有调用不另写镜像测试，继续定向 Host Clippy/fmt、分层、八项文档门禁与十路径精确总审。首次 fmt native 1 仅四处新增 import 换行，原日志保留，规范化后受影响检查另留编号。证据位于本机 ignored `.cursor/plans/debt-remote-decode-20261009-r0/`，不会自动随 Git 转让；原库测试只用自有 loopback 与临时 grant，无外部 provider/真实模型/TTS/用户 DB/历史业务回放。K-RESILIENCE-01 仍 Partial / Full Deferred，公共兼容规则待维护者选择；其它冻结项不解冻。
- **实际适用出口**：规范化后的相同 Remote 29/29、定向 Host all-targets Clippy、fmt、分层、默认/五份改文链接、docs-only 旧路径、镜像/登记/债结构/编码/diff 均 native 0。未新增或放宽原测试；147 行/12 auto plans 仅为结构输出。追加本句后只复验改文链接/diff并核十路径与原 helper/传输/配置/类型/测试、许可台账单元格和历史正文；充分即停止补证。compiler 输出、换行提示及原 fmt 失败分别保留，不称零 stderr。

### DCL-20261009-69 · slot resolution 测试进程环境保护补齐

- **原因 / 实例**：前批 `2a85179094ee4f45cdba80a90f2a938f9c2670a4` 本地完整链 native 0（1277.3788755 秒），正式 [37818504017](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37818504017) attempt 1 的 Windows `env_llm_override_surfaces_in_debug_chain` 得到 None 而非 Some(remote)，15/17 success、ci-gate 随失败。本地通过不代替远端终态；原 Emotion/Remote 两片不再称整批远端已验，失败日志与观察器 native 1 原件保留，未 rerun。
- **实际隔离缺口 / 决定**：测试仅两条显式 env 用例持锁，其余两条 AppState 初始化也会通过原生产 DB→env 路径清 backend。按[四路径计划](ROUND-02-PLAN.md#k-llm-env-02--slot-resolution-测试进程环境保护补齐2026-10-09)给全部四条用例同一私有 fixture；退出/unwind 在锁释放前恢复原 backend 值。原断言、生产读取/环境事务和 Kernel/Host 公共语义不改；不为偶发放宽断言或让全部 CI 测试串行。远端具体清值线程未记录，不制造精确归因。
- **验收 / 原状态 / 止线**：旧实际 CI 为红例，原四条并发入口、定向 Clippy/fmt与适用文档/债结构/编码/diff 后形成 local checkpoint，再一次完整本地和新 SHA 正式验收；没有公开 API 变更，不追加 doctest 或同构锁测试。父债原 K-LLM-ENV-02 有限 Done 保持；K-EMO-03 OPEN、K-RESILIENCE-01 Partial / Full Deferred 与其它已定冻结不扩大。controller 实施和自查，independent=false；旧 CI、工作树和 CP-INT 原件保留，不启动真实模型/音频或读取用户数据库，充分即停止排查。
- **本片实际出口**：原四条在 `--test-threads=4` 下两次均 4 passed / 0 failed；定向桌面测试 Clippy `-D warnings`、fmt native 0。默认/三改文链接、docs-only 旧路径、登记/债结构/编码/diff 均 native 0；147 行/12 auto plans 只为结构输出，不是偿债数量。随后只复验受影响链接/diff并核四路径、原断言和生产/锁/历史保全，停止重复并发排列；完整本地和新 SHA 正式 CI 尚未取得，不能以窄测宣布 main 恢复。缓存 npm audit 输出不代替在线供应链结论，换行提示与原 CI failure 如实保留。

### DCL-20261009-70 · 可选扩展暂缓与公共兼容设计准入

- **维护者决定**：选择 1A/2A/3A/4A；K-CORE-BOUNDARY-01 仅准入有限公共兼容设计，Production Stream 继续暂缓，脚手架 Stage 2C 继续观察。K-EMO-06 明确 Deferred，高级情绪参与长期记忆筛选/重要性/权重保留为第三方模块或发行版的可选方向，不列为核心项目当前实现目标，不增加 Base 必做方法或字段。依据与重评条件只在[决定 §9](../../creator-docs/architecture/DESIGN_DECISIONS.md#emotion-memory-extension-deferred)，不复制模块定义。
- **原因 / 例子**：当前阶段优先保证基础能力并处理真实缺陷。高情绪的临时抱怨是否进入长期记忆，属于具体扩展的产品策略，不由六槽基础检索自动决定；有价值的未来方向不等于现在须由核心项目实施或现有契约已阻碍接入。只有可复现案例证明某接口挡住明确高级需求，才有界评估契约调整；新语义仍须确认。
- **实际范围 / 接续**：按[五路径计划](ROUND-02-PLAN.md#已确认的可选扩展暂缓与公共兼容设计准入2026-10-09)记录决定与受影响依赖，保持其它行/旧事件/历史证据。controller 实施及自查，independent=false；后续只设计六槽 Base 与最小逻辑角色的兼容草案，未经审核不宣布规则生效、发布稳定版或关闭父债。没有实现、测试、依赖、权限、模型/TTS、用户数据库或旧 CP-INT 身份变化。
- **前批真实终态 / 节奏**：基线 d984 的完整本地 `check:ci-local` native 0（1346.7177701 秒）及[正式 37864145434](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37864145434) attempt 1、17/17 success 含 ci-gate 已取得；旧 2a851790 的 15/17 红例保留，新观察器 API EOF 是 CLI 观察失败，终态由同 run API 核验，未 rerun。随本次实质决定登记，不为绿灯另起提交；不将旧 SHA 绿灯算作新方向记录或兼容草案的远端通过。本轮适用文档/结构核验后存本地 checkpoint，不重复 Rust、业务回放或完整矩阵。

### DCL-20261009-71 · 六槽 Base 与最小逻辑角色兼容审阅草案

- **产出 / 范围**：依维护者的 1A 在现有[兼容 SSOT](../../creator-docs/COMPATIBILITY.md#six-slot-minimal-compatibility-draft)及英文镜像提出有限公共清单、相容/Breaking 例子、迁移责任和适用测试入口；其它完整 Host/磁盘/网络/存储接口不因所在 crate 或名字相同被纳入。高级情绪驱动记忆仍按[已确认决定](../../creator-docs/architecture/DESIGN_DECISIONS.md#emotion-memory-extension-deferred)由第三方/发行版可选探索，K-EMO-06 Deferred，没有实现。
- **原因 / 例子 / 自主设计**：`Option` 字段在 JSON 中可能可选，在 Rust struct literal 中仍可能使旧实现不能编译；local future 增加 `Send` 也可能淘汰原合法实现。因此兼容层级必须分别说明，而非只看版本号或“新增”。复用当前 `#[non_exhaustive]` 错误边界和现有 Breaking 流程，不发明协商系统、稳定 1.0 或运行时兼容包装；保留原版本和所有现行能力语义。
- **证据 / 结论边界**：按[四路径设计计划](ROUND-02-PLAN.md#k-core-boundary-01--有限兼容审阅规则草案2026-10-09)读取现有 binding/types、最小逻辑校验、根 re-export 及直接 fixture 入口即停止。源码足以设计清单，不等于真实跨版本消费已经验证；controller 语义自查，independent=false。只跑文档/结构/编码及历史/写集核对，没有新 Rust/业务/模型/音频验证，不重复全量矩阵。具体规则待维护者审核，本条不能当规则已生效或父债 Done；公共契约仍依既有范围。

### DCL-20261009-72 · 采纳有限兼容规则与有界清理出口

- **维护者决定 / 当前范围**：已审核上片方案并选择采纳有限审阅规则；[中英兼容 SSOT](../../creator-docs/COMPATIBILITY.md#six-slot-minimal-compatibility-draft)从草案转为现行。六槽调用、local future、请求/失败载体与最小逻辑定义/校验的清单、例子及迁移边界保持，不扩大完整 Host、磁盘、网络或稳定版本承诺。K-CORE-BOUNDARY-01 父债仍 OPEN；K-EMO-06 Deferred 与其它暂缓不变。上片 DCL-70/71 是当时设计记录，原文保留。
- **辅助材料 / 收口理由**：维护者提供桌面《OCLive-历史技术债清理与完成标准》v2.0 供结合使用；仅将其中“明确实际受阻动作、复用有效证据、必修阻碍解除后结束专项”的要点接到[既有债务入口](README.md#有界清理与恢复开发出口)。桌面核实快照不成为仓库状态表、架构或门禁来源，不原样复制全篇/分类表，不自动改变优先级、解冻、删除缓存或启动全部验证。
- **原因 / 例子**：高级记忆策略暂缓不阻断基础六槽开发；真实音频缺口只影响需真实音频的验收，不能靠重复 Host 单测消除。恢复的是已明确且无阻碍的目标路径，不把剩余 OPEN/Partial 改写为 Done，也不把队列没有 auto 项当所有历史债务完成。
- **计划 / 验证边界**：基线为本地 `79dd65a0`，按[六路径接续计划](ROUND-02-PLAN.md#有限兼容规则采纳与有界清理接续2026-10-09)只更新当前规则/台账对应口径、依赖行、README并追加本计划/本事件。controller 直接实施和语义自查，independent=false；仅核受影响文档门禁、原范围/历史/源码与锁/桌面字节保全，不新增 Rust/doctest/业务/模型/实机或完整 CI。main 已验 d984 保持；本片只存本地 checkpoint，不以它宣称新 SHA 正式已验或全部开发已恢复。

### DCL-20261009-73 · 上游安全补丁与临时 override 撤销

- **原因 / 自主决定 / 实例**：此前批准的 shell-quote 1.11.0 override 是等待上游修复的临时兼容补丁。官方 concurrently 9.2.5 已在原支持线精确采用 1.12.0，原 ^9.2.1 允许；沿维护者已确认撤销方向更新父/叶并删除唯一该覆盖，无主版、架构、安全边界或工具链改变。保留 defu、deepmerge-ts、Tauri SDK 引用和 WebDriver browsers 四条例外，避免把一条撤销扩成全依赖刷新。
- **局部事实 / 停止线**：按[七路径计划](ROUND-02-PLAN.md#k-supply-12--上游安全补丁与临时-override-撤销2026-10-09)，生成锁恰两节点、其它字段全同；原 npm 10 ci / 全树 native 0。复用原兼容探针新副本，良性 quote/parse 和参数展开通过；四种危险分隔符只在内存 TypeError，双子进程成功 CLI 0、子进程 7→CLI 1；production build native 0。前后 production 0、full 4 low／0 moderate／0 high／0 critical。到此实际撤销条件足够，不继续证明攻击可达性或调查其它例外。
- **状态 / 历史 / 证据坐标**：K-SUPPLY-12 父债仍 Partial、KaTeX low 不关闭，旧批准/失败/audit/事件与其它已定冻结原文保留；中英安全记录增加当前撤销事实。安装 deprecated 提示、可选依赖缺失和 Git 换行提示如实报告，不称零 stderr。原件在本机忽略目录 .cursor/plans/debt-compat-20261009-r0/42–56-*，不随 Git 自动转让。controller 实施和专项自查，independent=false；没有新产品/Rust/测试/API或真实模型/TTS/用户 DB 运行。
- **批次出口**：与本地 79dd65a0 / ee99fdf3 的相关方向文档合批；适用门禁后冻结最终 SHA，一次完整本地 check:ci-local，再依既有推送授权 FF/push并取得该 SHA 正式 CI。当前尚未取得后两项，不能以 d984 已验或窄测宣称新 main 通过；终态存冻结回执，不另起仅回写绿灯提交。

### DCL-20261009-74 · 撤销后供应链摘要同步

- **原因 / 自主处置**：本批已撤销 concurrently 的临时兼容 override，中英供应链指南的当前摘要仍写“待上游撤销”。按 G17 对齐两处摘要并链接当前滚动观察；八月历史、旧安全例外、其它条目与父债 Partial 原样保持，不把指南改成新的状态来源。controller 实施与语义自查，independent=false。
- **计划 / 证据边界**：按[四路径文档计划](ROUND-02-PLAN.md#k-supply-12--撤销后供应链摘要同步2026-10-09)在 fd19e489 的完整本地链真实结束后实施，前12路径冻结验证期间未改。此片没有源码、配置、锁、脚本或运行行为变化，仅核适用文档与差量/历史保全，复用 fd19 的同一实现/依赖图全量证据，不重复 Rust/全量本地。最后实际 HEAD 仍须取得其正式 CI，不能声称它直接执行过 fd19 的本地命令，不是 CI 绿灯回写提交。
- **接手范围**：K-SUPPLY-12 保持 Partial、KaTeX low 和其它四条例外独立；K-EMO-06 Deferred、有限公共兼容规则已采纳，其它冻结不变。原本地记录在 .cursor/plans/debt-compat-20261009-r0/，忽略原件不随 Git 自动转让；本片到摘要对齐停止，不从这处文案重开全仓调查。

### DCL-20261009-75 · 基线全景审查与文档执行口径收束

- **原因 / 收益例子**：维护者希望先形成项目整体印象再选开发路线。有限八样本已足够行动；黄金路径仍写“CLI 接入待完成”，会误导使用者忽略已有 `validate-minimal-local`，属于当前文案漂移而非代码缺失。手册对 doctest、历史勾选和目标的表述也会使接手人误判覆盖或重复全量，按[七路径计划](ROUND-02-PLAN.md#基线全景审查与手册有限收束2026-10-09)修正。
- **交付 / 限度**：[项目审查与三条路线](waves/WAVE-20261009-PROJECT-REVIEW.md)分别解释 Kernel/六槽接入、Host/发行版主流程和具体维护债；手册及核实协议区分默认文档测试与显式选择，G8 不降低；中英黄金路径仅承认有限本地准备与 CLI，不宣称生成器/媒体/全部生命周期。旧历史、全部源码/锁/测试、台账与队列状态保持；没有真实模型、音频、硬件或生产数据访问。
- **自查 / 出口**：controller 实施与自查 independent=false；适用纯文档门禁及七路径、编码和历史保全核验后独立提交。代码基线 `2ef5af05` 正式 CI 已验，文档提交只作局部验证和分支备份，不冒充新 SHA 正式 CI通过。K-EMO-06 Deferred、Full 韧性/Agent/双核/签名等既定边界保持；没有 runnable 不意味着全债 Done，后续按具体使用目标选择而非继续无限证明。

### DCL-20261009-76 · AI 限制优化与有限证据复核

- **原因 / 自主取舍**：维护者要求结合 DeepSeek 桌面建议改善模型工作限制。搜索方法、旧注释、冻结状态和共享 harness 的误判风险成立；候选“单实现必须有第二实现者”和四数量只能降却可能误挡正常契约/工具开发，故改为真实职责审阅与可定位的只读观察，不加必需 CI。旧 unwrap 示例累计错算及 HEAD 重切的未提交工作风险一并修正。
- **实施 / 保留**：按[十二路径计划](ROUND-02-PLAN.md#ai-限制优化与有限证据复核2026-10-09)和[本批报告](waves/WAVE-20261009-AI-GUARDRAILS.md)补核实/拆分规则及入口摘要，contracts 仅改两处过期注释。保全主树四项候选与桌面原件，在独立工作树施工；公开签名/业务/测试/权限/feature/锁和原分层/依赖硬基线不动，现有台账与队列不改状态。Full/双核/单 Agent/签名等冻结不解。
- **验证 / 停线**：controller 实施和语义自查，independent=false；观察器同实现自测与错误负例、原分层/Dimension 5 --ci、适用文档/债结构/编码/diff和定向 contracts doctest。没有公共 API 或集成链改变，不重复 workspace 全测、正式 CI 或业务/live。达到已定位规则修正即停；Memory selector/trace 结构疑点先保留为未证实候选，不开启穷尽调查。本机保全/日志不随 Git 自动转让，最终结果以本批报告为准，局部验证不冒充债务 Done。
- **实际本地出口**：同实现自测/真实观察/npm/命令级负控通过，原11 contracts doctest和31项 Dimension 5 `--ci`通过（sample lib 显式跳过）；八项文档/原分层检查、fmt/语法与十二路径精确保全通过。独立 Git 夹具的非法 UTF-8/未知参数为预期 native 1，增长为0；换行与编译提示保留。达到本片 Locally verified 后冻结可转让提交；主树合流须先确认原四候选仍为保全字节及没有新重叠，不复用旧正式 CI 为新 SHA 盖章。
### DCL-20261009-77 · 路线一：外部 Memory 的真实最小消费

- **合流编号对账**：本事件来自 `f94069ddd7b26e7661c81dd310175c0b6fc9e857` 中的 DCL-20261009-76；主线 `b6a918748a933757011611d625e0e6e17f2e6823` 已将该编号用于上一事件，组合时将本事件登记为 77。两个原提交与分支原文保留；编号调整不改变原实施范围或将其 CI 改绑组合 SHA。

- **原因 / 锚点**：维护者选择 Kernel / 六槽模块实现者主线，第一片锁定外部 Rust 实现者 + Memory。按[本片计划](ROUND-02-PLAN.md#路线一--外部-memory-base-的最小消费者案例2026-10-09)先交 [MODULE_MAP §3.1.1 接入点](../MODULE_MAP_AND_HANDOFF.md#311-base-实现者接入点清单一页)，再交独立编译的 example crate；不把入口定位扩成全仓最小性调查，也不重开无关旧债。
- **实现 / 收益例子**：调用端编写 `RecentLiteralMemory`，只用公开 Base types / traits，现有 `MinimalRoleBaseConsumer` 真实委托；同一批材料按各实现的明确检索约定，新实现选最近两条、原关键词实现选三条，引述 / 否定 / 换行原样保留。空匹配正常返回、未支持要求保留完整 `Unsupported`、换材料不访问旧调用；模型调用零。共享消费者、契约、Host、锁、配置和旧 rich API 均无需修改，没有新增强制接口或六槽顺序。
- **证据 / 范围**：[本片交付](waves/WAVE-20261009-EXTERNAL-MEMORY.md)记录真实命令和失败；编译红仅因新案例私有实现模块缺失，不是旧公共符号缺陷。controller 实施与语义自查 independent=false；定向回归后本主题只做一次完整本地链，再冻结实际 SHA 取得正式远端 CI。台账和自动队列状态不改；本地证据不能宣称新 SHA 正式已验，远端编号先进交付报告，不为回写绿灯造提交。
- **停止 / 冻结**：外部 Memory 真实调用和适用门禁齐全即停；其余五槽 / 发行版 / 角色包作者路径不拓展。K-EMO-06、Full、签名 / TLS、双核、多 Agent、Production Stream、CI 责任分层等冻结保持；H04、S01、真实 TTS / 平台 / 未覆盖崩溃 / 浏览器 / 性能边界没有变化。

### DCL-20261009-78 · 辅助限制接续与并行基线合流

- **问题 / 取舍**：维护者要求接手辅助限制完善。上一批已完成的规则和观察器不重开；本轮只处理收尾摘要误读为无条件全量、CI 观察错误被误归远端失败、编译红归因，以及两支线同用 DCL-76 的真实接手冲突。按[有限计划](ROUND-02-PLAN.md#辅助限制接续与并行基线合流2026-10-09)和[本批报告](waves/WAVE-20261009-AI-GUARDRAILS-FOLLOWUP.md)修正，public G8 / L / required gate 保持。
- **合流 / 原件**：保留 `b6a91874` 的 AI 事件 76，导入 `f94069dd` 的 Memory 事件为 77，登记来源；两原提交、Wave、Rust 例子原件不回改。MODULE_MAP 只将 B1 旧说明改为历史判读，与已更新的 rustdoc 对齐，不改变六槽接线或契约。
- **出口 / 停线**：controller 实施和语义自查，independent=false；组合范围以本地定向回归、原观察器检查、文档与历史 / 写集核对验收，只有实际完成的命令才算证据。旧 Memory 完整链与正式 CI 保持 `f94069dd` 身份；本轮 Locally verified，不以原绿灯为组合 SHA 盖章，不标债 Done、不重复完整矩阵。既定冻结与台账 / 队列状态保持；达到实际口径修正及安全合流就停止。

### DCL-20261009-79 · Memory 实现者的人类开工入口

- **问题 / 原因例子**：已有基础 Memory 案例可被共享消费者实际调用，但人类开工包仍只把丰富 `MemoryRetrieval`、蓝图和 STM/LTM 列为入口。新作者可能因此先修改存储 / Host，而非实现自己的基础算法。按[本片五路径计划](ROUND-02-PLAN.md#路线一--memory-实现者的人类开工入口2026-10-09)补中英文路由和实际案例用法，不改模块定义或再证明全部六槽。
- **实施 / 保留**：[本片交付](waves/WAVE-20261009-MEMORY-AUTHOR-GUIDE.md)记录入口、已有命令与输出及消费者 / 资源限制；§1–§6 的丰富路径保留，仅核准参考 Host 的编排归属。两份人类指南仍拥有路由和 checklist，合同只链接 MODULE_MAP；原 runtime、契约、Host、测试、配置 / 锁、原记录及台账 / 队列状态保持，没有新的实现或配置式替换承诺。
- **验证 / 停止**：controller 直接实施与语义自查，independent=false；适用文档 / 镜像 / 债结构 / 编码 / diff和五路径 / 原历史 / 命令复用对账。原案例证据保持原 SHA，纯入口片不重复 Rust 或完整矩阵。读者能选择路径、找到实现与绑定、运行现有案例并理解有限结果即收口；不扩其余五槽、树外打包或发行版，不标债 Done，既定冻结和验收边界保持。
