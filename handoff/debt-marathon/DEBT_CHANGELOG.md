# 技术债变动记录与 AI 接手协议

**SSOT 范围**：技术债的依赖登记、变动事件、证据关联与安全续跑坐标；不维护第二份当前债务状态表。
**最后更新**：2026-10-30（历史事件外置；当前状态与执行准入未改）。

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
| K-TOOLS-01、K-VERIFY-01 | **共因**：门禁真实调用依赖与体检口径；二者不互为整债硬前置 | 主台账 §1；两个 ratchet 实际调用 PATH 上的 `rg`，`dimension5` 调用 ratchet；其原局部 `rgCount` 无调用点，本轮移除。须区分无匹配、缺命令和执行失败；体检区分存在/可用/真实门禁通过 | 仓库内行为修补及可携带静态体检见 [工具链 Wave](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20260928-TOOLCHAIN.md)；`E:\Env` 原始记录/个人脚本只读，外部修改另定范围 |
| K-BUILD-06、K-BUILD-07 | **共因 / 建议顺序**：共享构建环境；先测量，再分别评估链接与缓存策略。互不要求整债 Done | 主台账 §1、`.cargo/config.toml`、`package.json`；实测/历史纠偏与 CLI 启动器收敛见 [构建 Wave](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20260928-BUILD-OBSERVATION.md)。CLI 集成脚本仍只有 `--test-threads=1`，真实生成项目构建仍在，存留 exe 数也不等于单轮链接次数 | 先核配置、逻辑与物理大小、缓存新鲜度及旧证据用途；受控复用编译和冷重建分开预算。无等价证据不移除现有 `-j 1`；清理建议不是既定白名单，不自动删缓存或 rlib 变体 |
| D-DEBT-LEDGER-01 | **协同**：支撑接手可靠性；全表规范化不是其他债实施的硬前置 | 主台账前瞻风险、[COVERAGE](COVERAGE.md)；需明确一个权威状态行与显式引用，再逐类迁移历史/补冲突检查 | 随被触及条目增量整理 owner、范围和事件；有状态冲突时只暂停受影响任务，不要求全仓先重写台账 |
| K-ENCODING-01 | **共享执行约束**：所有中文文档写入；不是“先把此债做完才可开发” | [GATES](AI_AND_PIPELINE_GATES.md) §7 的编码红线及写后检查 | 对本轮改文核 UTF-8、无 BOM、正常汉字；长期执行观察与新的自动检查范围分开 |

### 安全、供应链与韧性

| 目标 | 前置关系与有效范围 | 依据 / 解除条件 | 可先执行的范围 |
|------|--------------------|----------------|----------------|
| K-PLUGIN-SEC-01 | Stage 3 原生隔离的证据门已在目标 SHA 完成；Stage 4 身份绑定以 K-SUPPLY-09 可信身份能力为**硬前置 / 当前暂缓的决策门** | [插件计划](long-plans/K-PLUGIN-SEC-01.md)与 [Stage 3 Wave](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20260930-K-PLUGIN-SEC-01-s3.md)；Full 仍需可信签名、轮换/撤销和开发 opt-out，不以窗口隔离代替 | Stage 4 只读梳理可复用；维护者恢复签名决策前不改生产信任根或桥授权，不关 Full |
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
| K-LLM-ENV-02 | **原缺陷已有限结案 / 新反例另立范围**：旧 DB 读取覆盖新环境并误清 dirty 的可控交错与目标 SHA CI 已验收；真实 provider、跨进程与长期压力不随之转绿 | 主台账 §1、[同债 Wave](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md)；`86cbb2d5` 的 CI 17/17，不把原先 Partial 阶段倒写成完成 | 无新的可复现反例不继续扩张矩阵；若出现独立故障，另定隔离与预算，不碰用户凭据或真实 provider 配置 |

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


### DCL-20261030-01 · Chat Pro rc.3 发包收尾

