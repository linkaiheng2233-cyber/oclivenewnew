# 技术债变动记录与 AI 接手协议

**SSOT 范围**：技术债的依赖登记、变动事件、证据关联与安全续跑坐标；不维护第二份当前债务状态表。
**最后更新**：2026-09-29。

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
