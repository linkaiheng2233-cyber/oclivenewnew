# 第二轮马拉松计划：解除阻断后再偿还

**SSOT 范围**：本文记录第二轮技术债马拉松的阶段安排及启动前复核；债务状态以 [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) 为准，自动队列以 [`MARATHON_QUEUE.md`](./MARATHON_QUEUE.md) 为准。
**最后更新**：2026-09-28（基线复核、工具链实施及构建成本观测）。

> 入口门禁：[`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md)。本计划先处理外部阻断，不擅自把 human/skip 项改成 auto。

> **2026-09-28 基线复核快照**：Wave 1 提及的 [#124](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/124)、[#125](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/125)、[#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 均已合并；下方 Wave 1 是历史计划，不是当前待审 PR 清单。复核时队列仍有 `pr-open` 项；后续状态与来源处理见本页“后续文档治理与接手准备”，不将历史快照当开工指令。

## 目标

按具体能力切片核对依赖，优先处理影响日常验证可靠性的本地事项；只在证据、权限和运行条件齐备后启动自动 Stage。已合入的历史 PR 栈只用于对账，不反复领取旧工作。

## 启动前基线审查（2026-09-28；限定范围）

本轮受检代码起点为 `6da239557b786c8f7b0d50242a6873f7e1477b62`（开场工作树干净，`main` 比 `origin/main` 超前一条文档提交）。这是一轮**快档基线＋架构/队列/文档定向审查**，不是全档健康评分，也不是任何债务的 Done 验收。审查期间仅修正巡检流程、核实协议和本计划的文字；产品源码未改。上一个远端 [`ci.yml` run 36326312754](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36326312754) 只覆盖父提交 `876b95d1a601499db86bc942dd4978c86d8b53fe`，不覆盖本地新文档 HEAD。

| 审查面 | 本轮证据 | 判定与边界 |
|--------|----------|------------|
| 综合本地门禁 | `npm run check:ci-local` **exit 0**（Dimension 5 → `npm run check` → Rust workspace/CLI 集成）；原始日志 `C:\Users\13603\AppData\Local\Temp\oclive-predebt-ci-local-20260928.log`，316603 B / SHA256 `646EDA57D77C9218F267C3FB59811634BA3B6BD13E8759EF5B124BA7CC1C6DE7` | 本地工程基线通过；CLI 的无效 `monolith.toml` 报错是通过的拒绝用例输出，不写成“零 error”；该命令不包含真实模型/语音或最终文档 HEAD 的远端 CI |
| 基线与前端 | `dimension5-acceptance.mjs --ci` **PASS (28 checks)**；`npm run test:unit` shared **243/243**、Chat Pro **92/92**；`npm run check:module-compat` **10 slots / 9 manifests / 7 UI contributions**，插件索引 6 项 | 本地适用检查通过；`--ci` 中 sample workspace lib tests 是脚本设计的 SKIP，完整 workspace lib 测试由上行 `npm run check` 的 Rust 阶段覆盖 |
| 内核边界 | `cargo metadata --no-deps --format-version 1 --offline` 的内部依赖仍为 types→contracts→runtime→host，上层 server/Tauri 消费 Host；分层 ratchet use-import **3/3**、FQ **1/1**，Host re-export **75/75** | 只证明静态依赖与 ratchet；不推出最小 Kernel 已物理抽离、全部六槽真实后端或发行版全合规 |
| 债务计划结构 | `npm run check:debt-marathon` **PASS (12 auto plans)**；受检源码目录的 TODO/FIXME 只命中 `plugin_scaffold.rs` 两处脚手架模板 | 结构校验不查询 GitHub PR 状态，不能代替开工前远端对账；未新增债务编号 |
| 外部状态 | 队列 seq 40/50/60/120/130 仍为 `pr-open`；当前 `gh pr list --state open` 未列出对应债务 PR；[#124](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/124)、[#125](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/125)、[#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 均为 `MERGED` | **自动派工暂缓**：先逐项映射队列、计划书、台账与远端 PR；不得把 `pr-open` 直接转成 runnable 或 Done |
| 流水线前提 | 项目 `.cursor/skills/oclive-dev-pipeline/SKILL.md` 与 `AI_AND_PIPELINE_GATES.md` 可读；本机 `~/.cursor/skills/dev-pipeline/SKILL.md` 未找到 | 本轮只读审查可依项目规则执行；后续 L 级债务 Stage 须先确认通用规则来源，不冒充已走完整七阶段 |

**分列结论**：工程本地基线 **PASS（限定于以上命令和受检源码）**；自动马拉松开工条件 **HOLD: needs-reconcile**。下一步先核实五条 `pr-open` 的真实 PR 映射与计划/台账，再确认通用流水线规则来源；之后才选择一债、一 Stage，按适用门禁与目标 SHA 的远端 CI 规则推进。本轮不将任何 Partial/OPEN 改为 Done。

未执行真实模型、真实语音/硬件、长时性能矩阵、完整技术债逐条复核，也未重跑 Chat Pro 限定接入场景。历史 S01 语义质量 FAIL、H03 真实音频与未覆盖崩溃窗口，仍按各自证据边界管理；[`CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md`](../CHATPRO_HOST_KERNEL_INTEGRATION_GATE.md) 的限定案例不扩大为全 Host 或独立小 Kernel 合规。本轮无产品架构变更、无台账状态迁移、无新 run ID。

## 后续文档治理与接手准备（2026-09-28）

本节更新上方快照的接续动作，不改写历史检查结果。维护者要求审查债务、建立实时变动与接手规则，并以仓库通用流水线为准安装到本机。**尺寸 M**：仅文档、调度前提和 Skill 副本，不实施业务债或改变架构；起点 `6ed1dda1f0ab5317450ce4609a6c29c47d423e50`，工作树干净。

| 切片 | 范围 / 产出 | 验证与止点 |
|------|-------------|-------------|
| 债务审查 | 台账现行状态、历史引用、覆盖声明与旧 PR 等待态；详细发现链 [COVERAGE](COVERAGE.md) | 现行源码/owner/PR 对账；未知原始证据不补造，未测产品状态不升 Done |
| 接手规则 | [DEBT_CHANGELOG](DEBT_CHANGELOG.md) 为变动事件来源；入口与模板统一链接，当前状态仍在主台账 | 文档链接/登记/路径；不得生成第二份状态表或回改冻结证据 |
| 调度暂停 | 五条旧 pr-open 与计划契约同步 blocked；保留 Stage 与 Minimal/Full 条件 | `check:debt-marathon` 结构通过，`--require-ready` 必须拒绝这些 blocked 项；本轮不启动马拉松 |
| 通用规则与镜像 | [仓库 Skill](../workflows/dev-pipeline/SKILL.md) 成为正式来源；三文件副本安装到本机 | Skill 格式、中文编码与 bytes/SHA256 相等；遇到本机不同私有字节先对账，不静默覆盖 |

**当前接续**：来源缺失由仓库规则解决；五个计划的调度仍暂停，后续先确认剩余能力与证据再决定关闭历史计划或修订新 Stage。详情和最后动作见 [DCL-20260928-01/02](DEBT_CHANGELOG.md)。这不授权此前暂停的 Full、实机、签名或跨仓工作。

**本轮 CI 节奏**：纯文档/计划状态检查＋只读最终 diff；不重复工程全量测试或业务身份。适用本地检查后建立文档里程碑提交；没有业务债 Done 迁移，不用父 SHA 的绿 CI 声称新提交已远端验收。若后续债结案，仍须其目标 SHA 的适用本地与远端证据。

## 当前开工准备（2026-09-28）

**范围与尺寸**：本轮为 M 级依赖/计划文档切片，base `415d9592990002923ab1d2285ecad9bc998a01bd`，开场工作树干净；未实施业务债。维护者要求先完成 [依赖登记](DEBT_CHANGELOG.md#依赖登记与判读) 再准备开工。首批工程切片按下面范围接续；自动马拉松、外部个人脚本改写、架构变更和 Full 实机/发布不由本计划自动开启。准备完成与债务 Done 分开。

### 工作顺序与独立条件

| 顺序 | 范围 | 进入条件 / 出口 |
|------|------|-----------------|
| 首批 | K-TOOLS-01 与 K-VERIFY-01 的仓库工具链切片 | 同源问题合用一个计划，分别记父债进度；先核真实调用/外部 owner，再做可携带项目检查与失败判据。外部体检联动和新环境完整门禁是单独验收，不因仓库窄测通过即关两债 |
| 后续候选 | K-BUILD-06 / K-BUILD-07 的只读测量 | 工具可用且日志口径明确即可开始；不等首批所有外部条件结案。先量当前链接/缓存及冷热构建，再定实验预算；无证据不删产物、不移除现有串行措施 |
| 随切片推进 | D-DEBT-LEDGER-01 的增量规范化 | 每次触及条目核权威状态/历史引用并写事件；状态冲突只暂停对应任务，完整治理不是业务修补的统一前置 |
| 条件到位后单独选项 | 插件原生安全证据、CLI 角色生命周期、资源工具及其他高优先级面 | 以登记的实际能力/平台/决策条件准入；不因本地工具优先安排而降级其台账优先级，也不自动按 QUEUE 之外的顺序派工 |

这是控制验证成本的建议顺序，不是新依赖图或第二份状态台账。五个 `blocked:needs-reconcile`、`human`、`skip` 和语音 v2 规范门保持原条件；旧 Wave 1/2 不作为首批本地工具链的统一阻断。

### 首批工具链切片合同（实施准备）

**Owner / 边界**：仓库门禁行为由 `scripts/` 和 `package.json` 决定；个人环境体检的 owner 为 `E:\Env\scripts\inspect-engineering-environment.ps1`（项目中性，不随 Git 携带），历史报告不是当前门禁运行结果。本轮已读源码；后续默认在仓库形成项目专属可携带检查，不扩大通用个人体检的职责。不改生产 Kernel/Host/DTO、安全策略、用户/系统 PATH 或个人环境脚本，不安装依赖。

**影响链**：PATH 上真实工具及退出码 → 共用进程/计数处理 → 分层与 re-export/纪律门禁 → 项目检查的存在/可用/实际通过三态 → 缺依赖修复提示与非零拒绝 → 子进程正负控及综合门禁。`rg` 无匹配的 exit 1 与缺命令/权限/异常退出必须分开；不得把后者吞成 0 次。当前 `dimension5` 的局部 `rgCount` 捕获全部异常后返回 0，第一项分层检查仍会拒绝缺 `rg` 的整轮；因此只登记局部判据缺口，不宣称现有整套门禁一定假绿。

| 切片 | 文件或能力范围 | 验收及适用原因 | 产出 / 失败止点 |
|------|----------------|----------------|-----------------|
| 0 · 重现与方案冻结 | 只读 `scripts/check-domain-layering.mjs`、`scripts/check-host-reexport-imports.mjs`、`scripts/dimension5-acceptance.mjs`、调用者及外部体检；选择复用位置 | 两个 ratchet 实跑取得受检源码基线；使用绝对 Node 入口和**仅子进程**受控 PATH 验证缺 `rg`；不改父进程/持久 PATH | 命令、计数、退出码与错误分类；确认单写集和 helper 复用。环境已有残留/原始证据不可取得时先记录，不擅自修个人环境 |
| 1 · 仓库修补 | 上述实际调用点、必要共享 helper/项目检查与行为自测；最终新增路径在切片 0 后登记，不预填不存在命令 | 同实现验证匹配/无匹配、命令缺失、非零执行异常，以及存在但不可用/未跑项目链的拒判；正常路径计数与受检基线一致 | 明确的非零、可执行修复提示及项目检查三态；不以隐藏失败、降低 ratchet 或把工具改成可选过关 |
| 2 · 冻结与集成验证 | 冻结代码、受影响 SSOT、父债/变动事件与交付 | 窄测和两个 ratchet → `dimension5 --ci`；因改动位于 `check:rust`/综合门禁链，里程碑再跑 `check:ci-local`。恢复缺依赖的受控环境须另跑范围内完整链，不把负控本身当修复通过 | 分别报告仓库 Locally verified、环境联动待证和各父债未齐条件；L 结案仍需目标 SHA 远端 CI，不能沿用父 CI 或仅按脚本数转 Done |

**止点 / 回退**：若需要改变计数语义、公共契约、机器全局环境或外部体检写集，先呈报实际差异再决定；确定性缺依赖不反复跑完整 Rust 构建。正负控与计数不一致先定点修复，不放宽基线。局部实现可按已核 diff 回退，不使用 stash/reset/clean 清理他人改动。

**CI 节奏与续跑**：本轮只跑适用文档/计划检查；下一工程切片先窄测，集成里程碑通过后再冻结提交/推送，不为每次注入失败触发远端。精确下一步为 `node scripts/check-domain-layering.mjs` 和 `node scripts/check-host-reexport-imports.mjs`（静态计数基线，可安全重跑），随后完成子进程错误分类负控；当前没有业务身份或生成预算被消费。所有接续结果同检查点写 [变动事件](DEBT_CHANGELOG.md#变动事件)。

### 工程切片启动（2026-09-28）

维护者已明确“开始工作”。工程 base `32cd11a81a6fe7641b228fa8da2d39676d2791d8`，工作树干净；先做 M 级仓库修补，不迁移父债 Done。切片 0 实跑分层 **3/3、FQ 1/1** 与 re-export **75/75**；子进程空 PATH 下两者均 exit 1 且裸 `ENOENT` 栈，原始证据 `.cursor/plans/debt-toolchain-20260928-r0/s0-baseline.json`。父环境未改，原始证据本机持有。

实际写集冻结为两个 ratchet、`dimension5-acceptance.mjs`、`package.json`；新增 `scripts/lib/gate-toolchain.mjs` 及其 `.test.mjs`、`scripts/check-toolchain.mjs` 及其 `.test.mjs`；关联本计划、变动记录、主台账及本轮 Wave。共享处理只服务真实命令调用与计数，不扩为另一个流水线框架。源码盘点确认 `dimension5` 的 `rgCount` 没有调用点，删除该死函数，不将其历史局部吞错描述当作当前行为。ripgrep 仍按匹配行计数，路径/计数用 NUL 分隔避免 Windows 盘符歧义，既有 baseline 和排除规则保持。

项目体检仅声明 **Node/Git/Cargo/rg 及静态项目链** 的结论：执行版本探测、两个实际 ratchet、Git 工作树和离线 Cargo workspace metadata，分别输出存在、可用、实际链通过（未运行为 null）。不推断链接器、前端/全部 Rust、语音/实机已可用；不改 `E:\Env`。窄测接 `npm run test:toolchain`，体检接 `npm run check:toolchain`；命令合同测试进入 Dimension 5，综合门禁按上表里程碑运行。

**工具链执行出口**：仓库切片 Locally verified，详细命令和字节依据只在 [工具链 Wave](waves/WAVE-20260928-TOOLCHAIN.md) 维护；缺 `py` 的首次综合失败保留，按既有配置显式选实际 Python 后全链通过。两个父债为 Partial；新环境完整链、外部个人体检联动及目标提交远端 CI 分列，未关闭父债或解冻其他计划。后续构建测量已按下节接续，目标 CI 仍须分别核对。

### 构建成本与缓存切片（2026-09-28）

**尺寸 / 范围**：M 级测量与台账纠偏，base `d30f47c75bc9ef7454fb308204f1997c884b3422`，开场干净；K-BUILD-06/07 仍 OPEN。本轮复用上述独立条件，不启动自动马拉松。配置、源文件和旧证据保持，当前观测与执行出口见 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md)。

| 切片 | 合同 / 验收 | 止点 |
|------|-------------|------|
| 0 · 只读盘点 | 冻结 Cargo/package 配置与限定环境变量；离线 metadata、当前 target 逐路径逻辑大小、硬链接/重解析点、独立枚举及旧日志身份。完整总量须无读错、无预算截断、两种枚举一致；逻辑大小不冒充实际占用或可回收量 | 单枚举 60 s / 200 万项；不足则记 partial，不补造总量，不删缓存 |
| 1 · 复用缓存编译测量 | 原 target 上 `cargo test --locked --offline --workspace --exclude oclive-cli --tests --no-run -j 1 --message-format=json`，最多两次且首次成功/输入不变才可重复；记录 native exit、Cargo fresh 标志、采样内存、前后元数据差。只编译，不运行测试/ignored 场景；第一次不预设为纯热缓存 | 每次 180 s；采样空闲物理内存低于 4 GiB 或所持 Cargo 子树私有内存合计超过 12 GiB 即停止该子树并保留失败；不改变 jobs/linker/profile、依赖版本或全局环境 |
| 1b · 单 Host 目标 | workspace 观测触及 180 s 止点后不重复它；以同一 workspace 选择和配置，仅将 `--tests` 收窄为 `--test a_turn_harness`，保持 `--no-run` / `-j 1` / offline。独立新输出根，最多两次且首次成功才重复；仅此目标可声明复用基线，不替代 workspace 未完成结果 | 每次 60 s，沿用相同内存止点；再失败即止，不扩大本组时间/范围、不用单目标绿替代全链；后续预算复评另列 1c |
| 1c · 定额完成基线（预算复评） | 1b 已按止点结束；日志显示依赖重编、无编译诊断，采样私有内存低于 1 GiB，旧源配置未漂移。原短预算无法完成基准；新独立输出根继续**同一**单目标命令，不重跑 workspace、不改编译配置。此为额外定额实验，旧失败保留；首次成功/输入不变才做短复测 | 本轮最后一组，首次最多 480 s、复测最多 60 s；内存止点保持。任何编译错误/预算触发/未知存活或输入漂移即结束，不继续递增预算 |
| 2 · 评审下次实验 | 区分完全复用、增量未命中、空目标目录（依赖源码可复用）与删除后重建；同一命令/源码/特性/工具链才作对照。先列候选路径、硬链接物理空间与冻结证据关系、磁盘/时间预算，再定冷构建或维护实验 | 本轮不跑空 target 或删除后重建，不删 `deps`/rlib/pdb/build/fingerprint，不启链接器或并发替换 |

**适用门禁与交接**：工程配置和门禁实现未改，本轮仅跑文档链接（默认＋改文）、文档旧路径、登记、债计划结构、编码和 diff 检查；不重复完整 Rust/业务验证。缓存测量成功只推进证据，不关闭链接峰值/维护策略两债。完整阶段结果按 Wave 入账；只读盘点可安全重跑并使用新输出名，编译测量每次独立日志，未知存活进程、输入漂移或旧证据冲突只暂停该实验。

## 历史阶段安排

下列 Wave 保留旧计划意图；不覆盖上方现行切片安排、[QUEUE](MARATHON_QUEUE.md) 或计划机器契约，也不将已合 PR/已结案债重新列为待实现。

### Wave 0：第一轮封存与证据对齐

- 核对 `MARATHON_QUEUE.md`、long-plan、inventory、Git SHA、PR 状态和 CI。
- 保持 `pr-open` / `blocked` / `human` / `skip` 原状。
- 产出：本轮 closeout、无冲突的队列和可复现的下一步命令。

### Wave 1：PR 栈审查窗口

- 审查 `#124 → #125 → #126` 的依赖、diff 和新 SHA CI。
- 只有获得明确合并授权后才执行 merge；否则维持 `pr-open`。
- 产出：每个 PR 的 accept/revise/block 结论。

### Wave 2：解除外部阻断

- `V-VSCODE-PERF-05`：姊妹仓和 `.vsix` 实机权限。
- `K-CROSS-01`：三平台 smoke 环境。
- `K-DIST-01`：签名/updater 密钥与发布权限。
- `K-VOICE-07`：RFC v2 锚点。
- 产出：授权快照或稳定 blocker code；没有条件则不派发 Implementer。

### Wave 3：下一批自动偿还

仅当 Wave 1/2 消除冲突后，从 queue 中把明确具备 `local-write,test` 条件的单项计划置为 `ready`，按 seq 每次只领取一个 Stage。候选顺序：`K-RESILIENCE-01 Full`、`K-SUPPLY-05-Full`、`D-ROLEVER-01`、`T-DOC-02`。

## 结束条件

- 无可验证证据的事项保持 `Partial` / `blocked`。
- 所有 Stage 均有 Wave、命令、SHA 和测试结果。
- 无 runnable auto 时正常结束，不为了延长轮数而改动 skip/human 状态。