- **记录者 / 类型**：主控 Codex；用户授权清理旧包并准备新安装器、内测包。范围、原失败与验收阶段只链 [rc.3 波次](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261030-CHATPRO-RC3.md)，`independent=false`；不派 Agent。
- **范围 / 保持**：关闭数据库后的恢复文件 IO 避免阻塞运行时，候选版本与说明同步，桌面 rc.2 归档；角色/模型/聊天数据保留。六槽、公开 DTO、角色包 schema、CLI/内核版本与冻结能力不变，父债与 QUEUE 不转 Done。
- **证据 / 续跑**：定向回归后冻结并执行发包门禁，安装器与内测包绑定实际源码 SHA；本机验证、远端 CI、原生聊天与全新 Windows 分别记录，不继承旧 rc.2 验收或提前宣称交付。

### DCL-20261029-01 · Host 可靠性三项续接

- **记录者 / 类型**：主控 Codex；进展与关联复核，`independent=false`。用户要求继续，重新打包与内测包留到最后。干净起点 `3609005b`；外部清单 KER-01 / KER-11 / KER-12 的有限范围与当前验证只链[本轮 Wave](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261029-HOST-RELIABILITY.md)。
- **范围 / 保持**：同步 RPC 的 runtime 拒绝、迁移恢复诊断、SQLite 真源下的镜像 IO 顺序；关联核对正常角色删除。没有新增六槽/公开 DTO、数据库迁移、默认能力或签名信任根。主台账、QUEUE、冻结与 D-HOST-RECOVERY-01 剩余状态不变，不关闭 Full 或登记 Done。
- **证据与续跑**：原失败日志、最后冻结、整合命令和本机证据由 Wave 拥有；本轮不推送、不声称远端 CI 或已交付新包。合成测试可安全续跑；真实数据注入、实际模型和安装包验收不由源码单测代证。

### DCL-20261028-01 · 第三方复核后的有限修补（本地）

- **记录者 / 类型**：主控 Codex；复核、进展与交接。基线 `c9360444`；数据/授权切片 `d0697780`、插件边界切片 `a7645ffb`、前端切片 `0defa235`。细节、原失败首因、测试身份和本机原件位置只链[执行波次](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/waves/WAVE-20261028-REVIEW-FIXES.md)，不复制第二份修复状态表。
- **范围 / 状态**：用户授权先保护数据，再收紧已确认输入边界、默认界面可靠性与文档。K-PLUGIN-SEC-01 保持 Partial，K-SUPPLY-09 不解冻；D-HOST-RECOVERY-01 剩余范围保持 Deferred，不因新增客户端停止提示而宣称服务器取消。主台账与 QUEUE 的状态、准入和 currentStage 均未变，无新债 Done；本次维护不恢复五项 needs-reconcile 调度。
- **证据与下一步**：已完成的定向本地证据及最终整合结果的唯一获取位置见 Wave。本轮无 push、远端 CI、公开发版或新安装包。重跑静态检查/新合成夹具安全，不复用已消费的 live 身份；后续按 Wave 的冻结 SHA、本地整合与只读总审证据判定，不将本事件当作全部验收。

## 历史事件索引

旧事件全文已保存到个人归档；以下引用固定在整理前源码，保留当时证据，不成为当前状态表。新增事件仍遵守上方更新规则。

