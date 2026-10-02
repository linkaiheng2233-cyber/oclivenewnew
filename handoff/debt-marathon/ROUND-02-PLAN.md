# 第二轮马拉松计划：解除阻断后再偿还

**SSOT 范围**：本文记录第二轮技术债马拉松的阶段安排及启动前复核；债务状态以 [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) 为准，自动队列以 [`MARATHON_QUEUE.md`](./MARATHON_QUEUE.md) 为准。
**最后更新**：2026-10-02（接续六月轮次 16–19 归档；早期基线与实验按各节日期保留）。

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

### CLI 集成启动切片（2026-09-28；构建债接续）

**尺寸 / 范围**：M，base `6c28b1e2e5cb42214120e7a5903c85c4420e5365`，开场干净。诊断发现 `oclive-cli/tests` 部分用例借 `cargo run` 重入本仓库构建，而现有 scaffold/registry 用例已使用 `CARGO_BIN_EXE_oclive-cli`。本切片统一为 Cargo 为本轮测试提供的 CLI 二进制，不改变 CLI 生产代码、默认特性、清单提交身份、生成项目的真实构建检查、编译 profile 或缓存保留策略；两父债保持 OPEN，自动队列不领取。

| 阶段 | 范围 / 验收 | 预算与止点 |
|------|-------------|------------|
| 定向前测 | 四个现行 explain/completions/dry-run/from-existing 目标，同一源码和 `-j 1` / 单测试线程；记录命令、原生退出、测试结果与子进程采样。该测量含首次构建，不能当纯热性能基准 | 最多 480 s；限定本命令继承离线环境，不改父环境；仅所持进程树清理，低于 4 GiB 空闲物理内存或合计私有内存超过 12 GiB 即停止 |
| 实施 / 窄测 | 复用 `tests/common` 的路径 owner，增加统一命令构造点；保持原业务参数、cwd、OCLIVE_ROOT 和 experimental 选择。覆盖全部旧 `cargo run` 启动点，留下生成项目的 `cargo build`；加一条真实缺 Cargo PATH 负控，定向四目标重跑 | 前测非零先归因；每尝试新日志，原失败不覆盖。不扩为生产 CLI 重构，不解除串行测试规则 |
| 里程碑 | CLI crate 全测（含已有生成项目/release 构建与 bench 夹具）、CLI all-targets/all-features clippy、workspace fmt、分层及本轮文档/债计划/编码/diff 检查；冻结后一次提交/推送，目标 CI 单独绑定 | 全测最多 900 s、clippy 最多 480 s；无真实模型/音频/业务回合/旧证据改写。无公开 Rust API或门禁组合改变，不重复无关全仓业务链；台账不升 Done |

**关联与交接**：测试生产者为本轮 Cargo 编译的 CLI executable，消费者是原集成用例；保留真实 CLI 命令、stdout/stderr 和拒绝行为，消除测试启动器额外 `cargo run`。Kernel/Host/Tauri/shared 的生产路径及发布安装方式已核对无需改。Git HEAD 被 runtime build script 监测属于清单溯源合同，不以减少重编为由删去。执行记录和剩余链接/缓存条件仍写入 [构建 Wave](waves/WAVE-20260928-BUILD-OBSERVATION.md)，不新增状态表。

### 单 Host 链接目标绑定（2026-09-28；诊断接续）

**尺寸 / 范围**：M 级观测，base 为已本地验证的 CLI 测试切片 `787aa5cb`（完整 SHA 写入本机输入件），开场干净。现有点采样尚未把 linker 内存与输出目标一一绑定，不能据此选择调试信息/链接器调整。只在原 target 再用既有单 Host `--no-run` 命令，捕获所持编译子树内 linker 的 command line / response-file 与 `/OUT`；匹配 Cargo artifact 中的实际 executable/共享库后才宣称该样本归属，不把缺失采样补造。

| 切片 | 合同 / 验收 | 止点 |
|------|-------------|------|
| 输入 / 参数 | 冻结 Cargo/package、runtime build script 与 CLI 切片；保持 `--locked --offline --workspace --exclude oclive-cli --test a_turn_harness --no-run -j 1 --message-format=json`，不人为 touch 源码或清产物，仅允许本次提交清单的正常重编 | 编译器已存活或输入漂移则不启动；不改 jobs/linker/profile 或提交身份合同 |
| 一次有限测量 | 一个新 `host-link-01`，最多 480 s；250 ms 目标点采样所持树并记录实际间隔/资源，不承诺捕获短进程。只读所观测 compiler 命令引用的 response-file，单文件最多 4 MiB，限制在系统 Temp 的 rustc 临时夹具或当前 target 范围内，保留捕获/未捕获原因 | 空闲物理内存 <4 GiB、所持树私有合计 >12 GiB、deadline、读取边界不明或未知存活即停止该实验；只终止所持 Cargo 子树，不按名杀进程 |
| 交接 | 原生 exit / Cargo build-finished / 源配置未漂移分列；绑定后只报告 sampled peak，不冒充真峰值或 OOM 复现。仓库仅更新本计划、Wave 与债事件，命令采集器保持本机一次性工具 | 不自动发第二次、空 target/冷重建、删缓存或调试信息试验；若绑定未取得则保留 unknown，后续方案另定，不反复扩预算 |

**适用验证**：纯测量/文档无生产或门禁改动；默认及改文链接、docs 旧路径、登记、债结构、编码/diff 与输入身份即可。它不替代已通过的 CLI 全测，也不把主台账 K-BUILD-06/07 转 Done。真实 provider、DB、音频和业务 ID 均不产生。

### 缓存分配与文件身份去重（2026-09-28；只读接续）

**尺寸 / 范围**：M 级测量与文档，base `83af77bb74daf9eed6092770b80824c5272ce1f8`，开场干净；只核既有 `E:\OCLive\oclive-dev-artifacts\oclivenewnew-cargo-target` 的 NTFS 元数据。使用 Win32 文件句柄身份去重，并分列按路径逻辑大小、唯一文件对象的逻辑大小和 `FILE_STANDARD_INFO.AllocationSize` 报告的分配量。对象的多个硬链接不能重复累加；存在根外链接、压缩/稀疏/命名数据流或扫描期间变动时单独登记，不将分配量称为可回收磁盘空间。K-BUILD-06/07、队列与保留策略均不迁移。