<a id="dcl-20260928-01--初始化审查与调度对账"></a>
<a id="dcl-20260928-04--仓库工具链首批实施"></a>
<a id="dcl-20260929-03--重复债务行改为权威引用"></a>
<a id="dcl-20260930-03--k-resilience-01-minimal-历史计划对账收口"></a>
<a id="dcl-20260930-04--k-cross-01-按声明能力划分宿主验收"></a>
<a id="dcl-20261001-01--文档声明支撑调查首批入账"></a>
<a id="dcl-20261003-04--k-ci-impact-01-全量-ci-节奏与责任边界登记"></a>
<a id="dcl-20261004-23--必需能力拒绝前发布修复的有限结案"></a>
<a id="dcl-20261004-36--同一最小角色的六槽可替换消费"></a>
<a id="dcl-20261004-37--独立-host-接入原生六槽"></a>
<a id="dcl-20261005-40--当前用户材料的-emotion-base-线索消费"></a>
<a id="dcl-20261005-41--按需-event-复用参考-host-的真实模型资源"></a>
<a id="dcl-20261005-42--已有真实-agent-向外开放-base-借用"></a>
<a id="dcl-20261006-43--cli-市场在替换旧安装前共用基础检查"></a>
<a id="dcl-20261006-44--当前最小角色状态凝缩历史原行保全"></a>
<a id="dcl-20261006-45--发行版默认身份兑现用户确认的优先级"></a>
<a id="dcl-20261006-46--cli-资源预览复用-host-编译器"></a>
<a id="dcl-20261006-47--cli-非交互资源策略草稿"></a>
<a id="dcl-20261006-48--新披露-npm-风险与测试链修复"></a>
<a id="dcl-20261006-49--cli-有限资源策略交互向导"></a>
<a id="dcl-20261006-50--selector-parser-单叶补丁预备片"></a>
<a id="dcl-20261007-51--兼容-chacha20-补丁与供应链合批"></a>
<a id="dcl-20261007-53--shell-quote-临时兼容补丁与撤销条件"></a>
<a id="dcl-20261007-54--资源原因说明合流与已验基线确认"></a>
<a id="dcl-20261007-55--remote-prompt-既有入口的增量收束"></a>
<a id="dcl-20261007-56--remote-memory-闸门有限收束与-prompt-终态回填"></a>
<a id="dcl-20261007-57--双核-beta-维护冻结与已验-remote-基线接续"></a>
<a id="dcl-20261007-58--cli-rust-action-的五处重复收束为一个步骤"></a>
<a id="dcl-20261007-59--供应链指南现状引用化与有限维护里程碑"></a>
<a id="dcl-20261008-60--英文直接情绪词沿既有规则补充"></a>
<a id="dcl-20261008-61--技术债接手现状的三处过期表述收束"></a>
<a id="dcl-20261008-64--tauri-sdk-override-的根声明引用化"></a>
<a id="dcl-20261008-65--保持-minimal并将-full-韧性暂缓"></a>
<a id="dcl-20261008-66--tts-平台拒绝主路径有限修补"></a>
<a id="dcl-20261009-67--注册表复用原-builtin-complex-emotion-provider"></a>
<a id="dcl-20261009-68--remote-result-解码复用原-helper"></a>
<a id="dcl-20261009-69--slot-resolution-测试进程环境保护补齐"></a>
<a id="dcl-20261009-70--可选扩展暂缓与公共兼容设计准入"></a>
<a id="dcl-20261009-73--上游安全补丁与临时-override-撤销"></a>
<a id="dcl-20261009-75--基线全景审查与文档执行口径收束"></a>
<a id="dcl-20261009-80--其余五槽实现者的入口导航"></a>
<a id="dcl-20261009-82--模块实现者里程碑与两条过期待验对账"></a>
<a id="dcl-20261009-83--干净工作树的验证资源前置与分阶段收口"></a>
<a id="dcl-20261011-87--kernel-契约基础工程脚手架"></a>
<a id="dcl-20261011-88--memory-第一条开发上手路径"></a>
<a id="dcl-20261011-89--基础脚手架的首次使用便利性"></a>
<a id="dcl-20261011-90--基础模块指令的常用与不常用分类"></a>
<a id="dcl-20261011-93--基础脚手架合批集成验收"></a>