| 阶段 | 合同 / 验收 | 预算与止点 |
|------|-------------|------------|
| 前置与真实夹具 | 冻结源码/构建输入及限定环境；核 root 为本地 NTFS、边界与重解析点。一次性采集器以零数据访问权限/只读元数据打开现有文件、随即关闭；在新忽略证据根用全新普通文件及硬链接验证身份去重、空文件和越界拒绝，不读取缓存内容 | 不运行 Cargo/产品/业务，不清缓存；夹具不引用已有产物。API 不可用/重解析点/身份不一致不以文件长度回退成分配量 |
| 有界扫描与独立核算 | 写新逐项元数据账本；重算按路径及对象汇总，并独立枚举路径/逻辑大小作对照。完整结果必须无错误、预算截断或扫描变动，句柄全部释放；没有命名流盘点或卷级证据就明确保留该边界 | 单扫描最多 60 s / 200 万项；失败仅记 Partial，不扩大预算或递归修改源目录。编译器前后点检查为空不宣称目录原子静止 |
| 文档出口 | 同步既有 Wave、台账与变动事件；默认/改文链接、旧路径、文档登记、债结构、编码及 diff。只报告本次 API/文件系统/时间点的元数据，不推断最新一份足够保留 | 不启清理/维护任务、冷热重建、压缩或硬链接改写；全部缓存用途与冻结证据关系另核后才能形成维护候选。纯测量不重复无关工程全链 |

**接续**：新证据根独立且只写 create-new 物件；旧日志、失败、配置和已测 CLI 字节保持。目标 SHA 的远端 CI 与本地元数据结果分列；如果父 CI 尚在运行，先分类提交再批量同步，避免取消父目标验收。

### 缓存用途与保留准则（2026-09-28；元数据接续）

**尺寸 / 范围**：M，base `2da472cae97638af9cd4141bf4001f9b3352dfb4`，开场干净。依据上一轮逐项元数据账本，分组核缓存用途、所选已完成 Cargo 构建的显式引用和历史保护清单；不从“未被本次构建引用”推出无用。保留分为冻结证据、当前明确消费者、重建辅助及未判明四类；硬链接跨组按对象登记，不将各组去重量再次相加为总量。

**实际关联**：旧保护头 34 的 `frozen_host_binary` 引用日常 target 的 server；当前字节已更新，但 `cp-int-b10-tools/frozen-oclive-kernel-server-46044200.exe` 有匹配的独立原件。本轮只登记旧路径身份与独立原件的区别，不回改旧保护头或声称旧 checker 对当前 HEAD 已绿。`scripts/lib/e2e-binary.mjs` 仍消费 target 的 debug/release 程序，命名路径和 fallback 不是无用证明。两份 Windows 人类文档的 LNK1104/旧 target 清理建议同步为先核文件、当前占用、引用与证据，再处理具体范围；不要求全目录 clean、按名杀进程或设置安全软件全目录豁免。

**有界验收**：只读既有账本（核 hash）、两个明确 Cargo stdout artifact 集合和从头 34 `additional_protection_sources` 递归可达的 JSON；单文件 ≤4 MiB、累计 ≤64 MiB、60 s，遇缺件/越界/reparse/预算止点记 Partial。未知对象保留，解析历史引用不等于重验整条保护链。原件副本定点 bytes/hash 核实；台账/事件/Wave 与中英文人类建议同轮更新，跑默认/改文链接、旧路径、文档登记、镜像、债结构、编码及 diff。本轮不删除、压缩或移动现有产物，不创建清理自动化/新 CI gate；K-BUILD-07 保持 OPEN。

### 单 Host 末端 linker 候选对照（2026-09-28；独立实验）

**尺寸 / 范围**：M 级 A/B 试验，同 base，保持源码、默认 Cargo 配置、profile、依赖、`-j 1` 和父环境。本机 Rust 1.97.1 随带 `rust-lld` 22.1.6 可执行；前置 `msvc-lld --version` 探针 exit 0 只证明版本出口，下方补订记录真实编译的拒绝与修正。旧 response-file 的临时输入 258 项已消失，不能直接重放旧参数。新对照用 `cargo rustc --locked --offline -p oclive_kernel_host --features tauri-commands --profile test --test a_turn_harness -j 1 --message-format=json`：A 无额外 codegen 参数，初版 B 只向**末端测试目标**附加 `-- -C linker=<本机 rust-lld 绝对路径> -C linker-flavor=msvc-lld`。不以新的 package 选择假称同于先前 workspace 命令；只比较本组 A/B。

| 阶段 | 验收 / 原件 | 预算与止点 |
|------|-------------|------------|
| 输入与观察器 | 冻结 12 份构建/已验 CLI 字节、当前 server 及历史独立副本身份；采样所持 Cargo 子树，绑定实际 linker `/OUT` 与 Cargo executable。声明 Git 清单提交溯源会导致 A 的正常重编，不能把 A 总墙钟当纯热对照 | 编译器已存活/限定环境残留/源配置漂移则不开始；旧件不覆盖，无新安装或全局环境修改 |
| A → B 各一次 | 每次独立 stdio/native/采样；A 成功且输入仍一致才跑 B，验证 B 的非末端 Cargo artifact features/profile/文件集合与 A 相同且没有依赖重编，否则仅报混合结果。PDB 与 executable 各存独立副本，当前 server 与旧冻结副本必须不变 | 每次最多 480 s，空闲 RAM <4 GiB 或所持树 private >12 GiB 停该子树；输出副本累计 ≤1.5 GiB，不扩预算或自动重跑 |
| 末端行为 / 出口 | 对两份本批 binary 先 list 后仅默认非 ignored 测试，计数须与冻结源码对应；不启 parent 场景、真实模型/网络/语音，不消费 CP-INT 身份。非零、artifact 比较不等或输出归属缺失先按实际首因登记 | 每份默认执行最多 90 s、独立 child OCLIVE_HOME；默认点采样仍非真峰值。单目标通过不授权全局采用 linker、取消串行或关闭 K-BUILD-06；是否采用另看等价范围与取舍 |

**适用门禁 / CI 节奏**：无生产源码/公开契约/配置或门禁组合改变，适用本轮文档/镜像/债结构/编码/diff 与输入身份；不重复无关全仓业务链。两切片可分开本地提交，等正在运行的目标 CI 终态再同步；保留各尝试失败，不做证据专用推送，父债状态保持。