- [DCL-20260928-01 · 初始化审查与调度对账](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-02 · 仓库流程成为正式来源](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-03 · 依赖登记与首批开工准备](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-04 · 仓库工具链首批实施](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-05 · 构建与缓存实测、历史口径纠偏](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-06 · CLI 集成测试直接使用本轮二进制](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-07 · 单 Host 链接资源样本取得目标归属](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-08 · 缓存元数据按文件对象去重](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-09 · 缓存用途与保留准则落文](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260928-10 · 单 Host 稳定 LLD 候选取得限定对照](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-01 · 已冻结 Host PDB 的离线消费通过](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-02 · Actions 固定来源与升级维护切片](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-03 · 重复债务行改为权威引用](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-04 · 七月 Verification 历史迁移](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-05 · 首部旧结论标明历史范围](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-06 · 独立 ID 行与引用的持续结构门禁](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-07 · Actions 固定引用目标 CI 终态入账](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260929-08 · 八月工程收口快照移出当前台账首部](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260930-01 · K-PLUGIN-SEC-01 Windows 原生隔离取证与测试判据修正](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260930-02 · Stage 3 远端验证完成，Stage 4 按维护者决策暂缓](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260930-03 · K-RESILIENCE-01 Minimal 历史计划对账收口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20260930-04 · K-CROSS-01 按声明能力划分宿主验收](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261001-01 · 文档声明支撑调查首批入账](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261001-02 · K-CI-IMPACT-01 失败作业重跑证据绑定修补](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261001-03 · K-CI-IMPACT-01 重跑绑定取得原生证据](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261001-04 · K-SUPPLY-10 rust-toolchain 同步合同](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261001-05 · K-SUPPLY-10 实际 Rust Action 更新配对模板](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261001-06 · K-SUPPLY-05 reqwest 范围与安全止点](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261002-01 · D-DEBT-LEDGER-01 六月轮次历史表迁移](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261002-02 · K-ENCODING-01 活跃文档自动防回退](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261002-03 · D-DEBT-LEDGER-01 Event Stream 长历史出权威行](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261002-04 · K-LLM-ENV-02 可控读取交错](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-01 · K-LLM-ENV-02 AppState 刷新入口的内存 DB 回归](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-02 · K-LLM-ENV-02 保存设置入口的标脏/应用对照](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-03 · K-LLM-ENV-02 chat 主入口的待应用设置](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-04 · K-CI-IMPACT-01 全量 CI 节奏与责任边界登记](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-05 · K-LLM-ENV-02 Theater 入口的待应用设置](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-06 · K-LLM-ENV-02 桌面 canonical seed 的文件库对照](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-07 · K-LLM-ENV-02 流式生成调用时的待应用设置](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-08 · K-LLM-ENV-02 文件库重建后的设置应用](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-09 · 停止扩展补证，回到具体债务处理](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-10 · D-DEBT-LEDGER-01 收敛 K-LLM-ENV-02 长状态行](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-11 · K-ENCODING-01 跟踪角色 JSON 防回退](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-12 · D-CLI-BLUEPRINT-05 显式最小角色文件校验](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-13 · D-CLI-BLUEPRINT-05 最小快照进入 Prompt Base](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-14 · D-CLI-BLUEPRINT-05 独立最小 Host 装配](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-15 · K-LLM-ENV-02 原并发缺陷有限结案](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261003-16 · D-CLI-BLUEPRINT-05 Prompt 能力输入去文件适配耦合](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-17 · 依赖图对齐最小角色与环境并发的现行裁定](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-18 · D-CLI-BLUEPRINT-05 双来源 Host 与磁盘边界](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-19 · 参考 Host 能力拒绝前的角色状态发布](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-20 · 最小角色基础互通与开发者转换器](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-21 · 参考 Rust Host 的最小角色基础文本接入](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-22 · 生成 library 直接消费最小角色入口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-23 · 必需能力拒绝前发布修复的有限结案](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-24 · 最小角色状态行与 CLI 用法去除过期表述](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-25 · 生成消费测试在 CI 彩色日志下误拒绝](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-26 · CI 公共契约与模型特化的渐进分责](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-27 · 最小角色发行版入口需要产品范围裁定](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-28 · 最小角色准备与 Prompt 实现解除固定绑定](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-29 · Host 按次选择 Prompt 而不改变默认调用](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-30 · 主流程目标与核心外共享适配已确认](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-31 · 核心外共用最小角色的基础文本操作](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-32 · 共享基础文本操作的冻结点已通过](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-33 · 最小角色的受保护基础传输](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-34 · 主聊天接线前的临时最小会话消费](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-35 · 最小角色接入 ChatPro 基础主流程](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-36 · 同一最小角色的六槽可替换消费](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261004-37 · 独立 Host 接入原生六槽](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261005-38 · 共享消费者复用参考 Host 的实际正文客户端](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261005-39 · 当前最小会话对话材料接入 Memory](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261005-40 · 当前用户材料的 Emotion Base 线索消费](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261005-41 · 按需 Event 复用参考 Host 的真实模型资源](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261005-42 · 已有真实 Agent 向外开放 Base 借用](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-43 · CLI 市场在替换旧安装前共用基础检查](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-44 · 当前最小角色状态凝缩，历史原行保全](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-45 · 发行版默认身份兑现用户确认的优先级](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-46 · CLI 资源预览复用 Host 编译器](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-47 · CLI 非交互资源策略草稿](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-48 · 新披露 npm 风险与测试链修复](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-49 · CLI 有限资源策略交互向导](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261006-50 · selector parser 单叶补丁预备片](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-51 · 兼容 ChaCha20 补丁与供应链合批](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-52 · 资源原因码的 CLI 展示闭环](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-53 · shell-quote 临时兼容补丁与撤销条件](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-54 · 资源原因说明合流与已验基线确认](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-55 · Remote Prompt 既有入口的增量收束](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-56 · Remote Memory 闸门有限收束与 Prompt 终态回填](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-57 · 双核 Beta 维护冻结与已验 Remote 基线接续](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-58 · CLI Rust Action 的五处重复收束为一个步骤](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261007-59 · 供应链指南现状引用化与有限维护里程碑](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261008-60 · 英文直接情绪词沿既有规则补充](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261008-61 · 技术债接手现状的三处过期表述收束](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261008-62 · 构建缓存的有限只读体检入口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261008-63 · 单 Agent 保持与多 Agent 合并暂缓](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261008-64 · Tauri SDK override 的根声明引用化](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261008-65 · 保持 Minimal，并将 Full 韧性暂缓](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261008-66 · TTS 平台拒绝主路径有限修补](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-67 · 注册表复用原 builtin complex emotion provider](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-68 · Remote result 解码复用原 helper](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-69 · slot resolution 测试进程环境保护补齐](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-70 · 可选扩展暂缓与公共兼容设计准入](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-71 · 六槽 Base 与最小逻辑角色兼容审阅草案](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-72 · 采纳有限兼容规则与有界清理出口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-73 · 上游安全补丁与临时 override 撤销](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-74 · 撤销后供应链摘要同步](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-75 · 基线全景审查与文档执行口径收束](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-76 · AI 限制优化与有限证据复核](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-77 · 路线一：外部 Memory 的真实最小消费](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-78 · 辅助限制接续与并行基线合流](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-79 · Memory 实现者的人类开工入口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-80 · 其余五槽实现者的入口导航](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-81 · 二次代码质量与过度工程审核](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-82 · 模块实现者里程碑与两条过期待验对账](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261009-83 · 干净工作树的验证资源前置与分阶段收口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-87 · Kernel 契约基础工程脚手架](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-88 · Memory 第一条开发上手路径](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-89 · 基础脚手架的首次使用便利性](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-90 · 基础模块指令的常用与不常用分类](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-91 · 基础工程问答与作者模板自定义](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-92 · 基础脚手架最终文档审查与反馈收尾](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-93 · 基础脚手架合批集成验收](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-94 · K-CI-IMPACT-01 责任分层首片准备](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-95 · K-CI-IMPACT-01 公共基础契约本地出口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-96 · CI分层分类基线与维护者对接](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-97 · CI责任边界确认与现有执行归属对照](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261011-98 · 公共契约与兼容检查执行所有权首片](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261012-99 · CI三层边界确认与首片合批集成](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261012-100 · 参考Host默认配置本地反馈](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261012-101 · 默认Host配置接入正式检查责任](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261018-102 · TTS棘轮解释器发现链](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-102 · 源码 SDK 第一条树外消费路径](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-103 · 脚手架中的终端接线声明视图](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-104 · SDK与接线视图文档、工程收尾](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-105 · 近期工作有限质量审查](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-106 · 二次复核候选的限定澄清](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-107 · 审查后入口样板实际收敛](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-108 · 当前批完整交接与SDK开发准备](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-109 · 源码SDK配方自动回归落地](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-110 · 单个Memory案例可模仿教学收束](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-111 · SDK复用权威文档与任务阅读入口](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261013-112 · 源码SDK最终审查、集成冻结与分类远端交付](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261024-113 · CI 分层认证的效果复查进入技术债](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261024-114 · 源码 SDK v0 的外部反馈进入技术债](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
- [DCL-20261029-115 · 草稿同步后 npm 公告补丁与依赖对齐](https://github.com/linkaiheng2233-cyber/oclivenewnew/blob/925310c83c94d53dda66fda8eb0c0f1cd9f50de4/handoff/debt-marathon/DEBT_CHANGELOG.md)