**参数兼容补订（首次 B 已停止）**：A native 0 后，B 在 0.51 s 以 101 退出，编译器明确拒绝不稳定 `msvc-lld`，未进入 linker。前置 `--version` 不能证明真实编译接受该值。按 [rustc 稳定接口](https://doc.rust-lang.org/rustc/codegen-options/index.html#linker-flavor) 改用 `lld-link`，先在新根用最小 `--emit metadata` 编译验证“稳定值成功/旧值与非法值非零”，再仅允许一个 **candidate-02 / 120 s** 末端候选；不重跑 A、复用日志或开启 unstable/nightly/BOOTSTRAP，不继续扩预算。原 A、失败 B、S0 与观察器保持，补订输入独立冻结；其余源码/依赖比较、内存止点、副本总预算和两份默认测试合同原样适用。此为已定位的一次性参数兼容修正，不把旧 B 改写为通过。

**当前计数核对**：实际两份 binary 的 list 逐行相等为 **94 tests**，各默认 **75 passed / 19 ignored**，Host跟踪源码匹配固定 base。历史 B3的89/70不是本轮预期；辅助读数与编排失误在 Wave追加登记，未运行 ignored场景或重用业务身份。最终默认配置保持，后续采用仍需明确目标/调试检查，不由本合同自动升为全仓默认。

### 已冻结 Host PDB 的离线消费（2026-09-29）

**尺寸 / 范围**：M，开场 HEAD `c60a04c99459025f7279a923676b1397fa9c8ba6`、工作树干净，前批同 SHA CI 17/17 成功。直接消费上一节固定 base `2da472ca` 的 MSVC / LLD 独立 exe/PDB；先核 hash 与 Host 源码未变，不运行 Cargo、不执行这些 exe、不附加产品/用户进程。有限的 PATH、Build Tools、SDK、VS Code 扩展和 Rust bin 盘点未找到 CDB/LLDB；使用已有系统 `DbgHelp.dll`，此盘点不等于整机无调试工具。

**合同与预算**：独立忽略证据根 `.cursor/plans/debt-pdb-symbol-20260929-r0/`，原件只读、输出 create-new。64 位受管 helper 串行调用同一系统 DbgHelp，`SymInitialize` 不枚举进程模块；仅指定本批本地搜索根，禁环境符号路径、提示、符号服务与错配接受。正例核实际加载 PDB 的路径/身份/匹配状态、真实 `http_idempotency::dto_json` 函数地址及源码行；缺失和错配 PDB 必须不能通过同一验收，不能回落原缓存或公网符号。若系统 API/ABI 不足，保留失败、不用“有 PDB”替代通过。每个 helper ≤60 s，最多一次有明确首因的修补另开 attempt；新隔离副本累计 ≤700 MiB，负控不改原件，不安装工具或扩预算。

**出口与边界**：保留每次原生退出、stdio、解析事实和输入收尾 hash；只接受所测系统消费者/两份固定产物的符号与行定位。静态符号 API 不证明断点、局部变量、调用栈或完整调试器等价，也不授权全局 LLD、profile/并发修改或关闭 K-BUILD-06/07。Wave、台账、变动事件同轮更新；适用默认/四篇改文链接、旧路径、登记、债结构、编码与 diff，不扩跑已验默认测试/ignored 场景。若下一层需安装调试器或改变默认构建，先提出具体范围再询问。

### K-SUPPLY-10 · 仓库 Actions 固定引用（2026-09-29）

**尺寸 / 目标**：L 级供应链收敛，base `e71e1c5fcd48fdb931a8e7c110bcafda4113af83`，开场干净；该提交 CI 尚在运行，先独立取证，不取消父验证。只把仓库 workflow 中已有外部 action 固定为其原上游对应提交，并补已有 Dependabot 的 `github-actions` 维护；不是 action 主版本升级、编译器版本锁定或发布执行。唯一状态 owner 为主台账，源码事实与来源写本轮 Wave；Done 需冻结目标 SHA 的正式 CI 和全部适用证据。

**写集 / 影响链**：`.github/workflows/{ci,cargo-audit-lockfile,nightly-advisory,release-kernel-checksums}.yml`、`.github/dependabot.yml`；盘点确认 `kernel/crates/oclive-cli/src/ci_cmd.rs` 的实际生成器也含 tag，纳入同一修补及其同函数合同测试，保留模板现有 v4/v2，不顺手升至主仓 v7。关联主台账、本计划、DEBT_CHANGELOG、供应链中英文和 collection 内新 Wave（入口登记）。只读核全部工作流、外部 action 元数据及官方说明。tag 必须逐层解引用到 commit，来源须为原上游、40 hex；不能凭记忆或搜索结果抄 SHA。`dtolnay/rust-toolchain` 按上游要求选 master 历史提交并显式 `toolchain: stable`，不固定会被回收的独立 stable 分支提交；其原 targets/components 保持。

**安全与验收**：逐个来源冻结 metadata/commit 链，静态解析前后 YAML 并核只有允许的 uses 与等价输入变化；既有 `on`/permissions/needs/env/run/if/runner/输出/密钥引用、CI 选择及 fail-safe 不改。来源失败/歧义、陌生 action、需要权限/安全边界改变即止。每次网络只读查询 ≤30 s、累计查询启动 ≤180 s（不在止点后启动下一条），原始结果 create-new，不安装工具或运行外部 action 本身；使用已安装解析器取证，不把它变成未声明工程依赖。固定的是仓库直接 action 代码，不扩大为镜像、runner、编译器或 action 下载的全部传递内容已可信。

**验证 / CI 节奏**：来源与结构正负控、已有 CI 执行策略/Compare/shadow 合同、`dimension5 --ci`；因全链 CI 依赖改变，里程碑跑 `check:ci-local` 并保留原生结果，未改 lock/API 不重复无关迁移或真实模型/语音/CP-INT 场景。文档/镜像/债结构/编码/diff 同轮验证；失败先定点归因，不放宽 gates 或更换 major。全部本地验证后分类冻结一次，等父 CI 终态再推送并等待新目标正式 CI。远端成功前仅 Locally verified；不为 run ID/Done 文字再造证据专用提交，结案候选先入交付报告，随后续实质变更同步状态。

**实施发现 / 限定接续**：已解析全部 5 个实际 workflow 和 CLI 生产模板；72+14 处引用对应 10 对上游/ref。当前 master 的 Rust Action 多了两个安装参数，改选仍可达的等价祖先 `f3510ffd`（除 required/default 的显式 stable 输入适配外，metadata 与原 stable 完全相同），不引入该行为变化。一次 metadata 查询 EOF 原样保留，仅新输出重试一次成功。结构核验的 9 个真实变异均被同一核验函数拒绝。CLI 模板是 Rust 字符串，Dependabot 不自动扫描它；维护规则明确人工同步，未建立新更新器或复制工作流，父债保留 Partial。各 native 窄门禁 ≤480 s、全 `check:ci-local` 单次 ≤1200 s，child offline/-j 1；deadline 非零保留，不删除缓存或重跑真实业务。细节只在 [Actions Wave](waves/WAVE-20260929-ACTIONS-PINS.md) 维护。

**全链首因修正（attempt1 非零）**：第一轮本地全链到 `oclive_ci_plan` 的 `repository_contract` 才暴露旧 `actions/download-artifact@v7` 精确文字断言。将该真实消费测试文件追加到写集，仅把断言改为所选原上游 SHA 与保留版本注释，不删除测试或放宽执行策略；定向 5/5 通过后用新原始日志跑第二轮全链。另 `git diff --check` 检出 `ci.yml` 既有混合行尾的改行。曾整文件 LF 化导致无关差量，随后按与 HEAD 的逐行匹配保留 581 条未改行的原换行，仅令 64 条实质变更行使用 LF；解析树与标准化全文相等、最终 diff 仍只有 64+/48−，不改其它 workflow。两失败件原样保留。

**格式止点（attempt2 非零）**：第二轮 `check:ci-local` 在 `cargo fmt --check` 截止，所增 `repository_contract` 断言需 rustfmt 换行，尚未进入本轮 Rust 测试。仅格式化这一测试文件、`cargo fmt --all -- --check` 新 attempt native 0；不修改实际断言或生产 Action 引用，另开第三轮全链，旧失败不覆盖。

**本地验收（attempt3）**：第三轮 `check:ci-local` 原生 0 / 744.91 s；Dimension 5 --ci、lint/typecheck/build、Rust fmt/clippy/lib、workspace 与 CLI 集成均按原链通过。源及文档输入收尾哈希 28/28 匹配、7 文 UTF-8 无 BOM/替换字符、9 结构负控与 default/改文链接均独立通过。之后只补充验证结果文字，需再复核文档 ratchet 与最后字节；父 `e71e1c5f` CI 当前仍在进行，不先推送新目标。

### D-DEBT-LEDGER-01 · 重复状态改为显式引用（2026-09-29）

**尺寸 / 目标**：M，base `245d6ca98edfa8ebb354e2609d556a455336cc73`、开场干净。只整理主台账中本轮逐行确认的 18 个重复 ID：保留已有当前裁定，给权威行显式锚点，其余行标为引用或历史引用。初次临时读取漏掉 O-1/O-2，扩大到实际 ID 格式后补齐，并保留读取层更正。冻结/Observe/Deferred 和 Minimal/Full 范围不得靠首次字符串命中重判；历史 Verification 原文不改，不以文档整理关闭父债。

**写集 / 关联**：主台账、本计划、DEBT_CHANGELOG 的接手协议与事件、COVERAGE 的治理缺口，以及 collection 内一个 Wave 和入口登记。§1 已有状态行优先；供应链独有条目仍归 §1.5，MEGA 与 D-PORT-03 的现行解冻条件仍归 §2。历史归档不维护第二份当前状态。原行先存本机独立忽略根 `.cursor/plans/debt-ledger-refs-20260929-r0/`；禁止改旧候选、原始证据、QUEUE、机器计划或生产/门禁源码。遇到不能从现行 owner 和冻结条件消解的语义冲突，只暂停相应条目并问维护者。

**验收 / CI 节奏**：独立核对原/新 ID 集合、每个重复 ID 恰好一个权威行、引用锚点存在且唯一、历史 Verification 逐字保留；未触及的状态/条件行须不变。默认及本轮改文链接、docs-only 旧路径、文档登记、债计划结构、UTF-8 与 diff 全部适用；纯文档不重复 Rust/业务场景。静态核对仅为本轮证据，不冒充新增持续自动门禁。先本地收口并分类提交，前批 Actions 目标 CI 终态后统一推送；目标 SHA 的远端结果独立绑定，不造仅回写绿灯的提交。D-DEBT-LEDGER-01 保留 Partial，长历史迁移和持续冲突检查仍未完成。

### D-DEBT-LEDGER-01 · 七月 Verification 迁移（2026-09-29）

**尺寸 / 边界**：M，base `5cd50d346f5fab351357af388f8e43f110b65696`、开场干净，上一节引用化已完成本地提交。只迁移主台账首部 20 个 `Verification (2026-07-…)` 段到 `handoff/archive/TECHNICAL_DEBT_VERIFICATIONS_202607.md`，主表保留明确的历史链接。归档不是当前 truth；历史文本除三个相对地址的目录重定位外必须逐字相同，旧 CI/SHA/状态/范围不改。首部其它工程长快照、§5 轮次表与全表状态词不在本轮搬移范围，不因局部迁移关闭父债。

**写集 / 验收**：主台账、本计划、DEBT_CHANGELOG、COVERAGE、同一台账 Wave、入口登记及一个 archive 文件。独立证据根 `.cursor/plans/debt-ledger-history-20260929-r0/` 保存原文、迁移映射、native 日志和最终身份。全表 ID/引用/锚点不变，除治理父债的进展文字外状态行原样；20 段顺序、内容与所有外部 URL 保留，归档内相对链接精确指回原目标。出现来源混淆、证据缺失或无法机械重定位的链接即止，不改旧证据/生产/QUEUE。适用默认及七篇改文链接、docs-only 路径、登记、债结构、编码/diff；仅文档不重复 Rust/业务链。分类本地提交，等 Actions 目标 CI 终态后按既有授权一次同步本轮里程碑，新目标独立取证。

### D-DEBT-LEDGER-01 · 持续登记结构检查（2026-09-29）

**尺寸 / 基线**：M，base `b3b2fef5774ff84985b8ccea705c9b302b55e41e`、开场干净，三篇文档提交已保全到独立分支；前批 Actions CI 仍待 Windows Rust 终态。本轮补登记结构门禁，不改产品、状态裁定、解冻条件或旧证据，不把治理父债转 Done。适用 G3/G11/G12/G14/G16/G17 与数字/编码核实；关联产品调用链无变化，Kernel/Tauri/shared 不需同步。

**合同 / 写集**：在 `scripts/lib/debt-ledger.mjs` 定义同一纯检查函数，单独 CLI、内存反例与既有 `check-debt-marathon.mjs` 均调用它；Dimension 5 原债计划步骤加入该函数的 Node 回归。只读核主台账 ID/子 ID 表的独立 ASCII ID、唯一当前登记、稳定锚点、引用标记与目标、状态列的转引格式。§5 中仅历史出现的 ID 不算当前 owner；已有当前行的历史重复须显式引用。历史说明可提旧状态，引用不是第二份状态声明。不解释长段落语义、组合 alias、Minimal/Full 是否实际完成，也不执行债计划。格式异常须非零，不能默默跳过 ID。主台账、接手协议、COVERAGE、同一 Wave 与入口登记同轮写实。

**验收 / 止点**：本机独立忽略根 `.cursor/plans/debt-ledger-check-20260929-r0/` 先保全九份输入及归档原字节。反例经同一实际函数，至少覆盖重复当前行、坏/缺/跨 ID 锚点、历史引用错位、引用另写状态、历史-only 不被算当前、O-1/O-2 与子 ID、空/坏表格与围栏边界；CLI 原生非零也验证。实际主表正例不得靠写死计数跳过；旧状态与历史归档须不变，QUEUE 和机器计划不动。运行定向回归、默认/改文链接、旧路径、登记、债结构、编码/diff 及门禁脚本适用的 `dimension5 --ci`，单次全链 ≤1200 s、child offline/-j 1；不重复 Rust/业务场景。异常逐次保留原日志；只有需要改变 owner 裁定或验收范围时询问维护者。

**CI 节奏 / 续跑**：先窄测再最终字节 Dimension 5，适用检查通过后分类提交。父 Actions CI 终态前不推 main 以免取消它；必要时新保全分支保留里程碑。同步后独立绑定新 SHA 的正式 CI，不能借父绿。持续结构检查通过仍只使 D-DEBT-LEDGER-01 保留 Partial，状态词与其余长历史另定切片。

### D-DEBT-LEDGER-01 · 八月工程快照归档（2026-09-29）

**尺寸 / 边界**：M，base `2852b285f6a0b33dbd73aba63037ac6c5f85680a`、开场干净。只将主台账首部第 12–24 行的连续历史块迁至 `handoff/archive/TECHNICAL_DEBT_CLOSEOUT_SNAPSHOTS_202608.md`，原位留下带日期与非当前 truth 边界的链接。历史里的旧 Done、CI、产品冻结与旧排期不能转成当前状态或开工许可；§1–§5、其它首部说明和七月归档不在本轮搬移范围。

**写集 / 保全**：主台账、本计划、DEBT_CHANGELOG、COVERAGE、同一台账 Wave、handoff 入口登记及一个新归档。本机独立忽略根 `.cursor/plans/debt-ledger-aug-history-20260929-r0/` 在编辑前保全七份原文与 HEAD。归档块原文顺序、日期、旧 SHA/CI、外部 URL 和文字除三个相对链接目录重定位外保持；重定位前后必须解析到同一现存文件，逆向重定位后正文逐字节匹配 S0。其余主台账行及既有 ID/引用/状态字段不变，QUEUE、机器计划、产品和旧证据不改。

**验收 / CI 节奏**：独立迁移审计、默认/改文 Markdown 链接、docs-only 旧路径、文档登记、债结构、UTF-8 与包含新归档的 staged diff 全部适用；纯文档不重跑 Rust/业务场景。先完成本地分类提交；父 `2852b285` 的正式 CI 终态前不推 main，随后按新目标 SHA 独立取证。结构与历史归档只使 D-DEBT-LEDGER-01 继续 Partial；其它长快照、全表状态词和自由文本语义尚未治理。

### D-DEBT-LEDGER-01 · 六月轮次 16–19 历史表归档（2026-10-02）

**范围 / 基线**：M 级文档切片，base `53479d8cbba6bffe92213dc65fde61fefac24882`、开场工作树干净。主台账 §5 的轮次 16–19 是 2026 年 6 月完成历史，已有部分行写成指向 §1 权威状态的历史引用；迁至 `handoff/archive/TECHNICAL_DEBT_ROUNDS_202606.md`，主表原位保留明确标注“历史、非现行状态”的单一链接。§1–§4、速查坐标、轮次 1–15 提示、旧归档和所有债务当前状态不在迁移范围。

**保真合同 / 写集**：从 `### 轮次 16 Done` 到“轮次 1–15 明细表”之前的四张表原顺序保留，只有九条 `(#debt-…)` 相对锚点改为 `../TECHNICAL_DEBT_INVENTORY.md#debt-…`；K-SUPPLY-DOC-01 行的“本文件 §1.5”作为原始历史措辞留存，归档页头说明其来源。迁移脚本先冻结原文和 SHA，再反向重定位比较全部表格正文；归档末尾仅去掉一个多余空行，核原/新 ID 行及四轮标题一致、锚点仍解析到当前权威行。写集为主台账、新归档、本计划、台账 Wave、DEBT_CHANGELOG、COVERAGE、handoff 入口和 archive 索引；不改产品、脚本、QUEUE、机器计划、旧证据或债务状态。reqwest 暂缓只追加到原计划/事件，单独对账。

**出口**：迁移核验、`node scripts/check-debt-ledger.mjs`、默认及改文 Markdown 链接、docs-only stale paths、文档登记、`npm run check:debt-marathon`、中文 UTF-8 与暂存差量均须通过；仅文档迁移不重跑业务与 Rust。当前本地提交 `53479d8c` 尚未推送；按用户既有里程碑授权，完成本切片后分类本地提交并决定是否合并推送，目标 SHA CI 与此前 `053ebdad` 的绿灯分列。D-DEBT-LEDGER-01 仍 Partial，持续门禁已覆盖结构但不能证明自由文本语义，其他长快照与全表状态词仍待处理。

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

## K-CI-IMPACT-01 · 失败作业重跑的计划证据绑定（2026-10-01）

**尺寸 L / base**：`e0afbf4b6f0ea35368114207e492248c3ae60494`，开场工作树干净。此切片修主分支 CI 门禁的已发生缺陷，归属与原生失败见 [DCL-20261001-01](DEBT_CHANGELOG.md#dcl-20261001-01--文档声明支撑调查首批入账)；不扩展 K-CI-IMPACT-01 的选择性执行范围，也不把父债标 Done。前一提交的远端 CI 尚在执行时可本地实施与验证，不推送新 main 提交去取消它。

**预期语义**：同一 workflow run 的 `ci-gate` 必须下载该次成功 `ci-impact-plan` 作业实际上传的计划，而不是猜测汇总作业的当前 attempt。`gh run rerun --failed` 没重跑计划作业时复用旧 attempt 的精确计划；计划作业若重跑则使用其新 attempt。产物缺失、计划作业失败或身份输出缺失继续 fail closed；不使用模糊 artifact 匹配、跳过验证或放宽受信规划器。GitHub 失败作业重跑保持原 `GITHUB_SHA`；此前原生 attempt 2 的计划作业输出在 `needs` 中仍为 success，唯产物名误用 `-2`。

**写集与验收**：限定 `.github/workflows/ci.yml` 的计划作业输出/上传和 gate 下载坐标、`oclive_ci_plan` 的仓库工作流契约回归、必要的本计划与技术债变动事件。先用定向测试证明旧 `github.run_attempt` 查找会被拒，再跑 `oclive_ci_plan` 合同、CI 策略自测与 `dimension5 --ci`；集成门禁改动再跑 `check:ci-local`。冻结后一次推送，以目标完整 SHA 的 `ci.yml` 正式结果验证常规路径。若无法在同一 SHA 构造真实失败作业重跑，则仅记“重跑分支尚缺原生验证”，不得凭静态合同和普通绿灯关闭该缺陷。旧运行、artifact 与日志保持原样。

### K-SUPPLY-10 · rust-toolchain 模板同步合同（2026-10-01）

**尺寸 / 路由 / 基线**：M 级维护切片；重风险规划已裁定限定 `dtolnay/rust-toolchain` 同步语义。Luna 起草后遇模型额度限制；维护者明确自适应流程不限模型，主控接续实施与复核。base `c2dd6112c60f0aed6e0687b5d3654f863e8266f8`，开场 `main` 干净，既有精确 SHA CI `36765997872` 为 17/17 success；不恢复 main、不迁父债 Done、不升级依赖、不改生产 workflow/生成内容或 CI 编排，K-SUPPLY-09 签名暂缓。真实 Dependabot PR #184 只更新 workflow 的 Rust Action SHA，CLI 五处模板可漏同步；本切片只增加 test-only 合同检测该风险。

**写集 / 闭环**：先改本文件；随后只改 `kernel/crates/oclive-cli/src/ci_cmd.rs` 的 `#[cfg(test)]`，复用 `serde_yaml_ng` 与真实 `render_ci_yaml`；再追加本轮 Wave、`DEBT_CHANGELOG.md` 的 DCL-20261001-04、供应链中英文 §5 第 6 项一句同步合同，并按需给主台账 K-SUPPLY-10 加简短进展。生产 workflow、Cargo/lock、环境、README/CHANGELOG、生成字节均不改。影响链为仓库 workflow Action 引用 → 生成器模板合同 → CLI 渲染结果 → 测试与文档维护说明；无公共 API、宿主、权限或运行时消费者变化，生产字节应保持不变。

**合同与反例**：从 `CARGO_MANIFEST_DIR` 向上三层读取 `.github/workflows/*.yml` 与 `*.yaml`，缺文件或仓库总体不含 `dtolnay/rust-toolchain` 时失败而不 skip。逐 workflow 解析；含该 Action 的每处引用必须为 40 位十六进制、显式 `with.toolchain: stable`，仓库内唯一引用 SHA 必须一致；不含该 Action 的 workflow 合法。两类真实 `render_ci_yaml` 模板均必须含该 Action、所有 SHA 等于仓库唯一 SHA 且保留 stable。内存 YAML 变异通过同一检查函数验证 bot-only 更新到 PR #184 新 SHA、只改一处仓库 pin、以及模板侧漂移均拒绝；缺 action、短 SHA、缺 stable 仅做必要覆盖，不改工作树反例。

**验收 / 停止 / 收口**：先跑 `cargo test --locked -p oclive-cli ci_cmd::tests -- --test-threads=1`，再按串行约束跑 `cargo fmt --all -- --check`、`cargo clippy --locked -p oclive-cli --all-targets -- -D warnings`、`cargo test --locked -p oclive-cli -- --test-threads=1`、`node scripts/dimension5-acceptance.mjs --ci`（临时子进程使用既有 portable Python，不改全局）；随后跑默认/改文链接、docs-only stale paths、doc registry/mirror、`npm run check:debt-marathon`、`git diff --check` 与 UTF-8、无 BOM、无乱码问号串核对。失败与成功日志分存 `.cursor/plans/debt-toolchain-sync-20261001-r0/`，历史证据不改。保持 K-SUPPLY-10 `Partial`；GPT6 复核真实 diff、写集、正负控、生成字节未变与命令证据后，controller 再按授权提交/推送。

### K-SUPPLY-10 · rust-toolchain 实际升级与模板同步（2026-10-01）

**范围 / 基线**：接续同步合同提交 `ed6f6457e92a26aa3a41c5a1a96e4771c0d9254c` 与自适应流程文档提交 `e3030a9f3d04726381e47c810bed5e39f883904f`；开场工作树干净。真实 Dependabot PR #184 仅把四份 workflow 中 14 处 Rust Action pin 从 `f3510ffd6ce03d3e6f96856b0b93d5dc6c2e683f` 更新为 `02cb101ec7c40f2c49e1d9714d64511d8e1b74de`，没有同步 CLI 生成模板。本切片在当前主线做同等 14 处更新，并同步 CLI 五处生产模板；PR 仅作来源，不直接合并、修改或关闭其旧分支。K-SUPPLY-09 信任根决策继续暂缓。

**风险与边界**：上游 compare 显示 7 个提交，`action.yml` 的执行差量是 `rustup toolchain install` 与 `rustup default` 各增 `--force-non-host`；当前 workflow/模板的 19 处均显式 `toolchain: stable`，未通过 Action 选择非宿主工具链或组件/目标。判定为当前已声明路径的兼容更新，不能称 Action 完全等价，也不能外推所有 runner、nightly/release 或新非宿主输入。若 PR head、上游差量或输入语义漂移，或需修改权限、失败策略与工具链选择，暂停本切片重新规划。

**写集 / 验收**：生产写集严格为 `.github/workflows/{ci,nightly-advisory,cargo-audit-lockfile,release-kernel-checksums}.yml` 及 `kernel/crates/oclive-cli/src/ci_cmd.rs` 的五处模板字面量，只替换上述 40 位 SHA；保留 `ci.yml` 原行尾、所有 stable、其它 Action、事件、权限、runner、脚本、Cargo.lock 与测试逻辑。文档仅更新本计划、Actions Wave、`DEBT_CHANGELOG.md` 与主台账 K-SUPPLY-10。先核文本与解析后的 YAML 差量只有预期 uses，跑现有 CLI 正负合同、fmt 与 diff；工具链影响整个 CI，再跑一次 `npm run check:ci-local` 及适用文档/债务门禁，不重复不受影响的真实业务场景。独立复核后，等待父提交 `ed6f6457` 的正式 CI 终态再推新主线；新目标 SHA 的正式 CI 含 Windows/Linux 与 `ci-gate` 成功前仅记 Locally verified。父债仍 Partial：本次仅证明一个实际升级的人工同步闭环，不宣称所有未来 Action 或模板自动更新。

### K-ENCODING-01 · 活跃文档编码防回退（2026-10-02）

**尺寸 / 基线 / 边界**：L，起点 `d7040c37cc95966764318a3f8aa6481c9a831b1e`、工作树干净且该 SHA 的主 CI 17/17 通过。此切片只保护 `handoff/`、`creator-docs/`、`human-docs/` 的活跃 Markdown；`archive/` 是历史原件，不改写也不作现行门禁输入。现有 307 份活跃文档在 UTF-8、无 BOM、无替换字符及连续三问号四项上均通过。CLI、产品源码、JSON、外部工作目录及历史证据不在写集。

**目标 / 写集**：新增只读编码检查脚本与真实文件正负控，将其接入 Dimension 5；在本计划、K-ENCODING-01 权威行、GATES §7、DEBT_CHANGELOG 和本轮 Wave 记录可检测范围。检查必须拒绝无效 UTF-8、UTF-8 BOM、U+FFFD 与连续三问号，输出具体相对路径和原因；不以“有汉字”作为所有文档的硬门槛，因为英文文档有效。不得自动修复或猜测受损中文。门禁对现有活跃文档全量运行，避免按变更集漏掉被整体损坏的文件。

**验收 / 止点**：先以临时普通文件验证四种拒绝与正常中英文、归档不参与默认扫描，再跑默认检查、Dimension 5、`check:ci-local`、适用文档/债务门禁、编码与 diff；冻结并推送后以目标完整 SHA 的正式 CI 验收。此切片无法从纯 ASCII 文件识别每一处一至两个问号替换，也不覆盖 JSON/历史或外部目录；父债仍 OPEN，写后人工逐文件汉字数检查继续有效。若现存文档出现历史例外，先核来源，不能静默加白名单或改写原件。

### D-DEBT-LEDGER-01 · Event Stream 长历史出权威行（2026-10-02）

**尺寸 / 基线 / 原因**：M，起点 `5f2836ce67a8103882a383765d5a01913ccbb365`、工作树干净，目标 SHA 的主 CI [36971687198](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/36971687198) 17/17 成功。`K-EVENT-STREAM-01` 权威行约 2,571 字符，R1–R6 探针流水账与当前生产缺口同格；接手者可能把 R5/R6 合成证据或 B0 Trace-only 误认作已交付 Stream。本切片仅清理状态入口，不重新验证探针或改变 RFC 结论。

**写集 / 不变量**：把修改前的完整 `K-EVENT-STREAM-01` 表格行作为带源 SHA 的原文快照移入 `handoff/archive/`；主台账保留同一 ID、问题、优先级、完成条件和 OPEN 状态，仅将状态格改成短的当前能力／未完成范围／快照链接。同步本计划、D-DEBT-LEDGER-01 现行行、DEBT_CHANGELOG 和 Wave。旧 RFC、代码、测试、其他债务状态、队列和机器计划不动。新主表不得暗示 R1–R4 真实样本能代替 Production、R5/R6 已接真实 QQ 或 A.2.2.2 已完成。

**验收 / 止点**：机械比对新归档中的旧表格行与起点提交原行逐字一致；新主表除该行及治理父债行外不变，且该行前四格与起点相同。适用改文及默认链接、docs-only 旧路径、文档登记、债结构、活跃文档编码、暂存 diff 必须通过。纯文档切片不重跑 Rust、QQ、B0 或 ignored 场景；目标提交 CI 与上一提交分列。D-DEBT-LEDGER-01 保持 Partial，其他长行与自由文本语义冲突不在本轮。

### K-LLM-ENV-02 · DB 读取交错的隔离回归（2026-10-02）

**尺寸 / 起点 / 目的**：M，基于 `19293fe8c66dbbdccb6f9d823d22736e16bb67a6`；上一文档提交的目标 CI 仍在运行，推送本切片前须先核其终态。现有修复以进程级异步锁包住 DB 读取、缓存和环境提交，已有原子版本测试；缺一项真正阻塞旧 DB 快照再并发应用新快照的行为证据。此切片仅补独立 Rust 集成测试和本计划／Wave／台账／变动记录，不改生产实现或用户设置。

**受控场景**：内存 `AppSettingsPort` 第一次读取时捕获旧 `OLLAMA_BASE_URL` 并在闸门等待；测试将设置更新为新值，放行启动屏障上的 32 个新 `apply_user_llm_env_from_db` 调用，确认第一次未释放前后续调用均未读 DB；释放后 33 次调用均成功，最终进程环境是新值。测试单文件单用例独立进程，无真实 DB、凭据、网络或模型；记录原环境并在退出时恢复。该场景检验 DB→env 串行化，不能据此外推完整 `AppState` 版本/dirty、save/chat/theater/canonical sync 或跨进程共享环境。

**验收 / 止点**：先定向运行该集成测试，再跑 Host lib/受影响集成、fmt、Clippy、分层和适用文档/债务门禁；代码触及 Rust 行为验证，里程碑跑 `check:ci-local`，推送后以目标 SHA 的远端 CI 验收。若测得环境变量竞争、测试进程未隔离或所需改动进入产品语义，停止该切片重新定界。K-LLM-ENV-02 仍维持 `Remote verified · stress pending`，只记这项可控交错子证据，不因一次测试升 Done。

### K-LLM-ENV-02 · AppState 刷新入口的真实内存 DB 回归（2026-10-03）

**尺寸 / 起点 / 目的**：M，基于已远端验证的 `b912237324fd9521968084cbf25f6c7dd267cc6e`、干净工作树。上一切片证明同进程内 DB 读取到环境写入受串行锁保护，但内存设置替身不经过完整 `AppState`。本切片只补一项隔离的 Host 集成测试，用真实内存 SQLite、注入的 `MockLlmClient` 和临时角色目录验证版本/dirty 快路径及显式刷新入口；不改生产应用、真实用户库、凭据、模型或网络。

**受控场景 / 停止线**：建状态后先写入本地 provider 与旧 base URL，调用 `reload_llm_user_env_impl` 应把 DB 值应用到环境；直接将 DB 改为新值但不标 dirty，再调用 `apply_user_llm_env` 应保持旧环境；显式标 dirty 后再调用，应应用新值。独立集成测试进程保存并恢复七个环境变量。若状态构建要求真实 provider、用户数据或跨服务夹具，停止此切片并记录跳过，不扩大到 save/chat/Theater。该测试不声称覆盖并发版本竞争或所有调用入口。

**验收**：定向集成测试、Host lib、fmt、定向 Clippy 与分层；文档/台账变动跑已改链接、编码、债结构、diff。若构成里程碑，冻结后跑一次 `check:ci-local`，推送后只用新 SHA 的 `ci.yml` 判断远端。父债保留未结案，剩余真实入口/持久库/长时间压力明确列出。

### K-LLM-ENV-02 · 保存设置主路径接入刷新（2026-10-03）

**尺寸 / 起点 / 目的**：S，基于 `8e39f6b6269e0a7aba3d305c9353d1fa1e1936ed` 的 AppState 内存 SQLite 测试。上一切片证明直接写库不会自动刷新；本切片只在同一隔离测试中增加一个合成角色，调用生产 `save_llm_user_settings_impl` 保存第三个 URL，确认 DB 与进程环境均变为第三个值。仅核 `save settings` 主路径是否履行标脏/应用责任，不改生产语义。

**调查/验收止点**：只追保存入口、其直接标脏/应用调用和角色加载所需的最小测试夹具；不追次级失败、云端 token、LoRA 或所有调用者。先跑定向测试、fmt、Clippy、已改文档/债务门禁，再按 Rust 测试改动跑一次本地综合链；上一目标 SHA 的远端 CI 终态前不推送新提交，避免把取消当成测试结果。父债仍缺 chat/theater/canonical sync、持久库、并发版本与长时间压力证据。

### K-LLM-ENV-02 · 对话主入口消费待应用配置（2026-10-03）

**尺寸 / 起点 / 目的**：S，基于本地保存入口切片 `1a0acca1`。`process_message` 在生成前直接调用 `apply_user_llm_env`，但此前用例没有从完整 `AppState` 的 DB 值验证这条入口。复用同一隔离测试，在保存设置后直接写入第四个 URL、显式标脏、发起一个合成角色的模拟对话，核回复与环境值；不改生产代码。

**止点 / 验收**：只验证一条正常 chat 主路径和 Mock LLM，不追流式、断流、Theater、canonical sync、云端或真实模型。定向测试、fmt、Clippy、文档/债务检查及一次综合本地链通过后形成独立小提交；父债仍保持未结案。待前一个远端 SHA 终态后再按顺序同步，远端每个目标分别核验。

### K-LLM-ENV-02 · Theater 主入口消费待应用配置（2026-10-03）

**尺寸 / 基线 / 目标**：M 级测试与债务记录切片，基线 `5eec7b42e706d28d78eb4c63ae087020c562716c`，工作树干净。本次只沿 `generate_scene → apply_user_llm_env → Theater LLM` 核一条有合成角色、有效场景和内存 SQLite 的主路径。先在 DB 写入第五个本地 URL 并显式标脏，再验证模拟生成器看到该值；与既有 chat 第四个 URL 对照。生产锁、DB 代码、公共 DTO、角色包和真实模型均不改。

**写集 / 停止线**：只改 `tests/user_llm_env_state_refresh.rs`、本计划、同债 Wave、DEBT_CHANGELOG 和主台账 K-LLM-ENV-02 状态格。复用同一独立测试进程与临时角色目录；模拟生成器区分 chat 回复与 Theater 标签生成，并记录调用时环境，避免只证明函数返回后的副作用。若现有注入点不能让 Theater 走到模拟生成器，或需改生产装配/公共语义，记录缺口并跳过，不加测试专用生产钩子；不追 canonical sync、真实 provider、长时压力或其它模式。

**验收 / 节奏**：先跑该集成测试、fmt、定向 Clippy、分层及适用文档/债务门禁；本批先形成可审查本地提交，不为每个测试切片触发一次全量 CI。后续同主题切片稳定时再做一次适用本地完整门禁与目标 SHA 远端验收。Theater 的本条证据不关闭完整 AppState 并发版本、持久 DB、canonical sync、流式/断流和进程级压力缺口，父债保持未结案。

### K-LLM-ENV-02 · 桌面 canonical seed 的隔离文件库回归（2026-10-03）

**尺寸 / 目标**：M 级同债第二切片，接续上方未推送的 Theater 测试。生产 `seed_shell_llm_from_canonical` 从 canonical `app.db` 复制设置到 UI shell 并调用 `mark_user_llm_env_dirty` / `apply_user_llm_env`；此前仅有源码对应，没有在隔离文件库上核对其行为。本切片只验证 local provider 和 Ollama Base URL 从临时 canonical 文件库进入内存 shell，旧环境先被生产 reload 应用，新值在 seed 后进入 shell DB 与进程环境。

**写集 / 安全边界**：新建一个独立进程的 `distros/desktop-tauri/tests/canonical_llm_env_sync.rs`，只用临时 `OCLIVE_APP_DATA`、最小 `app_settings` 表、内存 `AppState` 和模拟 LLM；同步本计划、同债 Wave、变动事件及主台账状态。环境变量进入测试前保存、退出时恢复；所有 SQLite pool 显式关闭。不修改生产同步算法、桌面权限、公共 API、真实用户库、模型、网络或姊妹仓。若公共测试入口不可用或必须改变生产装配，则登记为未覆盖并停止该切片，不引入测试专用生产钩子。

**验收 / 停止线**：新增单测试、fmt、定向 Clippy、分层与适用文档/债务检查通过；与 Theater 切片共同形成同主题本地冻结点。首轮 `check:ci-local` 因测试夹具旧目录名被 Dimension 5 拒绝，定点修正后第二轮完整本地链 exit 0。只证明一条 local-provider seed 路径，不外推双向同步、失败重试、云端 token、并发写入或启动进程联机。目标 SHA 的正式远端 CI 待独立核验；父债仍不升 Done。

### K-LLM-ENV-02 · 流式对话调用时配置观察（2026-10-03）

**尺寸 / 起点 / 目的**：S 级定向测试，基线 `1e9cd8e0`。复用已隔离的 Host AppState、内存 SQLite、合成角色与模拟 LLM，仅在同一测试末尾写入第六个本地 URL 并标脏，调用生产 `process_message_stream`。模拟流式生成器在收到请求时记录进程 URL 并发出一段 token，以此核实流式主路径在生成前消费待应用设置。

**写集 / 停止线**：只改 `kernel/crates/oclive_kernel_host/tests/user_llm_env_state_refresh.rs`、本计划、同债 Wave、DEBT_CHANGELOG 与主台账 K-LLM-ENV-02 状态格。不改生产流式/SSE 协议、并发锁、DB schema、真实网络或模型；不扩成断流、fallback、HTTP/Tauri 联机用例。若正常流式入口不能稳定走到现有模拟生成端口，记录缺口并跳过，不加生产测试钩子。先定向测试、fmt、Clippy、分层及文档检查；合批收口才跑一次完整本地链。

### K-LLM-ENV-02 · 文件库重建后的本地设置刷新（2026-10-03）

**尺寸 / 目标**：S 级独立集成测试，接续流式定向测试。以临时 app-data 与合成空角色目录调用生产 `AppStateBuilder::production(...).with_llm_client(MockLlmClient)`，在第一份 AppState 写入 local provider 与 URL，关闭 pool 并释放状态；再用同一 SQLite 路径重建 AppState，执行公开刷新入口，断言落盘设置与进程环境一致。验证迁移后的文件库和重建路径，不使用用户数据库、真实模型或外部网络。

**写集 / 停止线**：仅新建 `kernel/crates/oclive_kernel_host/tests/user_llm_env_file_restart.rs` 并更新本计划、同债 Wave、DEBT_CHANGELOG 与主台账状态格。生产构造未要求额外外部服务或测试专用钩子；不追崩溃中断、跨进程、云端凭据、并发旧读取、完整桌面启动或压力。定向测试、fmt、Clippy、分层及文档检查均通过；与流式切片合批的 `check:ci-local` 原生 exit 0，目标 SHA 远端 CI 待核。
