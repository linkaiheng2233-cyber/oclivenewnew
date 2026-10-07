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

### 当前执行纠偏：结束 K-LLM-ENV-02 扩展补证（2026-10-03）

维护者明确本阶段目标是技术债处理，调查足以支撑处理即停。依据[实施停止规则](../AI_VERIFICATION_PROTOCOL.md#210-技术债实施足以修复与验收即停止补证)，K-LLM-ENV-02 的后续动作收敛为对原问题和有限完成条件办理验收；原事务修复、可控交错回归与现有入口证据足以停止当前扩展补证。保留已取得的实际证据及目标 SHA CI 待核状态，不再自动启动完整 AppState 竞争、长压、真实 provider、SSE/断流、全迁移或更多排列组合。前文各切片“未覆盖”仅描述当时证据边界，不再作为不断追加任务的清单。

下一项优先选已有原因和明确改法的实现债；缺决策或外部条件的先跳过，汇报具体阻塞。验证收益开始下降即停止该项，列明已知结论、剩余疑点与深入成本，由维护者确认是否深入；不得自行续预算或换名继续补证。新反例与实际风险如实报告，既定修复的必要回归照常完成。本次仅校准准则、安排与台账，不运行工程全量 CI 或新增业务测试。

### D-DEBT-LEDGER-01 · K-LLM-ENV-02 长状态行收敛（2026-10-03）

**问题 / 完成条件**：K-LLM-ENV-02 的当前状态格逐次累加实现和测试历史，当前裁定被埋在长段里。把修改前的整行原文保存在 `handoff/archive/` 并链接回当前行；当前行只保留问题修复、已有证据的有限范围、当前验收状态和下一动作。原债 ID、问题、优先级及完成条件逐字不变，保留 Partial，不重新判定实现或扩充验证。

**写集 / 止点**：仅主台账、新历史快照、本计划、台账 Wave 与 DEBT_CHANGELOG。原文从起点 `7b0b41a145847bfefcedd642f29b1e16d3ad52cc` 读取并保全；机械比对原行和快照、除治理父债行外其它主表行不变、结构/链接/编码/diff 检查通过即可停止。纯文档迁移不运行 Rust、真实模型或完整 CI；不顺手迁其它长行。K-DUAL-ROLLBACK-02 的冻结决策门及 K-UID-DEFAULT-02 的语义选择均跳过。

### K-ENCODING-01 · 官方角色 JSON 编码防回退（2026-10-03）

**尺寸 / 基线 / 目标**：M 级门禁切片，起点 `1b3680b9d4084357deeee0ceba0b40100503f05e`、工作树干净。现有检查仅默认覆盖三类活跃 Markdown；本轮将 Git 跟踪的 `distros/chat-pro/roles/` JSON 纳入同一只读编码检查。只防无效 UTF-8、BOM、U+FFFD 与连续三问号这四类已知字节异常，不以汉字数量或 JSON 语义推断内容正确。

**写集 / 隔离**：仅 `scripts/check-doc-encoding.mjs`、其测试、本计划、同债 Wave、DEBT_CHANGELOG 与主台账状态格。通过 `git ls-files -z` 取官方角色 JSON，避免遍历未跟踪的 `.oclive_directory_plugin_data` 聊天记录或其它本机运行数据；路径若经过符号链接则拒绝。无自动修复，不改角色内容、产品加载语义、历史事故原件或外部文件。

**验收 / 停止线**：测试须证明已跟踪角色 JSON 入选、未跟踪运行数据排除，且坏 JSON 字节被现有检查拒绝；默认检查与 `dimension5 --ci` 通过，随后做适用文档链接、债结构、编码与 diff 检查。当前角色 JSON 基线无四类异常即停止，不扩为全仓 JSON 搜查、不追单个问号与语义还原；K-ENCODING-01 保持 OPEN，正式目标 SHA CI 另核。

### D-CLI-BLUEPRINT-05 · 显式最小角色本地校验入口（2026-10-03）

**尺寸 / 基线 / 目标**：M，起点 `86cbb2d5a6651da8a0caba594933f8c8a67850b5`、工作树干净。维护者选择继续最小角色包接入；本片只让 CLI 复用已有 `load_minimal_role_local_file`，对调用方指定的定义文件及资产根做有界只读准备。新增 `pack validate-minimal-local <asset-root> <definition-reference>`，预算是 CLI 可覆写的本地策略，不冻结统一包名或磁盘版本。

**写集 / 边界**：CLI `pack_cmd.rs`、一份 CLI 集成测试、本计划、`ROLE_PACK_BOUNDARY`、角色包规范中英摘要、CLI crate README、主台账和 DEBT_CHANGELOG。旧 `pack validate/create` 及参考宿主加载逻辑不变；不生成角色目录、不注入旧 `Role` 的关系/人格默认值、不改 `slot_registry`、公开 Prompt 接口或回合编排。成功只说明逻辑定义与非空本地资产快照合格，不证明媒体可解码、角色激活、真实跨宿主回合或抗并发目录替换。

**验收 / 停止线**：CLI 定向测试用无蓝图的一图加 prompt 目录证明正向入口，缺资产与越界定义路径必须拒绝，原参考宿主 `pack validate` 对同目录仍拒绝。跑 fmt、CLI 定向 Clippy、分层和适用文档/债务门禁；同主线后续切片形成里程碑时才再跑全量本地与目标 SHA 远端 CI。若实现需改变公开 `PromptInput`、角色缓存或生命周期，停止本片并向维护者提出具体迁移取舍，不以第二套解析器或补产品默认值绕过。

### D-CLI-BLUEPRINT-05 · 最小快照到 Prompt Base 的增量适配（2026-10-03）

**尺寸 / 起点 / 目的**：S，接续 `e2ad3066`。维护者选择保留旧接口增量接入；先让已验证的最小角色本地快照能够配置一项真实可调用的 Prompt Base 能力，不把旧 Host 的完整 `Role`、关系/人格/蓝图默认值带入该能力。

**写集 / 停止线**：只在 runtime domain 增加一个借用快照的 Base 实现及定向测试，更新本计划、角色边界文档、台账与变动记录。复用已有文本连接核心，额外非空要求明确拒绝；不改 `PromptInput` / `PromptAssembler` / `AppState` / `load_role` / `process_message`，不注册新默认后端或做第二条回合流水线。此片的成功是能力输入可用，不是 Host 角色激活；Host 绑定另立小片。

**验收**：外部 crate 测试经 `&dyn PromptBase` 以一图加 prompt 的本地快照调用，检查原样人设、材料顺序和额外要求拒绝；runtime 定向 Clippy、fmt、分层及适用文档门禁通过。与上一 CLI 小片合并形成里程碑再跑一次完整本地 CI；远端仅以推送后的目标 SHA 判断。

### D-CLI-BLUEPRINT-05 · 独立最小 Host 装配案例（2026-10-03）

**尺寸 / 起点 / 目的**：S，基线 `c51f8415f702fa8b90da666827cd125cddb41220`。维护者选择先做独立最小 Host 装配案例，再决定是否迁入参考 Host。现有 CLI 与 Prompt Base 能力已具备，但缺一个 Host 自己持有快照、绑定技术身份、择用能力并处理失败的可运行路径。

**写集 / 边界**：只新增 runtime crate 的独立 `minimal_role_host` 示例，附在示例中的隔离测试；同步本计划、角色边界、债务台账与变动事件。示例用临时根目录的一图加 prompt 走真实本地加载器，Host 自己指定技术 ID，持有快照并调用现有 Prompt Base 与内存 Echo LLM；错误在 Prompt 阶段即停止，不产生第二次 LLM 调用。它不接参考 `AppState`、不把最小内容伪装为完整 `Role`、不引入持久化/网络/真实模型/显示器或固定六槽顺序。

**验收 / 止点**：`cargo test --example minimal_role_host` 覆盖正常调用、错身份、额外要求拒绝与缺资产加载失败；`cargo run --example minimal_role_host` 原生 exit 0，输出只报告有界案例通过，不打印人设或资产。定向 Clippy、fmt、文档/债结构通过即停止；本切片不从示例外推参考 Host 生命周期或跨平台发行版验收。与后续同主题切片合并后才跑下一次完整 CI。

### K-LLM-ENV-02 · 原并发缺陷有限验收（2026-10-03）

先前的停止规则要求不再补排列组合，只核目标 SHA 正式 CI。`86cbb2d5a6651da8a0caba594933f8c8a67850b5` 的[正式运行](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37100060646)已 `success`，17/17；本地综合链也已 exit 0。结合可控交错与保存、chat、Theater、canonical seed 的有限路径，按原问题办理 Done。边界和历史阶段口径见[同债 Wave](waves/WAVE-20261002-K-LLM-ENV-02-CONTENTION.md#2026-10-03--原缺陷的有限收口)；不自动扩至真实 provider、跨进程或压力验证。此项只更新治理裁定，不改生产实现或重跑业务测试。

### D-CLI-BLUEPRINT-05 · Prompt Base 输入去本地文件耦合（2026-10-03）

**尺寸 / 起点 / 问题**：S，接续独立 Host 案例 `f1050eb7`。现有 Prompt Base 实现虽然不读文件，却持有 `LocalMinimalRoleSnapshot`；这把可移植能力绑定到一种文件适配结果。已存在共享的 `MinimalRoleDefinition`，且 Host 本就负责保留资产快照。按内核向外延伸的边界，将能力改为借用经共享校验的逻辑定义，Host 从自己的快照提供定义。

**写集 / 止点**：只调整 runtime 的最小角色 Prompt Base 实现、外部 crate 测试、独立 Host 示例与本计划/角色边界/台账/事件。构造时复用 `validate_minimal_role_definition`，不复制 parser，不读取资产，不增公开旧 Prompt 字段；保留对附加非空 requirements 的拒绝和文本输出语义。已有本地 Host 案例继续通过，另加纯内存定义驱动同一能力的定向断言。若需把文件引用、角色 ID 或资源句柄加进 Base 请求，停止并复评，不污染六槽公共合同。

**验收**：定向测试、示例测试与原生示例运行通过；fmt、定向 Clippy、分层和文档门禁通过即形成小提交。再与前两批合并跑一次完整本地 CI，目标 SHA 远端 CI 分列核验。此前 `c51f8415` 的远端结果只证明其自身字节，不作为本切片验收。

### 依赖图现行口径对齐（2026-10-04）

**尺寸 / 止点**：S，文档治理片。已推送的 `cdbfa867` 含独立 Host 案例与最小逻辑定义的 Prompt Base，且本地完整链 exit 0；其目标 SHA 的远端 CI 另行跟踪。顶部依赖图的 D-CLI-BLUEPRINT-05 仍称旧接口选择未决，K-LLM-ENV-02 仍称原缺陷缺压力门禁，均与现行裁定不符。只改调度表两行及事件记录，不触碰历史正文、产品实现、磁盘格式或 CLI 默认。以债务结构、文档链接/编码和差异检查验收；不为这个文档小片重复完整 CI。

### D-CLI-BLUEPRINT-05 · 独立 Host 接受两种内容来源（2026-10-04）

**尺寸 / 目标**：S，接续独立 Host 案例。维护者认可跨发行版先保留逻辑契约、由发行版适配磁盘来源；当前示例 Host 自身仍只存 `LocalMinimalRoleSnapshot`，使第二种来源只能测试 Prompt 能力，不能测试 Host 装配。只在示例内部定义窄内容来源接口，让既有本地快照及纯内存定义/资产都能走同一技术身份、Prompt 与内存 LLM 路径。

**写集 / 止点**：示例、角色边界、债务台账及事件记录。Host 在能力调用前校验逻辑定义、资产数/顺序/非空字节；不规定磁盘文件名、URI 语义、媒体解码或发行版生命周期，不向 Kernel 公共 trait 增字段，不迁移参考 `AppState`。若需要公共来源 trait 或旧 `Role` 产品默认值，停止并另行评估。

**验收**：本地与纯内存来源各走一次同一 Host 路径，失配资产先于 LLM 拒绝；示例测试/原生运行、定向 Clippy、fmt、文档门禁通过。与前一业务提交的目标 SHA CI 分列记录；不因示例重跑完整工作区以制造重复证明。

### D-HOST-ROLE-ACTIVATION-01 · 必需能力拒绝前不发布角色状态（2026-10-04）

**尺寸 / 起点 / 原因**：L 风险的有限修复片，基线 `10e4d4c1d8ca146b7635ccf42b74ccb49dffe921`、工作树干净。该基线已完整本地验证，远端 CI 另核。在为最小角色寻找参考 Host 增量入口时，发现现有 `load_role_impl` 先写当前角色标记并初始化插件 UI 状态，之后才检查 required extension 能否激活；因此被拒角色会改变内存选角和两份本地文件。已有 required 能力拒绝语义不变，仅把无副作用的能力检查移到这两次发布之前。

**闭环 / 写集**：角色存储读取 → 既有 ExecutionPlan 校验 → 角色插件状态发布 → 原 DB、缓存及 RoleData 路径。改角色服务的顺序、`directory_plugins/runtime/mod.rs` 的内部只读初始状态预览、`capability_registry.rs` 的共用状态输入及 `execution_plan.rs` 的接线/拒绝/成功测试；文档只改本计划、主台账和变动事件。必要的兼容边界：能力可用性原本消费初始化后的插件禁用列表，不能简单前移检查而漏掉 legacy v1 待迁移设置；预览必须复用同一初始状态选择、已有角色记录与 global 合并规则，不消费 legacy、不写文件。HTTP/Tauri 共用此服务，wire、错误码、权限、旧 Role/Prompt 接口和官方角色包无需修改。主控自行实施并按实际 diff 自查，不声称独立审查。无真实 provider、插件进程、用户数据或网络。

**回归 / 止点**：用合成 v4 包及内存 SQLite，先成功加载一个可用角色，再尝试 required capability 不可用的角色。原实现必须复现失败后标记/插件记录变化；修复后同一用例应证明当前标记、有效插件快照及状态文件字节不变，被拒角色未建 DB 运行态或缓存。另核 legacy v1 中禁用的 required provider 仍被拒且不提前迁移；保留既有正常激活路径。只调查能力拒绝这一条已发生顺序，不追 DB 错误、并发切换、崩溃原子性或所有状态失败组合；这些不在本债完成条件。

**验证节奏**：先定向红/绿回归，再 Host lib、fmt、定向 Clippy、分层、module-compat 及本轮文档/债务门禁；适用失败与成功日志分别保留。形成可审查本地切片，后续同主题里程碑才跑一次完整本地链并推送；目标 SHA 远端成功前保持 Locally verified，不用 `10e4d4c1` 的结果替代。原 D-CLI-BLUEPRINT-05 仍 Partial，此修复不等于最小定义已经注册到参考 Host。

### D-CLI-BLUEPRINT-05 · 开发者转换与基础闭环的实施范围（2026-10-04）

**维护者回答 / 已定方向**：开发者负责自有丰富格式的转换器；最小角色应能在各发行版运行基础闭环，未有的扩展明确不可用。继续保留参考 Host 旧接口，不补默认关系/人格、不改统一磁盘格式、不把全体发行版已兼容当事实。边界唯一维护在 [ROLE_PACK_BOUNDARY §0.1](../ROLE_PACK_BOUNDARY.md#01-已确认的最小逻辑-contract)。这替代上一事件中仍待产品范围澄清的调度条件，签名、双核、Event Stream 等冻结项不变。

**有界调查结论 / 下一施工点**：已有共享定义、快照及 Prompt Base 足以停止对数据面补证。参考 Host 的 `load_role_impl → role_cache<Arc<Role>> → process_message` 仍走丰富角色；旧 `SendMessageResponse` 要求情绪、关系和人格等字段，不能用默认值冒充这些扩展已有结果。下一片先分开“开发者准备的最小内容与技术身份”“Host 基础输入/结果”“可选产品扩展状态”，保留旧调用面作为兼容适配；新增路径仍在 Host 既定回合入口内组织，不另造六槽解析器或复制整条 pipeline。准确类型和代码写集在实施该片前按实际调用点确定，不预定新 crate、通用转换器协议或中央生命周期框架。

**有限验收 / 停止**：先让一个参考发行版的最小内容走基础文本闭环，显式拒绝不支持的扩展要求，同时保留现有丰富角色正常路径；无显示 Host 可以不渲染资产。仅这两条主路径及必要失败回归，不预先追全部媒体、语音、市场格式、跨平台或崩溃窗口。文档不支撑的实现缺口继续留在原债，不转为无界调查；如实现确需改变已冻结的公共结果/错误语义，再提出具体取舍。

### D-CLI-BLUEPRINT-05 · 参考 Rust Host 基础文本入口（2026-10-04）

**尺寸 / 基线 / 冻结语义**：L 的有限增量切片，起点 `c6d75a9b`、工作树干净。维护者已确认继续施工。开发者先把自有格式转为共享定义与逐引用资产字节；Host 准备一个私有字段的不可变角色句柄，技术 ID 不成为作者字段。现有 `OcliveKernel` 增加基础文本方法，薄转发到既有 `process_message.rs`：校验消息 → 共享 MinimalRolePrompt → 现有 Host 模型设置与已装配 LlmClient 的非流式生成 → 简洁结果。不能构造默认 `Role`，不能把无扩展结果转为旧 `SendMessageResponse`。结果明确表示本路径的产品扩展不可用；非空额外要求仍通过 Prompt 的 typed Unsupported 拒绝，在模型调用前停止。模型失败保留原 AppError，不造 fallback 或重试。

**写集 / 关联面**：`oclive_kernel_types/src/models/dto/minimal_role.rs` 及 dto 导出；Host `service/role/minimal.rs`、role 模块声明、`domain/chat_engine/process_message.rs`、共用 `message_error.rs` 及导出、`role_kernel.rs`、`lib.rs`；新增 `tests/minimal_role_public_api.rs`。角色边界、主台账、本计划及 DEBT_CHANGELOG 同片同步，角色规范中英仅更新接入事实并链接 SSOT；本地快照与 Prompt 的 rustdoc 仅修正当前接入事实。准备句柄可直接接开发者转换后的内存数据，也可从已校验本地快照准备，复用解析与资产读取而不另建通用转换器 trait。旧 RoleCache、角色选角、丰富回合、收据和 HTTP/Tauri 不改。无新 crate、权限、错误码、DB 迁移或角色磁盘格式。

**验收 / 停止线**：合成数据走生产 OcliveKernel builder 与新增门面，证明一次 Prompt/模型调用、资产快照保留、无伪造扩展字段/无角色运行态和聊天落库；必要失败只核无效准备输入、空消息、真实额外要求及模型失败，保留已完成空文本。原 rich facade 集成测试作为兼容回归。新公开 API 跑 workspace doctest，Host lib、适用 Clippy/fmt、分层、module-compat 与文档/债务门禁；控制方自行语义审查，不称独立审查。完成同主题切片后一次全量本地链、冻结提交与推送；目标远端成功前原债仍 Partial。不给此单次 Rust 文本调用赋予历史、多轮记忆、持久化恢复、流式、渲染、语音或 ChatPro UI 接入结论，不扩查那些路径。测试仅临时夹具与内存模型；不使用真实模型、用户 DB、网络或进程插件。

### D-CLI-BLUEPRINT-05 · 生成 library 的最小角色调用面（2026-10-04）

**尺寸 / 起点 / 原因**：M 的有限消费片，起点 `96ca9d29832c9560e1427843db76154dbb63d915`，工作树干净，父批正式 CI 另核。基础 Host 已能处理开发者转换后的最小角色，但 CLI 已链接的 library 仍只在根部导出旧丰富门面，并只给丰富角色示例。本片让生成库直接导出已有最小角色句柄、请求、结果、扩展状态和 typed error，提供能编译的 Rust 用法；不增加薄转发函数或第二条编排。

**写集 / 闭环**：CLI `src/templates/lib.rs.hbs`、`README.generated.hbs`、`generator.rs` 的现有 library 合同测试、新增 `tests/e2e_minimal_role_library.rs` 与其隔离 fixture；CLI 指南中英、ROLE_PACK_BOUNDARY、本计划、主台账和 DEBT_CHANGELOG 同步。输入是开发者转换后的定义/资产 → 原准备句柄 → 原 Host 基础方法 → 原三字段结果 → 生成库消费者；无新 DTO、运行依赖、权限、格式、命令或数据库迁移。未链接的 serde stub、旧 rich API、kernel-server 和默认角色包生成保持原逻辑。隔离生成项目沿用根 lock 的已解析版本，避免离线用例只在旧机器缓存上成立。原试加 Host dev-dependency 被默认轻量依赖树回归拒绝，已撤回；改为沿用 `diagnostics-host` 显式启用消费测试，由外层 Cargo 准备 Host 闭包。CI 写集仅扩至 `ci.yml` 的既有 cli job 新增这一条特性测试命令；runner、job 选择、actions/permissions、默认命令和 gate 均不变，不在此重新设计 CI。

**验收 / 停止**：先在旧模板上跑新生成合同的拒绝，再生成独立 library 并由真实 Cargo 编译/运行一次生产 Host + 内存 LLM，核原样 reply、显式 unavailable 与 typed Unsupported 的零新增生成；生成 rustdoc 用法也编译。复用仓库外 Cargo target，嵌套 Cargo 串行，不另建整套 Host 验证。CLI 定向测、Clippy/fmt、workspace doctest 与适用文档/债务门禁完成后形成可审查提交；整批冻结才跑完整本地链并按已有授权推送，不为每个小改启动正式 CI。父批 success 不替代新 SHA；原债保持 Partial。完成实际生成消费即停，不补 HTTP/Tauri/UI、全平台、真实模型/媒体或生命周期证据。主控自行实施和 diff 自查，不声称独立复核。

### D-DEBT-LEDGER-01 · 最小角色当前状态与接手用法收敛（2026-10-04）

**尺寸 / 基线 / 原因**：M 级文档治理，起点 `5dc3d1d0d7e43d6ee30c3ca3cbb194e36786cae2`，工作树干净。D-CLI-BLUEPRINT-05 的当前状态格累积多轮历史，先前“未接”的阶段描述与当前实现并列；CLI `pack create` 指南还笼统写生成/识别未实现，不能准确区分已有显式文件准备与没有统一磁盘生成器。整理现行入口，不重新测绘代码或降低产品承诺。

**写集 / 保全**：主台账只缩短 D-CLI-BLUEPRINT-05 的状态格，ID、问题、优先级与完成条件原文不变；将起点整行原文保存到 `handoff/archive/TECHNICAL_DEBT_MINIMAL_ROLE_STATUS_20261004.md`。同步本计划、台账 Wave、DEBT_CHANGELOG 及 CLI 指南中英的 `create` 说明，共七份文档。当前状态仍唯一由主台账负责，归档仅保留历史；D-CLI-BLUEPRINT-05 与 D-DEBT-LEDGER-01 均不转 Done，QUEUE/机器计划、Rust/模板/门禁和运行原件不动。

**验收 / CI 节奏 / 止点**：机械比较快照与起点整行、前四列及其它权威行保持原文；核现有 `validate-minimal-local` 与边界 §0.5/§0.9 的主路径即可。默认及改文链接、镜像、docs-only 旧路径、登记、债结构、编码及 diff 通过后形成本地文档提交；不重复 Rust、业务场景或完整本地链，也不为这一小片立即再推送触发全量 CI，随下一实质批次统一推送。上一工程目标的远端结果单独核对，不把它当本文档 HEAD 的验收。主控自行按 diff 复核；完成该行与指南的准确分层即停止，不迁其它长状态、不新增所有未覆盖面的调查任务。

### 生成库消费回归 · Cargo 彩色日志合同修补（2026-10-04）

**尺寸 / 起点 / 实际失败**：L 的有限 CI 修复，起点 `4f98d34ec49dd94a0260251785a80cea95012b7c`，工作树干净。`5dc3d1d0` 的正式 CI `37149237736` 中原 CLI 步骤通过，新增消费步骤失败；子 Cargo 消费和 rustdoc 实际成功，但 CI 强制彩色日志在 `Doc-tests` 与 crate 名间插入 ANSI 控制字节，使外层纯文本断言误拒绝。保留原作业日志，不通过无变化 rerun 或删断言假绿。

**写集 / 验收 / 止点**：仅 CLI `tests/e2e_minimal_role_library.rs` 为子 Cargo 显式指定 `--color never`，同步本计划与 DEBT_CHANGELOG 的失败归因；不改模板、产品 API、依赖、工作流或原断言。用进程级 `CARGO_TERM_COLOR=always` 在旧测试复现拒绝，修后同配置消费和 rustdoc 各一项成功，恢复原环境；定向 Clippy/fmt 与改文/债结构门禁后冻结，按 main 修复要求一次完整本地链，再合并已核文档切片统一推送并核新 SHA CI。确认这一个实际失败修正即停，不新建 Cargo 日志解析器或扩平台研究；主控自行审查，不声称独立复核。

### K-CI-IMPACT-01 · 公共契约与模型特化的渐进归属（2026-10-04）

**尺寸 / 基线 / 维护者确认**：M 的设计文档切片，起点 `5b3153ae3bbe0a0bf0b3359202d22c36a0f62b89`、工作树干净。维护者确认 Kernel CI 与特定 7B 模型工程应分责、模块化按需要推进，无需完整重新设计。复用 [CI 设计 §2.4](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#24-主工作流的执行所有权) 的既有四层模型，只补分类依据、模型条件和有限迁移出口；不另建 CI 设计或债务 ID。

**写集 / 不变量**：本计划先登记范围，再更新 `creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md` 与 DEBT_CHANGELOG。Kernel 的接口/合法性测试不绑定模型；具体实现的质量、提示词及资源承诺归实现/Host，证据绑定其模型与运行条件。模块化先整理实际接口、依赖和可独立运行的验证，真实耦合阻碍时才提出代码拆分。当前 workflow、模块元数据、validation catalog、中央影响图、选择性策略和门禁源码均不动；父债状态、其它暂停项不变。

**验收 / CI 节奏 / 止点**：只运行文档默认及改文链接、docs-only 旧路径、登记、适用镜像、债计划、编码与 diff 检查。三文件 diff 自查通过后形成本地提交，随下一实质批次统一推送，不为此重复 Rust、模型测试或全量 CI。后续迁移只选一个有实际过选/重复执行依据的范围；未知保留，足以决定路由即停止调查。不得凭责任标签删依赖边、预设新的物理 crate 或将该片记为 CI 分层实施完成。

### D-CLI-BLUEPRINT-05 · 参考发行版入口的待决边界（2026-10-04）

**已核断点 / 停止线**：基础 Rust 结果见 [角色边界 §0.9](../ROLE_PACK_BOUNDARY.md#09-参考-rust-host-的增量基础文本入口)，只有技术身份、正文和扩展不可用，不含持久化历史或恢复保证。ChatPro 的 `RoleData` / `RoleInfo` 及 `roleStore.mapRoleInfo` 仍维护关系、人格等丰富状态，`chatStoreSend` 按丰富结果应用产品状态；不能用默认值或类型断言把最小结果伪装为丰富结果。

**待维护者选择 / 接续**：已提出两项具体产品范围：独立基础会话（先接加载、文本交互、不可用提示，不承诺落库/恢复），或主聊天统一接入（增加真实能力禁用、历史与生命周期适配）。回答前不改生产 DTO、传输、加载缓存、角色选角或聊天 UI。确认后只写所选范围的实际代码写集、成功/必要失败及旧路径回归，继续有限施工；不为待决问题重测已有数据面或扩查全部产品路径。当前共享契约与开发者转换责任不重开，原债继续 Partial。

**2026-10-04 后续裁定**：维护者选择先在 Host 与六槽之间提供可复用最小角色消费者，不要求每轮全六槽执行；Host 继续按原契约与能力对接。下一施工顺序改为公共消费者，以上两种 ChatPro 产品入口留作之后的产品选择，不再阻塞下述共享切片。无损限定为保留适用请求/要求、结果/错误及来源，不暗增调用或授权；不保证零成本、零延迟或把丰富数据投影为 Base 后仍携带全部扩展。

### D-CLI-BLUEPRINT-05 · 可选择 Prompt 的最小角色共享消费者（2026-10-04）

**尺寸 / 起点 / 待修问题**：L 的有限增量公开调用面，起点 `1524d5e992fca8d3dc07d4ebc2c0ccd9d2226c72`、工作树干净。最小角色的内容准备当前只在 `MinimalRolePrompt` 中与固定拼接能力一起提供，不能复用这份内容准备去消费 Host 选择的另一个 Prompt Base。维护者已确认可复用消费者与原 Host/六槽语义保持相容；本片只解决这一处真实耦合，不造六槽转发框架或通用执行器。

**冻结语义 / 写集**：新增 runtime `domain/minimal_role_consumer.rs` 的 `MinimalRolePromptConsumer`，构造时复用最小逻辑校验、借用定义及调用方选择的 `dyn PromptBase`；每次显式调用保留原人设和所有材料片段的字节、次序及边界，追加既有角色/材料文本标题，把 `requirements` 原样交给所选 Prompt，至多发起一次调用，原样返回其结果或完整 BaseCallError。不读取资产/文件、不创建 Role 或状态、不驱动其它槽、无 fallback/重试，也不增强非 Send/Base 的调度承诺。当前 `MinimalRolePrompt` 复用消费者接既有 `LiteralMaterialAssembler`；保留旧非空要求的提前拒绝与原错误说明。runtime domain 声明、该旧实现、新外部测试为代码写集；MODULE_MAP 只记消费者权责，ROLE_PACK_BOUNDARY 只记角色适配与范围，本计划与 DEBT_CHANGELOG 记录实施/接续。六槽 trait/types、Host 调用面、生成模板、Cargo 配置/锁、角色包、HTTP/Tauri/UI 和暂停项均不改。

**必要回归 / 复核 / 止点**：先让新外部消费测试在旧源码编译拒绝，再验证 Host 可选择 Prompt、非空要求与材料保持、一次调用、正常空结果及完整失败原样传递；一个真实 Pending 用例核借用材料在恢复后仍有效、不会再次调用。既有最小 Prompt、独立 Host 示例和参考 Host 最小入口作为兼容回归。定向 Clippy/fmt、分层、workspace doctest及适用文档/债务门禁；公开 API 的实际外部使用由本片集成测试编译，runtime 既有 doctest=false 不改、不声称其代码块已被 doctest 执行。主控自行实现与语义自查，不声称独立审查。完成此准备/消费断点即停，不为其它五槽机械添加同形包装或扩产品路径；同主题里程碑冻结后只跑一次完整本地链并按既有授权合批推送，目标远端另核，原债继续 Partial。

**实际兼容处理**：旧 `MinimalRolePrompt` 保留定义引用的存储方式，构造校验与非空要求提前拒绝不变；通过 consumer 模块内唯一准备函数消费原 Literal 实现，不直接持有 `dyn PromptBase`，避免丢掉旧类型的 `Send/Sync` 属性。新外部测试增加编译期属性断言与原完整错误说明的比较；通用消费者仍接受本地、非 Send 的实现。该实现选择不改变 Host 或 Base 的线程承诺。

### D-CLI-BLUEPRINT-05 · Host 显式选择共享 Prompt 消费者（2026-10-04）

**尺寸 / 基线 / 有界断点**：L 的增量公开调用面，起点 `6784db4cc1b7daa5fbcaefe3720f26c235e558ce`、工作树干净；父批远端另核，可本地开发、不推送去取消父批。维护者确认 Host 保留六槽对接与选择，共享消费者负责基础适配，并授权连续小批次实施。当前 `OcliveKernel::process_minimal_message` 仍固定 Prompt，无法通过这条实际基础入口使用已实现的公共消费者选择能力。只处理此断点及独立 Host 的用法，不进入 ChatPro 产品入口选择。

**A · 参考 Host 接线**：保留旧方法签名和非空要求的原 Unsupported；增加 `process_minimal_message_with_prompt(role, request, &dyn PromptBase)`，所选 Prompt 尚未带人设，由共享消费者准备一次。domain 的同一基础编排先拒空消息，再选择原最小 Prompt 或共享消费者；之后统一复用既有 Host 模型设置、单次模型调用与三字段结果。材料/要求、正常空输出与完整 Prompt 错误不改写；失败不生成、不 fallback/重试、不填扩展默认值。facade 只薄转发，非 Send 的本地 Prompt 仍可用，不引入注册表、全局选择或生命周期管理。代码写集为 Host `process_message.rs`、chat_engine 导出、`role_kernel.rs` 和现有 `tests/minimal_role_public_api.rs`；不新增 DTO、trait、错误码、依赖或数据库迁移。

**B · 独立 Host 使用**：现有 runtime `examples/minimal_role_host.rs` 增加按次选择 Prompt 的入口与一个只支持“逐片 JSON 引用并保留材料内容”要求的本地实现，复用共享消费者与示例原模型调用，不复制角色准备、拼接核心或 Kernel pipeline。引用是调用方明确选择的组装协议，解码后每片内容/次序保持，不冒充输出字节未变化或下游防注入保证。示例的旧身份/资产校验和默认路径保持；该实现的有限要求语义仅属于示例，不成为 Kernel 的要求语言。记录实际 native 运行和一项选择/拒绝回归即可，不扩大为第二个实际发行版验收。

**文档 / 必要验证 / 停止**：ROLE_PACK_BOUNDARY 维护两条使用入口，MODULE_MAP 仅在现有消费者段补 Host 接线；主台账只更新本债状态格的当前能力和接续链接，前四列与 Partial 不变；本计划、DEBT_CHANGELOG 同批同步。A 先在旧方法不存在时编译红，再用生产 builder + 临时 SQLite + 内存模型验证自选/一次调用/非空要求及原样 LLM 输入、五类 Prompt 错误停止且保留完整信息、空消息零调用、正常空 Prompt 与模型结果/模型失败不修补；原五项作为兼容回归。B 保留原五项并加一项窄测、实际运行示例。定向 Clippy/fmt、workspace doctest、分层/错误码漂移/模块兼容与适用文档/债务门禁；控制方自行语义审查，不称独立审查。两片先本地提交，合成一个冻结批次只跑一次完整本地链，再按既有授权推送并核目标 CI。达到真实选择/消费主路径即停，不扩查另外五槽、媒体、全部异常或崩溃窗口；签名、双核、Event Stream 等暂停不变。

**实际实施 / 兼容核对**：A 与 B 共用消费者，没有变更六槽 trait、DTO、配置或依赖。Host 的九项外部回归中，新增用例确认同一 Host 在显式选择之后再走默认入口时仍使用原 Literal，选择仅属于当前调用；本地非 Send Prompt 在真实异步让出后读到原材料。B 选择有限 JSON 协议，解码后逐片核内容而非宣称任意 Prompt 都输出同一字节。两片已合并为同一实质提交批次，避免为中间片单独完整验证。编译红、首次 fmt 红及修后结果分别保存于 `.cursor/plans/debt-minimal-role-host-choice-20261004-r0/`；当前 Host lib 635 项、外部九项、示例六项/native、定向 Clippy 与 workspace doctest 47 项已通过。后续适用文档门禁和冻结完整链另记，不将本地结果外推为目标远端已通过。

**2026-10-04 收口 / 下一片准入**：上述实际代码冻结于 `8a642fa004cb62fb9364b66bc2bc33eed4b9552a`；完整本地链 native exit 0，前后同 SHA、干净工作树、临时环境恢复，[正式 CI 37160662341](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37160662341) 对同一 SHA success、17/17。有限共享消费者/Host 接线里程碑已通过，父债仍 Partial。当前只登记这一迟到的远端结果，写集限本计划、DEBT_CHANGELOG 本事件及主台账本债状态格，前四列不变；适用文档检查后本地提交，随下一实质批次推送，不为结果登记重复 Rust/模型/全量 CI。ChatPro 的独立基础会话或主聊天统一接入仍待明确产品范围，本次没有实现传输/UI，也不自动加入历史、恢复和丰富生命周期。

**2026-10-04 后续选择 / 已确认归属**：维护者已选择让最小角色进入发行版主流程，独立基础会话不再是默认施工方向；随后明确“共享运行库承担最小适配，保留小 Kernel 边界”，并补充仅在核心外加一层，方便最小角色内容通用。Host 保留能力/资源绑定、调用与结果接线；不增加完整产品角色的通用必填要求，也不把六槽变为每轮必跑阶段。该选择没有将角色生命周期、授权、状态应用或固定回合调度转入小 Kernel，也不自动承诺历史/恢复。两次确认归入 DEBT_CHANGELOG，不再等待这一归属决策；后续实际写集见下片，父债 Partial 与暂停项保持。

### D-CLI-BLUEPRINT-05 · 核心外共享基础文本消费（2026-10-04）

**尺寸 / 起点 / 实际缺口**：L 的增量公开库调用面，代码基线 `d780bec1`（完整 SHA 由本机输入记录保存）；开场仅上段本控制者的未提交选择登记，无他人变更。维护者已确认主流程接入目标和核心外共享层。现有公共消费者只处理人设与 Prompt；独立 Host 仍自行把已准备输入接入 LLM。补这一实际复用断点，再进入参考发行版的实际接线，不扩查全部产品/六槽实现。

**冻结语义 / 关联闭环 / 写集**：在已有 runtime `domain/minimal_role_consumer.rs` 增加 `MinimalRoleTextConsumer`，借用最小定义及调用方绑定的 `PromptBase`、`LlmBase`，构造复用现有逻辑校验。一次显式 `generate` 复用已有 Prompt 消费者；只有 Prompt 正常完成后才调用 LLM，输入逐字等于 Prompt 输出。两种失败分阶段包裹完整 BaseCallError，正常空输出、要求和借用保持；无 retry/fallback、结果修补、调度器、资源访问、状态/身份/持久化或其它槽调用。此为可选基础文本操作的实际因果，不规定 Host 的六阶段顺序。既有 Prompt 消费者和旧默认接口不变。独立 Host 的显式选择分支改用此操作，默认分支与原窄错误载体保持。代码写集仅该模块、现有外部 `tests/minimal_role_consumer.rs` 和 `examples/minimal_role_host.rs`；契约/types、Host/HTTP/Tauri/UI、依赖、模板和用户数据不动。角色用法在 ROLE_PACK_BOUNDARY，模块权责只在 MODULE_MAP；本计划、DEBT_CHANGELOG 及父债状态格同步，不改债行前四列或 Partial。

**必要验证 / 复核 / 停止**：先让新外部调用在旧源码因符号不存在编译红，再验证完整材料/要求与 Prompt 输出直接到 LLM、一次调用、两阶段各五类完整错误（Prompt 失败模型零调用）、正常空输入/输出不改写、逻辑无效时两能力零调用；通过两个真实 Pending 边界验证本地非 Send 实现与 LLM 输入借用。原四项消费者回归、示例六项/native 与参考 Host 九项作兼容核对。定向 runtime all-targets/all-features Clippy/fmt、workspace doctest和适用分层/模块/文档/债结构/编码门禁；runtime 既有 doctest=false 保持，不声称其代码块已执行。控制方直接实施和语义 diff 自查，不称独立复核。局部通过后与已保留文档提交合成一批，冻结后一次完整本地链并按既有授权推送，新 SHA 远端另核。达到公共基础调用与一个真实消费者接线即停，不制造六个同形包装、固定 Kernel 回合或通用要求语言，也不凭本片宣称 ChatPro 主流程已经完成。
### D-CLI-BLUEPRINT-05 · 主流程接线的最小文本传输适配（2026-10-04）

**尺寸 / 基线 / 本片出口**：L 公共传输与 Host 接线，起点 `692aa7f4c51fcd89c9282dd1745f569a203a3516` 干净且已推送；该起点完整本地链 exit 0、[正式 CI 37184869027](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37184869027) 17/17。维护者已确认主流程目标与核心外共享适配，本片不重问归属。先让转换后的最小内容走真实受保护 HTTP → 已鉴权桌面 Rust 桥 → IPC → shared API，后续同批另片接选角与主 composer；本片只验收实际传输出口，不宣称 UI 已接或父债 Done。

**输入 / 行为冻结**：新增参考 Host 专用的本地来源与消息封装，显式提供技术 ID、绝对资产根和相对定义引用，复用已有逻辑 JSON / 资产快照加载器。文件名由调用方选择，64 KiB 定义 / 4 MiB 单资产 / 16 MiB 总资产是这个 Host 的本地预算，不是通用最小格式。消息保留原文，仅拒绝全空白；默认传输绑定现有 Literal Prompt，非空 requirements 明确以既有 InvalidParameter 拒绝，不清空要求获得成功，不建立通用 BaseCallError→HTTP 映射。结果使用现有基础 DTO：reply 与 product_extensions=unavailable，无假关系/人格/落库 ID；无自动重试、丰富激活、收据、历史或恢复。

**闭环 / 写集**：共享 DTO `kernel/crates/oclive_kernel_types/src/models/dto/minimal_role.rs` → Host `service/role/minimal.rs` / canonical `domain/chat_engine/process_message.rs` 及其 re-export → `http_api/chat.rs` / `mod.rs` → desktop `api/chat.rs` / `chat_backend.rs` / `kernel_attach/chat.rs` / `lib.rs` → `distros/shared/src/api/chat.ts`。新增 Host `tests/minimal_role_http.rs` 与 shared `api/minimalRoleChat.test.ts`，desktop 桥的合同检查放在既有 chat 模块测试区；同步本计划、角色边界、模块接线、父债状态格与 DEBT_CHANGELOG。旧完整 Role、发送/恢复/SSE、角色注册、六槽 traits、官方包、依赖与用户数据不改；Local 分支只服务现有测试/非桌面 Host，桌面仍由 HTTP 进程权威执行。

**实施 / 复核 / 停止线**：控制方直接实施和语义自查，不称独立审查。现有 Base future 不要求 Send，在 Host 明确的 blocking worker + 当前 Tokio Handle 执行局部基础调用，不扩大六槽 async 约束。先写真实 Router（不监听端口）与 shared API 窄测，覆盖成功、鉴权拒绝零模型、坏来源/越界/超预算/非空要求零模型、模型失败原码和无丰富状态副作用；桥检查体/目标路由与既有错误映射，不手工加渲染层令牌。通过后核 Host/desktop 定向 Clippy、fmt、公开 API workspace doctest、layering/error-drift/module-compat、typecheck/eslint 与适用文档/债务/编码检查。本片先本地保留；主流程关联小片完成后的预定冻结点只跑一次完整本地与目标远端 CI。达到这些条件停止扩证，不启真实模型、音频、产品进程、监听器或历史身份；若发现新的产品/权限歧义，保留当前片并列明具体问题。

### D-CLI-BLUEPRINT-05 · 主聊天接线前的临时最小会话消费（2026-10-04）

**尺寸 / 基线 / 有限依赖**：M 级前端状态切片，起点 `8c84bdbd5193f207c93012c7df040ebb2353c3e2` 干净、仅本地验证。现有 roleStore 的完整 RoleInfo 与 chatStore 的历史缓存不是最小结果的合同；删除 / 编辑会调度缓存写入，不能把最小结果塞入旧状态再靠默认值隔离。本片先提供主聊天可直接消费的 shared 临时状态，不开放第二个独立会话产品，也不宣称选角界面已完成。

**冻结行为 / 写集**：新增 `distros/shared/src/stores/minimalRoleChatStore.ts` 与同目录 `minimalRoleChatStore.test.ts`，复用现有 ChatMessage 形状及 `sendMinimalMessage`。显式绑定本地来源只登记临时输入，文件校验仍由 Host 在发送时权威执行；不称已加载 / 激活，不写丰富 RoleInfo / Host 插件选角。每次正常发送一次 IPC，正文 / 正常空回复原样保留；核返回技术身份与 unavailable，结果只加本地气泡 ID。取消、重新绑定和较新的发送使旧结果失效，立即移除旧在途气泡；取消只撤销客户端展示，不承诺取消 Host 生成。失败清理在途气泡并原样抛错，无普通发送 / 恢复 / 重试。最终事件一次、skip_auto_tts=true，消息提交 / 最终事件与真实 bus 相连；临时状态不接历史 / IDB / persist 插件或丰富扩展。已有语音 submit 消费者会预热资源，故在 `composables/useVoiceAutoTts.ts` / 同目录测试补显式 skip_auto_tts 的提交早退，保留旧事件行为。同步本计划、ROLE_PACK_BOUNDARY、父债状态格及 DEBT_CHANGELOG，其余源、UI / locale / 角色包 / Kernel / 依赖不动。

**验证 / 复核 / 停止**：用真实 Pinia store、真实 mitt bus，仅 IPC 替身验证在途 / 原文 / 单次结果、来源快照、正常空结果、取消后晚成功 / 晚失败、重绑定与新发送、错误 / 身份错配零最终事件、无丰富发送 / 恢复。定向测试与旧 API / stream / rich send 回归、typecheck / ESLint / module-compat、文档 / 债务结构 / 编码 / diff 检查后自行语义复核并本地保留。无 Rust API 改动，不重跑 workspace doctest / Rust 或完整 CI；待同批 UI 接线后冻结统一验证。不扩查全部角色消费者，不启动模型 / 音频 / IPC 进程 / 用户数据；达到这个复用状态入口即停止这一片。

### D-CLI-BLUEPRINT-05 · 两套主聊天界面的最小角色接线（2026-10-04）

**尺寸 / 基线 / 归属**：L 的主流程状态与关联消费，起点 `3958126bdcc9d3bb0c384081f457d0c6c4159143` 干净、前两片仍仅本地验证。沿已确认主流程 / 核心外共享适配 / 增量接口方向，不重问归属，不扩大六槽、小 Kernel 或后端丰富激活。前端独立最小绑定表示当前主聊天上下文；原丰富角色数据保留为可返回的上下文，不把它重标为最小角色，不承诺其后台服务已关闭。

**行为冻结**：Fluent / Tool 的现有选角区提供同一个最小来源入口，调用方指定绝对资产根与相对定义文件，前端为这次临时绑定产生技术 UUID；不规定作者文件名、关系或蓝图，不添加 renderer 文件 / 令牌读取。绑定前取消旧客户端发送与当前成人队列，队列取消失败不进入最小模式；来源字段复制冻结，文件 / 模型可用性仍在第一次发送由 Host 权威检查。复用现有主 composer、ChatMessageList 与共用发送 hook，把基础回复交给 §0.12 的临时状态；切换 / 取消 / 卸载失效旧结果，返回完整角色恢复原丰富路径。丰富立绘 / 关系 / 场景 / 人格 / 成人 / 插件工具 / 语音 / 历史及角色设置在此模式不可用，页面明确提示；模型管理作为 Host 资源入口保留。停用角色快照轮询、插件角色变更 / ASR 提交、语音预热 / 热键等直接关联入口，不能只隐藏按钮却继续向旧角色下判。

**写集 / 闭环**：新增 shared `components/role/MinimalRoleSourceControls.vue`、`composables/useMinimalRoleSelection.ts`、`integration/minimalRoleMainFlow.test.ts`、`composables/minimalRoleScope.test.ts`。修改 shared `stores/minimalRoleChatStore.ts`（来源快照校验共用）/ `roleStore.ts`、`composables/useChatSend.ts` / `useKernelStatus.ts` / `usePluginEvents.ts` / `useGlobalHotkeys.ts` / `useVoiceAutoTts.ts` 及后者既有测试、`components/chat/ChatInput.vue` 与既有测试、`components/TopBarMorePanel.vue`、`i18n/locales/fragments/app.{zh,en}.ts`；ChatPro `composables/useMainShell.ts` / `useMainShellChat.ts` / `useMainShellHotkeys.ts`、`shells/fluent/FluentShell.vue`、`shells/tool/ToolShell.vue` / `ToolActivityBar.vue` / `ToolStatusBar.vue`。同步本计划、ROLE_PACK_BOUNDARY、父债状态格和 DEBT_CHANGELOG。Kernel / Rust / Host、权限 ACL、rich DTO、官方包、依赖、历史证据与其它暂停债务不动。

**验收 / 停止线**：真实 Pinia、mitt、来源表单、composer、主聊天共用 hook / 列表的内存集成覆盖绑定 → 发送 → 权威正文、取消 / 失败 / 返回旧角色与无丰富请求；实际 scope 守卫覆盖快照在途与新调用、角色事件 / ASR、语音配置 / 切换与热键停用。IPC / 音频为替身，不启真实产品、网络、模型、用户 DB。旧前端全套、i18n、两壳编译 / 构建、typecheck / ESLint / module-compat 及适用文档 / 债 / 编码门禁；控制方按真实 diff 自行语义复核，不称独立审查。局部通过后冻结这一关联批次，在同一 SHA 上一次完整本地链，按既有推送授权推送并核目标远端；不每改一个控件跑全量。完成这个基础主流程即停止扩证，不追全平台 / 所有崩溃窗 / 全部插件或媒体验收，不把父债转 Done；出现真正新的产品 / 权限歧义再问维护者。

**实施中关联写集补充**：`shared/composables/useTheme.ts` 的角色主题消费者也必须随最小上下文清除旧角色的内联主题，并在返回后恢复原角色主题，不把旧包品牌投射到最小绑定。按住讲话已经开始时再绑定最小角色，MainShell 热键消费者显式发一次 stop 收尾（仅自身已开始的句柄），避免 disabled 后 keyup 不再命中；这不启动最小角色语音，也不改通用热键协议。

**按键生命周期闭环补充**：`shared/composables/useUnifiedKeybindings.ts` 在能力停用时还须清除自己保存的 pressedHold，并仅为已按住的动作执行 onStop；否则最小模式期间 keyup 被拒后，返回完整角色会继续误认为按键仍按住。同一 scope 测试通过 importActual 挂载真实注册器、派发键盘事件，核停止一次 / 无新启动 / 返回可再次启动；不改绑定存储格式或扩大为通用热键重构。

### D-CLI-BLUEPRINT-05 · 最小角色的六槽可替换消费（2026-10-04）

**目标澄清 / 基线**：维护者明确目标是同一个最小角色可使用发行版选择的六槽实现，尤其 Memory / Emotion 实现不同也不要求改包或补丰富角色字段。基础文本主流程是已有子片，不等于该目标；返回完整角色仅是 ChatPro 的切换兼容行为。基线 `85a9df845dd2b912817745485ec9bd2c43229662` 干净，本地完整链 exit 0、远端 [37202649802](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37202649802) 17/17。

**本片 / 冻结语义**：L 的公开复用调用面，controller 直接实现与自查，不宣称独立审查。先补 runtime 的六槽共享消费者：借用已校验的最小定义与 Host 显式绑定的六个 Base；Prompt 复用既有人设 / 材料准备，其余槽沿对应 Base 请求消费 Host 提供的查询、材料、上下文或任务，保持原值和完整结果 / 错误。不向每个槽注入人设、产品身份或权限，也不把所有字段封成隐含 RichRole。消费者提供六种独立调用，不自动串行、重试、读写数据或应用领域状态；真实依赖、记忆来源 / 命名空间、任务授权、资源与调度仍由 Host 负责。旧 Prompt / Text 消费者、六槽契约、小 Kernel、丰富接口与传输保持不变。

**写集 / 有限验证**：`kernel/crates/oclive_kernel_runtime/src/domain/minimal_role_consumer.rs`，新增同 crate `tests/minimal_role_six_slots.rs`；同步 ROLE_PACK_BOUNDARY、MODULE_MAP 的共享消费归属、本计划、父债状态格与 DEBT_CHANGELOG。外部 crate 的 Host 组合用同一定义绑定两组不同 Memory / Emotion，六槽均实际调用并将实际结果继续交给后续消费者；至少两种合法调用关系避免固定六阶段，保留人设和最终输入。补逻辑拒绝零调用、正常空值与全部 typed 失败不冒充成功、不触发其它槽或重试，以及本地 Pending / 借用生命周期。使用现有生产 Base 参考实现与内存能力，不启动模型、音频、网络或用户库。

**止点 / 关联批次**：这片完成的是共享六槽调用接入，不声称现有发行版已迁移全部生产绑定。实际 Host 装配是下一片，必须核具体 provider 是否能承担 Base 视图，不能靠返回空值、把四槽标扩展不可用或直接把数值 DTO 换名绕过缺口；涉及额外模型调用、权限或产品策略的新取舍再问维护者。当前先跑六槽 / 旧消费者窄测、runtime Clippy / fmt、workspace doctest及分层 / 文档 / 债务门禁，本地保留；同一关联批次冻结时只跑一次完整链与目标远端，不把每个小片独立推送。第一轮不调查所有发行版、媒体、崩溃窗口或六槽实现，证据够实施即停。

**本片局部出口**：六槽共享入口、同一定义的两种 Memory / Emotion 装配及必要失败 / 空值 / 借用边界已实现；旧基础操作保持。实际检查与一次本机 Python 门禁修正见 [DCL-36](DEBT_CHANGELOG.md#dcl-20261004-36--同一最小角色的六槽可替换消费)。这一片在本地保留，下一片直接处理参考 Host 的实际能力绑定，不继续扩大共享调用面的反例调查。

### D-CLI-BLUEPRINT-05 · 独立 Host 的原生六槽装配案例（2026-10-04）

**基线 / 有限出口**：M 的案例接线，起点 `f48095a1a63f2fb676b7aaea46b3fea9272d410f` 干净、ahead 1、仅局部验收。只在已有 `minimal_role_host` 案例追加一条 Host 自己选择的六槽操作，沿既有本地 / 内存内容适配与共享消费者运行。参考生产 Host 的旧 Event 需要真实人格 / 情绪 / 历史，其 Agent 有工具资源 / 授权约束；不能填默认值或替换当前产品装配来假装接通。本片提供能运行的开发者用法，不为 ChatPro 开启新调用或资源。

**行为 / 选择**：同一份最小内容分别绑定原生 KeywordMemoryBase / KeywordEmotionBase 与 QueryMemoryRetrieval / BuiltinUserEmotionAnalyzer，Prompt 使用现有 BuiltinPromptAssembler 的 Base 视图，Event 使用独立 LlmEventAnalyzer，Agent 显式委托 ScalarCountAgent 支持的纯计算任务；六槽全部实际消费，不转换产品数值结果。Host 组织候选材料 / 查询、当前消息、分析背景和合法委托；实际结果进入最终 Prompt / LLM，人设只准备一次。Event 的协议生成器与正文模型为明确内存替身，前者一次、后者一次，各路径总 2 次假生成；不宣称真实模型质量。其它 Host 可选不同依赖，案例操作不是公共固定流程。原文本 / 自选 Prompt 路径保持。

**写集 / 复核**：`kernel/crates/oclive_kernel_runtime/examples/minimal_role_host.rs`；仅修正 `domain/base_memory.rs` / `base_event.rs` / `base_agent.rs` 中“任何 Host 尚未接入”的过时范围措辞；同步 ROLE_PACK_BOUNDARY §0.14、MODULE_MAP 的案例接线、父债第五列、本计划与 DEBT_CHANGELOG。不增公共请求或 trait、依赖、生产 AppState / ChatPro / 默认角色 / 授权 / 存储 / rich lifecycle。新案例回归先编译红，再验两种真实内容来源与六槽装配、技术身份拒绝零生成、分析格式 / Agent 要求失败不达正文模型且已发生的分析不被称为零调用。跑案例九项（旧六 + 新三）、原六槽 / 消费者回归、案例 native、runtime Clippy / fmt 与适用文档 / 分层 / 模块门禁。controller 实施与自查，不称独立审查；局部通过后本地保留，关联冻结点才一次全量链与目标远端。达到实际装配与错误出口即停，不继续穷尽模型 / 跨平台 / 媒体，也不把父债标 Done。

**本片出口 / 批次冻结（2026-10-05）**：九项案例与原消费者 / 六槽共 22 项、native 实际运行、Clippy / fmt 和适用文档 / 债务门禁已通过，见 [DCL-37](DEBT_CHANGELOG.md#dcl-20261004-37--独立-host-接入原生六槽)。这两片相关范围至此稳定，在干净最终 SHA 做一次完整本地链与一次推送，远端必须核同一 SHA；不继续追加公共消费者验证或宣称生产 Host 已完成。下一片的生产绑定保持独立：只接有实际资源 / 材料和已确定语义的能力，不擅自开启丰富 Event / Agent 的额外模型或工具授权。

### D-CLI-BLUEPRINT-05 · 参考 Host 已装配正文模型的 Base 接口（2026-10-05）

**基线 / 原因 / 尺寸**：L 的增量公共接线，起点 `b4d0cc17cfb731a2be0fab1e5b5cf1b30fc18616` 干净且已推送；上批完整链与精确 SHA 远端证据随这次实质改动入账。共享消费者能用六个 Base，但参考 Host 现有最小路径只在内部调用 `AppState.llm`，开发者不能把该已装配客户端绑定给消费者；不应为此另建 Ollama 客户端并绕开实际资源 / 授权包装。本片只开放该真实正文调用的 Base 视图，不迁移记忆来源、产品 Event / Agent、ChatPro 请求或角色状态。

**冻结行为 / 影响链**：`OcliveKernel::text_generation_base()` 借用 Host，构造时零调用；被 poll 时复用原最小文本入口的当前用户模型设置和 `generate_with_opts`（options 仍为 None），原输入不改，一次调用已装配客户端，不承诺其内部只有一次 provider 请求。正常空文本保留。Base 边界按 typed Host 错误投射：授权拒绝 / RemoteServiceUnavailable 为 Unavailable，其余 Failed，诊断原文保留；旧入口仍返回原 AppError，不 round-trip Base 错误。现有 Host 错误没有 typed 超时 / 取消，故不从文本制造 TimedOut / Cancelled。没有角色人设自动注入、额外探测、预热、重试、落库或新模型选择；实际 client 自带策略继续由它负责。生产者是原客户端，契约沿 LlmBase，适配是该薄视图，消费者为核心外公共消费者；状态 / 回退沿旧 Host 入口。Tauri / shared / ChatPro / Theater / 插件 / 角色内容和姊妹仓的 wire 不变，无需关联修改。

**写集 / 有限验收 / 停止**：`host/src/domain/chat_engine/process_message.rs` 抽出唯一正文 helper，新 `minimal_llm.rs` 与 `mod.rs` 只做薄视图，`role_kernel.rs` 增量 facade，原 `tests/minimal_role_public_api.rs` 补真实 builder + 内存客户端回归；文档只动 ROLE_PACK_BOUNDARY、MODULE_MAP 接线归属、父债第五列、本计划与 DEBT_CHANGELOG。先红测，再核同一 PreparedMinimalRole 与实际 Host LLM 接入共享六槽、输入 / 正常空值 / 失败 / 构造零调用、旧入口兼容；Event 分析和正文两次调用只在测试显式选择，产品默认未变。跑相关 Host 集成 / lib、Clippy / fmt、workspace doctest及适用分层 / 模块 / 文档 / 债务 / 编码门禁；controller 直接实施和自查，不称独立审查。窄测后先本地保留，与下一片已确定的生产材料消费关联冻结，只在批次出口一次完整本地链与目标远端；若下一片出现真正的材料 / 任务策略待决则询问，不扩大调查来拖延本片。父债仍 Partial，达到实际客户端复用即停，不启动真实模型 / 服务 / 音频 / 用户库或历史身份。

### D-CLI-BLUEPRINT-05 · 最小会话的当前对话材料接入 Memory（2026-10-05）

**确认 / 基线 / 尺寸**：维护者已选择仅使用当前最小绑定的临时对话，切换不跨角色共享、不接旧丰富记忆，不新增持久身份 / 历史恢复。L 的 Host / DTO / 前端能力链，起点 `3ae400bd90d515d1d9fc15cca4c91397e660e0da` 干净且前一片 Locally verified、未推送。共享运行库与六槽契约不变，不把临时检索冒充长期记忆或六槽全接通。

**行为 / 影响链**：共享最小 store 在发送当前用户消息前，快照当前绑定已成功完成的 user/reply 对；失败、取消、在途和其它角色消息不进入候选，提交事件监听器后续修改不能改变快照。新增可选 reference Host 传输字段 `conversation`（默认空），每对只含原 user_message / reply，无身份或状态包；薄 Rust IPC / 已鉴权 HTTP 原样承载，无新命令和授权。Host 固定选择已有 QueryMemoryRetrieval 的 Base 视图，以本次用户输入检索带明确原发言人标签的候选；真实选中内容进入共享 Prompt 人设 / 材料准备再到原正文调用，人设一次、正文仍一次。没有命中是正常空选择，不制造记忆或回退到丰富路径；旧无字段请求保持原 Prompt 字节。不会自动启用 Emotion / Event / Agent 或额外模型 / 工具。

**有界材料政策**：这是参考 Host 的请求预算，不是最小角色作者格式。最多最近 8 个完整对话对、合计原 user/reply UTF-8 64 KiB；前端取能在这个字节预算内连续容纳的最近后缀（不裁文字、不绕过过大的最新对来拼更早材料），Host 独立拒绝越界载荷，先于本地文件加载 / 模型调用。正常空回复仍构成已完成对。资产 / 人设原预算保持。上下文由已鉴权调用者提供，标记为引用材料不保证模型防注入，不能推导存储真实性。

**写集 / 验收 / 停止**：types `models/dto/minimal_role.rs`、host `domain/chat_engine/process_message.rs` / `mod.rs`、`http_api/chat.rs` 与原 `tests/minimal_role_http.rs`；shared `api/chat.ts` / `api/minimalRoleChat.test.ts`、`stores/minimalRoleChatStore.ts` 及测试、`integration/minimalRoleMainFlow.test.ts` 按真实请求断言需要改，app.zh/en 范围提示同步当前会话与持久历史的区别。兼容复核发现直接给旧 public DTO 加 Vec 会破坏 Rust struct literal，因此保留旧 DTO / 本地入口，新增 conversation 封装与转换；Tauri `api/chat.rs` / `api/chat_backend.rs` / `kernel_attach/chat.rs` 和 Host 原路由仅改类型接线，原命令、路由和鉴权不变，旧 backend / HTTP 方法保留。现有桥内纯测试核旧 literal、空候选 wire 等价和引用文本；不改 ACL 或历史 B6 证据。文档只同步角色边界、MODULE_MAP 引用、父债第五列、本计划与变动事件。先在旧源码 red，再验材料选择 / 原主体与否定 / 无命中 / 旧请求、预算越界先拒绝、当前会话第二轮 / 新绑定 / 取消失败 / 晚结果 / 快照及最近后缀；用真实 store 与实际 HTTP router，模型 / IPC 为内存替身。适用 Rust / 桌面纯合同 / 前端、doctest、Clippy / fmt / typecheck / lint / 模块 / 文档门禁后自查、本地提交；与上一片合批冻结时一次完整链和目标远端。达到 Memory 原材料消费即停，持久化、其它槽默认策略、媒体、模型质量和全发行版不扩查。

### D-CLI-BLUEPRINT-05 · 当前输入的 Emotion Base 线索消费（2026-10-05）

**基线 / 原因 / 责任**：起点 `c83e501259ab5793d19b27767ab27d97d2b5f77d` 干净，上一批实际正文客户端与临时会话 Memory 已全量收口，精确结果随本片实质变更入账。共享六槽消费者已有 Emotion 调用面，但参考最小主流程尚未消费分析。只沿 canonical 本地最小入口显式选择已有 BuiltinUserEmotionAnalyzer 的 Emotion Base；这是参考 Host 的有限选择，不是要求所有发行版使用词表、替换 rich 配置或在所有回合调用 Emotion。按 L 关联批次和材料语义的重风险深度留计划；controller 实施与专项自查，不称独立审查。

**冻结行为 / 闭环**：当前请求原文 → 已有无模型词表 Base（context=None）→ 完整线索报告 → 明确标记为参考分析的 Prompt 材料 → 原正文客户端。仅分析当前用户材料，不分析人设、检索出的历史或旧 rich 状态；不剥离报告的主体 / 引述 / 条件未判定、否定启发式等限制，不生成七维或情绪状态。正常 None 不加占位或中性标签，并保持该情况下原 Prompt 字节；线索存在时与当前会话引用分区，当前原文与人设仍保留一次。一次分析不增加模型调用；旧直接进程内基础 / 自选 Prompt 接口保留原行为，HTTP / IPC 原 wire、鉴权、响应、临时会话与可返回完整角色上下文不改。固定实现的真实加载失败保留诊断并停止正文，不吞成 None、回退或重试，不定义通用 Base 到 wire 的错误协议。

**精确写集 / 有限验证**：Host `src/domain/chat_engine/process_message.rs` 与 `tests/minimal_role_http.rs`；runtime `src/domain/base_emotion.rs` 只修正当前产品消费范围注释，原分析算法 / 词表 / 七维端口不改。文档只动 ROLE_PACK_BOUNDARY、MODULE_MAP 的接线引用、父债第五列、本计划、DEBT_CHANGELOG。先以实际鉴权 HTTP 路由新增红测，核完整报告进入真实模型输入、引述与否定原文保持、历史材料不作为当前情绪、无匹配后不残留上一回合线索、原响应 / 单次正文 / 零 rich 激活及模型失败保留；实际模型用内存记录器。跑 Host HTTP / 公共 API / lib、既有 Emotion 外部回归、Host+runtime Clippy / fmt、workspace doctest及 applicable 分层 / 模块 / 文档 / 债务门禁；这片不改前端与桥类型，既有主流程调用继续消费原响应，按 G17 核读无需修改。适用局部门禁后本地保留；只有关联里程碑稳定才一次完整链与一次目标远端，不每个测试都推送。达到真实材料消费即停，不扩查情绪准确率、全发行版或媒体。Event 若需要额外模型预算、Agent 若缺合法任务，先单列待决，不填假操作。

**局部出口 / 维护者选择**：本片相关能力链与兼容回归通过，见 [DCL-40](DEBT_CHANGELOG.md#dcl-20261005-40--当前用户材料的-emotion-base-线索消费)。维护者已选 Event 按需调用、普通聊天默认不追加分析；下一片只提供参考 Host 已装配模型的显式 Event Base 入口，调用方自选材料 / context 并承担单次额外分析，不将其加入本地最小聊天或新增 UI 策略。Agent 不以默认计数任务填充主流程。

### D-CLI-BLUEPRINT-05 · 参考 Host 的按需 Event Base 入口（2026-10-05）

**确认 / 基线 / 尺寸**：维护者选择 Event 按需调用、普通最小聊天默认不追加分析模型调用。Emotion 片已在 `9fcb4bdd9256b9603b836143124cf6693d2b70dd` 干净本地保留，尚未推送；本片与它合批。L 的增量公开 Host 接线，按公共 API 重风险深度审阅，controller 实施 / 专项自查，不称独立审查。无需再次从六槽概念或旧 rich Event 重新调查。

**行为 / 闭环**：新增 `OcliveKernel::event_analysis_base()` 返回借用 Event Base，构造时零调用；只有调用方明确 poll analyze 才将当前请求的 material / context 原值交给已有 LlmEventAnalyzer，分析生成器复用 HostTextGenerationBase 的已装配客户端 / 用户设置 / typed 错误投射。一次 analyze 只发起一次该客户端调用，无 adapter 重试、预热或第二模型；客户端内部策略仍归其自身。原独立分析器的协议、正常 Some / None 与失败完整保留，格式失败不能吞成 None 或自动用普通回复冒充分析。没有自动注入角色人设 / Memory / Emotion，没有要求 caller 必须持有完整角色；现有共享六槽消费者可显式绑定此视图并把结果用于其选择的后续操作。材料与报告不发布事件、不改变数值 / 关系状态，不授予 Agent 或工具权限。此入口是参考 Host 的有限协议选择，不证明任意真实模型适合分析；不是 ChatPro UI / IPC 的新增分析操作。原普通最小 HTTP / IPC 与进程内文本方法均不调用此工厂或分析。

**写集 / 验证 / 出口**：新增 Host `src/domain/chat_engine/minimal_event.rs`；仅在同目录 `mod.rs` 登记私有适配、`role_kernel.rs` 增量 facade 及 rustdoc、原 `tests/minimal_role_public_api.rs` 加实际调用；runtime `src/domain/base_event.rs` 只更正当前 Host 绑定范围注释。ROLE_PACK_BOUNDARY 新 §0.18、MODULE_MAP 接线引用、父债第五列、此计划与 DEBT_CHANGELOG 同步，不改 types / contracts / 公共请求 / wire / ACL / 依赖 / state / 模型选择 / 六阶段编排。公开 API 先编译红，后核生产 builder 与原已装配内存客户端：显式构造零调用、分析 Some / None 与 None / 空 / 有 context 原值、Unicode / 引述 / 条件原文、协议失败与模型错误保留 / 不重试、共享六槽真实绑定、原 HTTP 单次正文保持。跑 Host lib / HTTP / 公共 API、原 Event 外部 / 六槽 / 消费者、Host / runtime Clippy / fmt、workspace doctest及适用文档 / 分层 / 模块 / 债务门禁；合批冻结最终干净 SHA 后一次完整本地链、一次推送、核该 SHA 正式 CI，不为绿灯回写另造提交。父债 Partial，Agent、真实分析质量、真实音频、所有发行版及 UI 分析操作不扩面；达到合法显式绑定即停。

**局部出口 / 当前停止线**：上述实现与有限回归已完成，实际结果见 [DCL-41](DEBT_CHANGELOG.md#dcl-20261005-41--按需-event-复用参考-host-的真实模型资源)。本片与 Emotion 两片到达关联冻结点：只完成总审、干净提交与一次批次完整链 / 精确 SHA 远端验收，不在等待中追加 Agent、UI 分析或低收益反例。六槽共享消费和参考能力绑定不替代所有发行版的具体材料、任务与资源接线，父债仍 Partial。

**2026-10-05 精确批次收口**：Emotion / Event 已冻结在 `fb26a15c2338c7dc3daecc62ce931b8b4fc0a663`，一次完整本地链 native exit 0（946.442798 秒，环境恢复、HEAD 未变、工作树干净），一次推送后的[正式 CI 37266206907](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37266206907) 对同一 SHA success、17/17 含 ci-gate，零 rerun。本机原始收口为 `.cursor/plans/debt-minimal-role-emotion-material-20261005-r0/55-closeout.json`，忽略目录不是 Git 携带证据。结果随下述实质切片入账，不另造绿灯回写提交。

### D-CLI-BLUEPRINT-05 · 已有真实 Agent 的显式 Base 借用（2026-10-05）

**起点 / 原因 / 尺寸**：上述精确干净基线。L 的公共增量接线，按公开能力与工具边界的重风险深度计划；controller 直接实施与语义 / 权限自查，不称独立审查。L1 已找到真实 `BuiltinReActAgent` 及其经过旧合同回归的私有 `HostAgentBaseView`，无需新增执行器或默认计数任务。外部 Rust Host 已可构造并持有真实 Agent 及资源，但不能借用该报告视图，容易被迫复制旧输入投影 / 报告逻辑。本片只补这个调用断点。

**冻结行为 / 闭环**：在现有 `BuiltinReActAgent` 增量提供 `task_execution_base(model, role_id, session_namespace)`，借用当前实例和调用方实际身份，构造零 discovery / 生成 / 工具调用；执行沿唯一既有 ReAct core 与原 branch-facts 报告，不构造丰富 `AgentInput` / 默认领域状态。资源 / parser / bridge 为该实例既有装配，授权不变、身份不授予权限。保留原 context 范围（None / 空可用，非空 Unsupported 且先于资源调用）、typed 错误投射、最大三轮和报告限制；正常模型回答不保证目标达成、无效应或重试安全。共享最小六槽消费者可显式绑定此视图；不固定调用顺序、不暗加人设或任务。这里开放具体 builtin 的已存在能力，不增 `OcliveKernel` 自动选择 / 重新装配 Agent 工厂，不声称任意 rich / remote / directory provider 都有 Base，也不迁移 ChatPro 普通聊天或旧短路逻辑。

**精确写集 / 有限验收**：Host `src/domain/agent.rs` 添加公开方法及可编译用法、`src/domain/agent_base_binding.rs` 只更新私有实现与公开入口的范围说明、新 `tests/minimal_role_agent_base.rs`；文档为 ROLE_PACK_BOUNDARY 新 §0.19、MODULE_MAP 接线引用、父债第五列、本计划及 DEBT_CHANGELOG。先在旧源码让外部调用编译红，再用真实 builtin / 原生产 parser、内存 LLM 与授权桥 seam 验共享消费者明确委托、实际模型 / trace 身份 / 两轮一工具、非空 context 零资源调用、无工具 / 空任务如实未承接、typed discovery / model 错误保留且不重试。已有 Agent core / fallback 回归、当前 HTTP / public API、runtime 六槽 / 消费者作兼容检查；公开 doctest、Clippy / fmt 与 applicable 分层 / 模块 / 文档 / 债务门禁必跑。达到已有实现外部可用即停止，不重做内部三轮 / 并发 / 全部错误反例或开启真实 MCP / 模型 / 用户数据。局部通过先形成可审查提交，同主题批次出口一次完整本地链与一次目标 SHA 远端；后续产品真实任务操作 / 选择策略若有取舍再询问，不把 Agent 工具并集、CLI 磁盘生成或全发行版验收塞入本片。父债保持 Partial。

**局部出口 / 冻结点**：上述公开借用与真实共享消费已实现，适用结果和本轮格式自纠见 [DCL-42](DEBT_CHANGELOG.md#dcl-20261005-42--已有真实-agent-向外开放-base-借用)。本片到达有限里程碑：总审后干净保留，随后一次完整本地链 / 推送 / 精确 SHA 正式 CI；期间不追加产品任务策略或低收益边缘证明。后续若进入任务 UI、当前 Host configured provider 的选择工厂或扩大工具集合，另核实际需求和权限归属，不把本片借用提升为迁移授权。

## 2026-10-05 · 冻结后并发接续：CLI 市场基础安装闭环

**基线 / 授权 / 分工**：维护者要求先冻结最小运行里程碑，再分发独立支线并继续偿债。基线 `f849e2a95e3a5035fc70045e87c13ebe2fbb28c7`；主工作树继续精确 SHA 的 CI 收口，接续片在该 SHA 的独立工作树准备，未验收前不改主线身份。三个有限 L1 分支分别复核已实现 Agent、核下一条债务、说明 ChatPro 任务待决项；不递归委派，不重做全仓调查。技术债与公共文档由 controller 单写；实际源码与市场文档的写集不相交。

**具体问题 / 风险**：`V-MARKET-01` 仍 Partial，旧 Minimal 文档任务不重新领取。Git 市场分支目前在读取 manifest 前删除已有目标并移动源码；既有 `plugin_ext::run_install` 已有 JSON 对象 / 依赖检查，但其依赖图根取自目标而非本次 source，同目录复制还会删源。本片 M，安装目标写入按重风险复核；只补已有基础安装承诺，不实施签名、可信发布者、Full 社区或完整 Host 清单校验迁移。

**闭环与冻结行为**：索引条目 → 已克隆本地根 / `gitSubdir` → 原 CLI manifest / 依赖校验 → 目标安装 → 结果 / 临时目录 → 新临时夹具。校验、依赖缺失或环检测失败均先于已有目标变更；本次 source 作为图根的事实来源；默认 source 与 target 是同一目录时不能删除自身。保留现有索引、克隆、非 Git 安装和显式角色装配语义。不假定已有 helper 提供了完整 Host schema / 身份检查；发现需要新增共享校验公共接口、默认 Host 依赖或新身份政策时先回 controller 重规划，不复制第二套完整 parser。

**身份最小补充（controller 已裁定）**：市场 Git 入口以既有 `serde_json::Value` 读取 manifest 非空字符串 `id`，在目标变更前与索引 `item.id` 精确相等；这兑现索引已有身份一致承诺，不新定义清单 schema / ID 语法 / 信任政策。JSON 根和依赖继续交同一个 CLI 安装 helper 校验。缺失、类型错误或不一致均拒绝；这项基础比对不声称等同桌面运行时的完整清单检查，也不以此关闭完整安装安全或发布债。

**写集 / 并发**：实施者仅写 `oclive-cli/src/market_cmd.rs`、`plugin_ext.rs` 和其中直接回归；controller 写本计划、DEBT_CHANGELOG 与父债第五列。另一文档支线仅修 `GITHUB_PLUGIN_INDEX_LINE.md` 的默认 URL / 同步示例、`PRODUCT_LINE_TASK_BUCKETS.md` 的重复状态引用，不提前重写安装协议。最小角色代码与合同、六槽 / Kernel、wire、签名与权限不改；`K-DIST-01` / `V-MARKET-01` 旧自动计划和其它冻结项不解冻。

**有界验证 / 出口**：仅新 TempDir 和必要的本地 Git 夹具，零外网 / 真实插件进程 / 用户插件目录。先保留 missing / bad manifest、source 根和同目录问题的失败，再核有效根 / 子目录、原目标 sentinel、依赖拒绝与既有路径。开发用离线 `cargo test --locked -j 1 -p oclive-cli` 定向过滤、局部 fmt、Clippy / diff；独立 reviewer 核验证先于目标写入、source 身份及临时路径归属。工作树共享既有 Cargo target 时串行执行，不装依赖或删缓存。文档合流后跑适用链接、登记、债结构与编码；批次冻结后才一次完整本地链 / 推送 / 精确 SHA 正式 CI。失败只修具体根因，不绕过测试或扩展完整安装安全矩阵；足够支撑本片行动即停，父债保持 Partial。ChatPro 明确任务、工具面、模型与报告消费尚未选择，不由本片替维护者决定。

**同轮接手文档收敛**：controller 登记这批状态时，只将已触及的 `D-CLI-BLUEPRINT-05` 累积长状态行按 `D-DEBT-LEDGER-01` 既有准则分离：新历史快照逐字保全基线原行，前四个登记单元格保持，现行第五列保留 Partial / 已有能力 / 剩余范围，详细逐次证据链出而不继续累加。新快照为 `handoff/archive/TECHNICAL_DEBT_MINIMAL_ROLE_STATUS_20261005.md`，不是第二份活跃状态 SSOT。旧归档不改，不改父债结论或领取更多清单调查；核 raw 行相等、前四列相等、唯一状态行及适用文档门禁。当前 Agent 精确 SHA 的完整结果只在真实终态核对后随本次实质片入账，不制造纯绿灯提交。

**合流与停止线（2026-10-06）**：两支线已实施并分别保留，安装最终方案有限独立复核通过，controller 主树 13/13 与适用 Clippy / fmt / 模块 / 文档 / 债务 / 编码 / diff 全部 native 0，详情与自主纠偏见 [DCL-43–44](DEBT_CHANGELOG.md#dcl-20261006-43--cli-市场在替换旧安装前共用基础检查)。最小角色归档原行相等、三个触及债项前四列相等；父债仍 Partial。源码、文档与接手记录形成同一相关批次的干净冻结，随后一次完整本地链和一次精确 SHA 正式 CI，不把上游 Agent 绿灯借给本批。实现已足够支撑本片，停止扩证；真实市场安全、完整 schema / IO 事务及新的 Agent 产品选择不在本批自动开工。

### K-UID-DEFAULT-02 · 发行版默认身份的实际优先级（2026-10-06）

**基线 / 用户决定 / 风险**：`cfcfc7343063d4eba7364523e59117fe68dc4b1e`，主线干净，市场批次正式 CI 尚在收口；本片另开独立工作树，未取得上批终态前不改主线或推送。维护者已两次明确：用户显式选择优先，无显式选择时采用发行版默认 A；A 不在当前角色 catalog 时退回合法角色默认 B，不伪造 A，显式无效选择仍按原 API 拒绝。本片 M / 重风险（用户身份及关系路由），controller 实施与语义自查；独立 agent 的后续只读规划触及用量限制，不借此声称取得本片独立复核。

**冻结闭环 / 行为**：HostProfile 配置 → 现有角色 catalog + global / per-scene DB 选择 → 同一个 Host 私有默认选择 helper → 回合模板 / relation 映射 → 身份状态 DTO / 恢复默认 → UI 默认态。合法 DB 显式选择仍最高；默认 helper 只选择 catalog 实际存在的 profile.default_id，否则 catalog 默认，再保留原无 catalog 的 legacy 提示。global 初始态、恢复默认和 per-scene 无覆盖均使用这一规则；per-scene 默认标记不被另一 global 选择误读。状态 DTO 的 default_identity_id 与 current_identity_id 反映该规则，恢复 global 默认按原同步方法更新 relation，UI 只改“跟随包默认”为“跟随默认身份”。不新增 schema / DB 字段 / SessionCache 状态，也不删除历史 default flag / sentinel wire 名称；allowed_ids 继续约束 API 显式选择，不扩大选择权限。

**写集 / 有界验收**：Host `domain/user_identity_loader.rs`、`service/role/identity.rs`、新直接回归；既有 HostProfile 配置解析测试作回归而不重写 parser。shared 两语种 roleRuntime 文案及既有 useUserIdentityState 测试；中英文 DISTRO_CAPABILITY_PROFILE / ROLE_PACK_SPEC 的实际默认说明，父债第五列、本计划及 DCL。先用新临时角色目录 / 内存 SQLite / 禁止模型调用的资源 seam 红测正常默认、显式选择 / 恢复、两 scene 隔离及无效 / 未配置 profile 回退；service DTO 与实际模板 / relation 一致，未知显式选择拒绝。用既有 frontend consumer 的默认 ID 回归核 UI，不以 mock 当实机。适用 Host lib / Clippy / fmt、前端窄测 / lint / typecheck、文档镜像 / 链接 / 登记 / 编码 / 债结构 / diff；若改变公共 API 再补对应 doctest，不把私有 helper 当新 Base。达到这条选择规则与直连 API 闭环就停，不扩成人授权策略、其它旧关系回退、全身份 DB 历史迁移或真实用户数据调查；关联批次出口一次完整本地链及目标正式 CI。

**关联注释补充**：`oclive_kernel_types` 既有 DTO / sentinel 的 rustdoc 也不应继续把有效默认称为 pack-only；只更新 `models/dto/identity.rs` 的字段注释与 `models/dto/chat.rs` 的身份 sentinel 注释，值、字段、类型、serde 和 relation sentinel 均不改，不构成新公共 API。测试夹具显式设置磁盘 identity_binding 并核实际加载值，避免 exporter 默认 per-scene 被误当 global；第一次暂态 5/6 失败来自这一夹具错误，不归因产品。修正夹具后旧源码 native 101（1 过 / 5 败），实现后六项全过，原输出保留。

**局部结果**：Host lib 637／身份直接回归 6、shared 身份消费者 3、Host all-targets/all-features Clippy、局部 fmt、前端 ESLint/typecheck 与适用分层／模块／文档／债结构／编码／diff native 0；[DCL-45](DEBT_CHANGELOG.md#dcl-20261006-45--发行版默认身份兑现用户确认的优先级)区分夹具错误、真实红测与实现证据。停止于这一规则，不将窄测当目标正式 CI；批次冻结后只运行一次完整本地链并推送。

**完整门禁纠偏 / 当前出口**：上批 `cfcfc734` 正式 CI 已实际 17/17 success，收口回执保留后才准许本片合流。身份原冻结 `ee1b0939` 的完整链在 Dimension 5 旧路径检查 native 1，原因仅为本人新 TempDir 测试局部变量不符合已有夹具写法；限定改 `root` 为 `dir`，不改实际目录或门禁。原失败证据不覆盖，窄测修复后冻结最终 SHA 再跑一次完整链；本批因此有一次真实失败，不再称总计只运行一次。此前 M 实施在最终债务验收升 L，仍不扩调研或推送中间失败提交。

**2026-10-06 实际终态**：身份批次冻结于 `58be11faba622dc87ed29ef677ea444cf2dd6599`；第二次完整链被控制端向嵌套生成项目传播 `CARGO_TARGET_DIR` 的错误环境配置拒绝，未改业务源码。取消该全局覆盖后，CLI monolith 定向 8/8，第三次完整本地链 native 0（1128.6415517 秒），一次推送后的[正式 CI 37353477334](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37353477334) 对同一 SHA 17/17 success、含 ci-gate，零 rerun；总计三次完整本地尝试，两次失败原件保留。本机 `.cursor/plans/debt-user-identity-default-20261006-r0/55-closeout.json` 已核 HEAD / tracking / remote 一致且干净；忽略目录证据不可随 Git 转让。本条默认选择契约 Done，不扩 legacy 全历史迁移；随下述实质切片回填，不制造仅回写绿灯的提交。

### D-SCAFFOLD-RESOURCE-01 · 只读资源候选预览（2026-10-06）

**起点 / 原因 / 尺寸**：上述干净且正式验收的 `58be11faba622dc87ed29ef677ea444cf2dd6599`。父债已有维护者独立实施授权；L1 限于既有 CLI doctor、ResourceAdapterRegistry、HostProfile 和资源纯编译器，已足以开工，不再枚举资源故障矩阵。M 实施，按 Host 共享语义接线审查；批次保留／main 推送出口升 L。controller 实施与只读语义自查，不称独立复核。

**冻结闭环**：Host 捕获的现有 `ResourceCoordinationDiagnostics` JSON → canonical types 反序列化 → 原 ResourceAdapterRegistry 的所有者／descriptor 校验与有限调度 validator → 原 `compile_resource_candidate_plan` → CLI 人类输出或原 `ResourceCandidatePlan` JSON。新增 `doctor resource-plan <diagnostics-file>`，显式要求已有 `diagnostics-host` feature；可用 `--distro-profile` 经原 loader 替换捕获 policy，旧 profile 缺资源节继续用原默认；可显式选择 `--gpu-device-index`，不偷偷套入本机环境覆盖。输入必须为当前诊断版本，错误读取／类型／所有者／重复 adapter ID 非零；blocked/degraded 是成功生成的诊断，不冒充工具失败。

**控制边界**：只读捕获不携带真实控制器，编译时 controller_ids 为空；不能把 descriptor 的 managed 或 selectable 宣称为实际授权。需转换的候选会保留原 controller 缺失原因；无转换候选的原 executable 字段也不构成实时准入。stdout JSON 仅一个原有类型的文档，stderr 明示离线捕获／无控制权；不把候选写回角色包或新造资源磁盘 schema，不启动／探测设备、插件、模型、进程、服务或用户 DB，不改 policy 执行、六槽、Host wire、桌面／共享消费者与默认 CLI 依赖面。

**写集 / 验收 / 止点**：CLI `src/main.rs` / `doctor_cmd.rs` 登记子命令，新 `doctor_resource_plan.rs` 与其定向测试；中英 CLI guide、父债第五列、本计划、DCL，同轮回填上一片身份验收。测试覆盖实际命令解析／JSON 与人类输出、临时文件/profile round-trip、legacy profile、无 GPU、observe-only、冲突计划、需控制器不能伪造权限、坏输入与默认 feature 明确拒绝；读取前后源文件不变，引用旧 scheduling/candidate 结果不作为新结果。跑 CLI 两种 feature 的窄测／Clippy／fmt，Host registry/planner 兼容回归及适用分层／文档／镜像／债／编码／diff 门禁；达到离线 preview 即停止，不新增交互编辑器、自动硬件建议、第三方装配或实机 soak。父债只 Partial，不全关。相关片稳定后冻结干净提交，一次最终完整本地链、一次推送、核目标 SHA 正式 CI；全局不导出 CARGO_TARGET_DIR，保留失败而不覆盖重跑。

**关联文字写集**：资源 RFC 中英各一处将泛称 CLI doctor 精确到旧 `doctor execution-plan`，只链接新离线入口，不复制合同或改变控制政策；COVERAGE 与 DCL 依赖表将已验收身份债移出待决项，主台账前四列仍逐字保留。无需修改 Host／types／contracts／Tauri／shared／发行版／目录插件／官方角色包／姊妹仓；没有新公共 Rust API 或依赖。原 JSON `executable` 语义不改，CLI 不提供运行候选的入口。

**局部出口**：实际 CLI opt-in 7／默认拒绝 1、Host compiler 8／registry 9／旧 CLI 1 项，Clippy／fmt 和适用文档／分层／债结构均通过；零测试假红测、原命令不存在的真红测、初始 fmt 与控制端参数错误各自保留并归因，见 DCL-46。当前只完成局部验证／controller 自查；在本片有限 milestone 总审、提交冻结后运行一次最终全链及正式 CI，不扩配置编辑、资源执行或实机压力测试。

**正式覆盖接线**：现有 CLI job 的 opt-in `diagnostics-host` 步骤只运行库消费者回归，默认全 crate 不会执行新 feature 内的七项；该原步骤增加 `--test doctor_resource_plan`，保留原 `e2e_minimal_role_library`、串行、default 依赖边界、现有 job／矩阵／ci-gate。这是本命令测试进入现有门禁的必要关联写集（`.github/workflows/ci.yml`），不重构 CI；本地按同一条命令验两个集成目标，最终全链负责 Dimension 5／工作流纪律。

### D-SCAFFOLD-RESOURCE-01 · 非交互资源策略草稿（2026-10-06）

**基线 / 原因 / 风险**：`9edfe719d71e32edf27bd6a482a0e56a3c52b403`，开工前主树干净。该片完整本地链 native 0，正式 CI 首轮 Linux runner 收到 shutdown signal／lost communication，五个取消 job 未取得 hosted runner；CLI job 成功。保留原日志后仅补跑一次 failed jobs，不放宽门禁；补跑未终态前不称已验收，也不推下一批取消它。当前缺口是开发者只能手工改 profile，本片先兑现非交互编辑／round-trip，M 实施、最终 main 里程碑升 L，controller 实施与语义自查，不称独立复核。

**冻结闭环 / 写入边界**：新增 `config resource-policy`，必须显式给出发行版 profile、只含原 `[resource_coordination]` 节的 TOML patch、现有诊断 capture 和新输出路径。通用 TOML 值只负责编辑结构；合并后仍由原 Host loader 及已有 preview 的 registry／有限意图 validator／候选编译器验证，不造第二套 typed schema 或 parser。只覆盖 patch 中给出的资源键；`commands` 给出时整体替换，空数组显式清除，缺省键与所有非资源节／未知扩展的 TOML 值保持。序列化可能重排键、丢失注释，所以只生成新草稿，禁止原地修改或覆盖已有输出。先在输出父目录准备临时文件并验证，再以 `persist_noclobber` 发布；所有拒绝不得留下最终草稿或损坏已有文件。

**有限验收 / 停止线**：以新 TempDir 的实际 CLI 子进程证明 legacy profile 增补、现有策略局部修改、原 loader clamp、commands 清除、非资源值保留、草稿再经 `doctor resource-plan` round-trip；坏类型／枚举、跨节 patch、非法或冲突 intent、坏 capture、已有输出、默认 feature 明确拒绝均非零。observe-only／无 GPU／无 controller 的有效但降级配置可以生成，打印原原因码，不把候选当执行许可。原 profile／patch／capture 字节保持，零真实资源／Host 启动／设备／模型／用户库。达到这些正负例即停，不进入交互向导、自动硬件建议、实时控制、通用原地编辑器或资源父债的全部矩阵；重复确定性失败回 controller，架构／权限新取舍才问维护者。

**精确写集 / CI 节奏**：CLI `src/main.rs`、`config_cmd.rs`、`doctor_resource_plan.rs` 与新 `config_resource_policy.rs`；原 preview 测试共享 capture fixture、新 `tests/config_resource_policy.rs` 及 `tests/support/resource_capture.rs`。中英 CLI guide、本计划、DCL、父债第五列；现有 `.github/workflows/ci.yml` opt-in 步骤增加新集成目标，既有 job／默认构建不改。先红测／窄测、两 feature Clippy／fmt、默认 CLI 回归和适用分层／文档／债结构／编码／diff；同一相关片冻结后一次完整本地链、一次推送、精确 SHA 正式 CI。Host／types／contracts／runtime／Tauri／shared／发行版与角色包源码、依赖和资源执行语义无需改；上片正式结果只随本次实质提交回填，父债仍 Partial。

### K-SUPPLY-12 · 2026-10-06 新公告与测试链修复

**起点 / 有限调查**：干净 `8cd7d5d1da36e9393cb67acc98ae12e107560875`。正式 CI `37424944008` 的 npm-audit 原始 job 日志明确为生产图三项 high（Vue/server-renderer 与 source-map-js），不是 runner 中断；本机完整 JSON 扫描另有 tinypool/Vitest critical。仅核公告修复版本、实际声明与锁定依赖、三个现有 Vitest 消费者；达到这些施工事实即停，不开展全库攻击面证明。旧资源草稿的本地完整链仍有效，但其正式验收不可标绿。

**闭环 / 写集**：上游补丁 → 根及三个 workspace 的合法版本声明 → npm 锁定树 → 生产／完整 audit 与 peer → 原有 shared/ChatPro/Theater 测试、lint/typecheck/build → 正式门禁。精确写集为根 `package.json` / `package-lock.json`、`distros/{shared,chat-pro,theater}/package.json`、中英 `security/KNOWN_VULNERABILITIES.md`、台账 K-SUPPLY-12 与资源工具父债第五列、本计划和 DCL。Vue/compiler 保持同一 3.5 补丁线；source-map-js 只刷新原声明允许的补丁；Vitest 3 的 tinypool 1.x 没有可用兼容修复，本片选最小已修复的 Vitest 4.1.11，并核本仓 Node 22 / Vite 6 契约，不跳到 5、不强制 override 或降低审计等级。若需要广泛改写测试／产品代码，停止该升级并向维护者列明取舍；有限配置兼容另先列精确写集再改。

**门禁 / 出口 / 限制**：先保存原始 audit 非零与完整 JSON；锁差量核无无关包刷新、source 版本匹配、原树可达性。生产与 full audit 按 high 硬门禁，保留未修 low/moderate 的实际数字，不宣称零漏洞；npm ls、三 workspace 原测试、lint、typecheck、build 与文档镜像／链接／债结构／编码／diff。controller 实施与语义自查，不称独立 agent 审核。相关源码稳定后本地冻结，一次完整本地链及一次修复推送，精确 SHA 正式 CI；旧 run 不盲重跑，不以旧绿证明新 SHA。无 Kernel／Host／角色包／IPC／权限改动，无真实模型、TTS、用户 DB、外部业务流量；联网仅 npm 官方包元数据／依赖下载与 GitHub 证据。已有签名／TLS 架构暂缓不解除。

**工具失败后的有界重规划**：本机 npm 10.9.8 在两条不同锁更新命令均抛 Arborist `loadPeerSet` 的 `edgesOut` null，尚未写锁；保留两次原输出后不重复同工具盲试。仅此次锁生成使用 npm 官方 11.21.0（满足已安装 Node 22.23.2），通过临时 npm exec，不改变全局 npm、CI Node/npm 或仓库工具链契约；仍必须由原 npm 10 的 `npm ci` 消费所得锁及核 peer。若继续同根因失败或锁出现无关大范围刷新，停止本升级，不能改为 legacy-peer-deps／force／override。

### D-SCAFFOLD-RESOURCE-01 · 有限交互策略编辑（2026-10-06）

**起点 / 编排**：`3eb1ca7cafabedd739075c6a506fa9ecfcb95425`，在独立工作树进行本片开发，主树保持该供应链修复的完整链冻结状态；不改主树受测字节、不启子 Agent、不推中间提交取消在途 CI。M 实施、最终 main milestone 升 L，controller 实施与语义自查，明确不是独立复核。L1 已找到六种原 `ResourceSchedulingCommand`、四种 strategy、原新草稿 publisher；足够施工，不扩硬件故障矩阵。

**闭环 / 冻结语义**：既有 `config resource-policy` 增 `--interactive`，与 `--policy-file` 严格互斥。终端行输入 → canonical 四策略／六类有限约束与保留量 → 同一 TOML 结构合并 → 同一原 loader／registry／compiler → 展示有效意图／候选，明确 yes 后同一 create-new 发布。适配器与档位只列已核 capture 的事实，blank 保留原键，`-` 明确清除 primary 或 commands，替换命令使用 canonical enum 序列化，原 validator 仍唯一负责合法性与冲突。按行输入也可由 stdin 脚本化测试，不增加另一份交互业务实现；q／EOF／未确认／非法选择或意图都非零且无最终草稿，所有源文件保持。

**精确写集 / 验收 / 止点**：CLI `src/main.rs`、`config_resource_policy.rs`、新私有 `resource_policy_wizard.rs`、原 `tests/config_resource_policy.rs`；中英 CLI guide、父债第五列、本计划、DCL。原 CI 已覆盖该目标，无需改 workflow、Host/types/runtime/contracts/发行版源码、依赖或公开 Rust API。实际 CLI stdin 正例覆盖留原值／新策略／六类约束／clamp／clear／再经原 preview，负例覆盖取消／EOF／非法菜单／未登记与冲突／互斥／坏 capture／已有输出；保留原非交互八项、preview 七项、库消费者及 default feature 拒绝。适用 CLI 两 feature Clippy／fmt、分层及文档门禁；主树合流后冻结最终相关批次一次完整链／推送／精确 SHA CI，不拿工作树窄测当 main 验收。达到有限交互即停，不另造硬件建议策略、任意命令执行、磁盘 schema、控制权限、原地编辑或全部资源父债证明；需新产品／架构决策先登记留给维护者。

**运行安排调整**：供应链正式 CI 恢复执行后，先冻结此独立工作树的干净提交、在同一 SHA 跑一次完整本地链；不为等待远端改主树。工作树用原 npm 消费与主树逐字相同的锁；完整链直接沿用该工作树仓库配置解析出的外部 Cargo 产物目录，局部命令的显式 target-dir 不向后继全局传播。拟复用主树缓存的准备步骤发现该目录已经由生成项目测试建成普通目录，按保护规则拒绝覆盖（native 1、零目录改写）；因此不创建 Junction，也不移动／删除缓存或改配置。完整链没有其它本地 Cargo 写入，仍不导出全局 CARGO_TARGET_DIR。局部／完整证据分别明确 cwd，之后主树同 SHA fast-forward 且无漂移时可沿用本次完整结果，不再为合流重复全链；供应链批次正式终态明确前不推下一批取消它。

### K-SUPPLY-12 · 单一 selector parser 补丁预备片（2026-10-06）

**起点 / 原因 / 预算**：干净 `10af367d1ac348b11cb8f6c25620196fcb63f70a` 已完成资源向导完整本地链并推送，目标 CI `37434691111` 尚在运行；独立工作树保留该受测源。上一供应链片 `3eb1ca7c` 的正式 `37427668306` 已实际 17/17 success。L1 已核剩余 moderate 唯一是 `@antfu/eslint-config → eslint-plugin-vue → postcss-selector-parser 7.1.5`；[上游公告](https://github.com/advisories/GHSA-rj75-hqrm-r3gf) 与 [7.1.6 发行](https://github.com/postcss/postcss-selector-parser/releases/tag/7.1.6)给出同系列修复。仅做这一个补丁，不追全库可达性、攻击样本或 KaTeX low 到零，不按 audit fix 的 low 建议降级 ESLint 配置。M 局部预备、controller 实施和自查，无独立 reviewer。

**闭环 / 精确写集**：原合法间接依赖范围 → npm 定向 lock update → 原 npm ci / peer → audit 与真实 Vue lint / typecheck / build / shared unit → 记录后续合批坐标。仅根 `package-lock.json`、中英 `security/KNOWN_VULNERABILITIES.md`、台账 K-SUPPLY-12 第五列、本计划、DCL；无 package 声明、override、产品源码、断言、工具链、CI 或权限改动。锁结构差量必须恰为一个 parser 节点（版本/来源/integrity）；若刷新其它节点，停止并保全，不能手改锁或放宽 peer。前批已记录 npm 10 的锁生成确定性 Arborist 缺陷，沿用临时官方 npm 11.21.0 生成，仍用原 npm 10 消费。Node/npm 全局保持。

**验证 / 停止 / CI 节奏**：审计原始 4 low / 1 moderate 留作起点；本片预期 production 0、full 4 low / 0 moderate / 0 high / 0 critical，不能称 full 0。原完整 npm ls、Vue lint、typecheck、production build、shared unit 和 applicable 文档门禁，锁/public 声明独立 diff 检查；生成 bridge 如仅换行变化先保全并核相同 Git blob。通过后干净本地提交，仅记 Locally verified，不标父债 Done、不立即 main/push/全量 CI。后续相关实质片合批达到里程碑才一次完整本地链/精确 SHA 正式 CI，减少单叶补丁重复全 CI；不复用前一 SHA 的绿作为本片验收。主树现有 CI 期间零漂移，无真实模型/TTS/用户库/旧 CP-INT 身份；已有签名/TLS 等暂停不解冻。

**前片终态已取得后的事实回填**：向导 `37434691111` 已实际 success、17/17 含 ci-gate，主树/tracking/远端仍为干净 `10af367d`；watch 的 EOF 原件保留，不重跑 workflow。此实质片在同一台账文件的资源工具第五列、DCL 和安全滚动观察中回填前片精确终态，保持全部债务前四列与父债 Partial；这不是扩大产品行为或为绿灯单开提交。适用门禁依据本片六文件实际 diff，未触及 Rust/public API，不再跑未受影响的 workspace 全链。

### 供应链维护 · 撤回的 ChaCha20 补丁与 K-SUPPLY-12 合批（2026-10-07）

**起点 / 原因 / 限度**：主树与远端为干净 `10af367d1ac348b11cb8f6c25620196fcb63f70a`；本工作树干净预备提交 `fa579137f67a9d9e4cfaae90e9054a69d2b05809` 已保存单节点 selector parser 补丁。本轮只处理锁中 `chacha20 0.10.1` 的已登记 yanked 警告；Cargo 警告归中英安全 SSOT，不把 npm 父债 K-SUPPLY-12 或已完成的 event-listener 专项 K-SUPPLY-11 改写成 Rust 通用债；[上游变更记录](https://github.com/RustCrypto/stream-ciphers/blob/master/chacha20/CHANGELOG.md)及 crates.io 元数据给出未撤回的同线 `0.10.2`（MSRV 同为 1.85，修复 SSE2 后端误用 SSE4.1 指令）。调查到版本、原依赖约束和锁差量足够施工即停，不证明本产品实际曾触发 CPU 故障。当前 workspace/all-features/target-all 反向树未打印该依赖，不外推所有外部构建都不可达。

**写集 / 闭环 / 风险**：仅 `Cargo.lock`、中英 `security/KNOWN_VULNERABILITIES.md`、主台账 K-SUPPLY-12 第五列、本计划和 DCL。上游兼容补丁 → 原锁解析 → cargo audit → 既有本地完整链 → 精确 SHA 正式 CI；不改任何 manifest、override、源码、断言、feature、TLS/reqwest 版本、CI 或权限。M 局部实施，批次收口升 L；controller 实施和语义自查，独立复核=false。先保留原 audit JSON，再用 `cargo update -p chacha20@0.10.1 --precise 0.10.2`；预期唯一 package 的 version/checksum 两字段变化、依赖数组不变。超出该差量或需要新接口时停止，不以手改锁或全依赖刷新规避。

**验收 / CI 节奏 / 副作用**：新旧 audit JSON 分存、上游元数据、完整锁结构与清单前四列校验；适用文档镜像/链接/登记/债结构/编码/diff。已有 selector parser 的窄测证据保留，不重跑同断言凑数；二片提交冻结后在同一工作树一次 `npm run check:ci-local`，必要 Tauri bundle 按既有流程先准备，子进程使用现有 portable Python、不导出全局 Cargo target。完整链通过后一次 fast-forward/push，读取目标 SHA 正式 CI，不创建 green-only 提交。主线保持原已验基线直到完整链通过，父债保持 Partial；低风险 KaTeX 上游范围不兼容、签名、TLS、大版本和资源建议规则仍暂缓。联网仅官方 registry/advisory 与 GitHub，零真实模型/TTS/用户库/旧场景重放。证据目录 `.cursor/plans/debt-chacha20-patch-20261007-r0/`，失败与成功分存、不覆盖旧记录。

### D-SCAFFOLD-RESOURCE-01 · 原因码的人类说明预备片（2026-10-07）

**起点 / 缺口 / 路由**：独立工作树从冻结供应链 `67baa952bb7e1ff0ba987609d6fe729ca327f24d` 接续，原工作树正在完整门禁、主线仍保留 `10af367d`，不改变任一受测字节或取消该批验证。L1 已核 `doctor_resource_plan` stdout 与 `config_resource_policy` stderr 只打印 Host 原原因码；本片兑现父债已有“原因码解释”，不新增硬件策略。M、轻语义展示；controller 实施和自查，independent=false。

**闭环 / 冻结语义 / 写集**：Host compiler/registry 原 reason_codes → CLI 同一个私有 formatter → doctor 人类 stdout / draft 诊断与拒绝 stderr / wizard 确认前 stdout → 原集成测试。仅 `oclive-cli/src/doctor_resource_plan.rs`、`src/config_resource_policy.rs`、`src/resource_policy_wizard.rs`、原 `tests/{doctor_resource_plan,config_resource_policy}.rs`、中英 CLI guide、主台账 D-SCAFFOLD-RESOURCE-01 第五列、本计划与 DCL。已知常见容量/可用性/约束码附简短英文说明，保留原码；未知未来码原样保留且标注暂无内置说明。说明不参与解析、验证、计划选择、排序或发布。`--json` 仍只输出既有原 DTO；全部退出码、profile/capture 不变、无控制权或准入保证，无新公开 API/依赖/Host schema/CI。

**有限测试 / 节奏 / 停止**：先增强真实 CLI 集成红测：degraded 的控制器说明、blocked 的容量说明、draft 诊断共用说明；同一 fixture 的人类调用前后 canonical JSON 逐字段相等。未知码仅测私有展示，不建第二套 resolver。测试先运行后实现，测试过程不与前批 Cargo 全链并发争锁；实际 CLI doctor 及 policy/preview consumers、default feature 拒绝、两 feature Clippy/fmt与适用文档/分层/债结构/编码/diff。前批门禁期间只准备隔离源码；窄测后干净本地预备提交，不立即每片完整 CI。达到展示闭环即停，不追全部资源异常或重新定义原因码，不进入自动硬件建议、实时采样、执行或发行版产品 UI；需新产品决策先跳过并汇报。零真实 Host/设备/模型/TTS/用户库，未复用旧场景身份；后续相关批次里程碑另冻结完整及正式 CI。

### K-SUPPLY-12 · shell-quote 临时兼容例外（2026-10-07）

**基线 / 已知首因 / 授权**：干净主线与远端 `67baa952bb7e1ff0ba987609d6fe729ca327f24d`。前批完整本地链 native 0，但[正式 CI 37505631986](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37505631986) 的 npm-audit 因新公告 [GHSA-pqg4-j6r4-53mv](https://github.com/advisories/GHSA-pqg4-j6r4-53mv) 拒绝，最终取消剩余运行，正式验收不成立。`concurrently 9.2.4 → shell-quote 1.9.0` 两节点与前个已验基线相同，不能归因此前 parser/Cargo 补丁引入；官方安全版本为 1.11.0，上游当前 9.2.4 与 10.0.5 均仍精确依赖 1.9.0。维护者已明确确认只记录并实施这条临时兼容补丁，待上游安全更新后更新父依赖链、撤销例外；不再重复请求确认。

**写集 / 闭环 / 风险**：恰为根 `package.json` / `package-lock.json`、中英 `security/KNOWN_VULNERABILITIES.md`、台账 K-SUPPLY-12 第五列、本计划和 DCL（DCL-53）。仅增加 `overrides.concurrently.shell-quote = "1.11.0"`，保留原四条 override；官方 npm 11.21.0 临时生成锁，原 npm 10 消费，预期锁仅一个 shell-quote 节点的 version/resolved/integrity 变化。声明 → 锁 → 原 npm ci/peer/audit → concurrently 实际良性子进程及退出码 → 原 build/前端消费者 → 收口记录；不改产品源码、公共 API、CI 阈值、父主版本、TLS 或权限。安全触发重风险规划，工程收口 L；controller 实施和专项语义自查，independent=false。发现额外依赖刷新先保全并重规划，不能手改锁、force 或放宽 peer。

**有限验收 / 副作用 / 撤销**：前后 production/full JSON audit 分存，修后 high 门禁应 native 0；`npm ci --ignore-scripts --no-audit --no-fund`、`npm ls --all`、concurrently 良性成功/失败退出、shell-quote 内存检查四种换行分隔符拒绝（不在 shell 执行攻击字符串），真实 lint/typecheck/shared unit/build 和适用镜像/链接/登记/旧路径/债结构/编码/diff。达到修复及兼容判断即停止扩证，不穷尽可达攻击面或清除无关 KaTeX low。源码冻结后仅一次完整本地链，成功后 FF/push 一次及新 SHA 正式 CI；旧 failed/cancelled run 不 rerun。联网限官方 registry/advisory 和 GitHub，零真实模型/TTS/用户库/旧身份回放。原件保存到 `.cursor/plans/debt-shell-quote-patch-20261007-r0/`，旧工作树/账本不改。由依赖维护者在上游支持线采用安全 shell-quote 后更新 concurrently 及其依赖链，移除本条 override，核锁/peer/audit 与实际构建退出；不能仅因上游发布新版本就自动撤销。尚未取得目标 CI 前父债保持 Partial。

### D-SCAFFOLD-RESOURCE-01 · 原因说明合流（2026-10-07）

**已验基线 / 合流原因**：维护者授权持续收敛；前片 `1e81033d6c8efcff8bb9f502c642560e57335cf1` 完整本地 native 0（1491.7835099 秒）、[37513747676](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37513747676) attempt 1、17/17 success 含 ci-gate，main/tracking/远端干净一致，原 closeout 原件保留。新的隔离工作树把已局部验证的 `e1e12da6d76f59cc1f3989ab675a67db2b4d6e90` 原因展示接到该安全锁基线；不在原冻结分支 rebase 或覆写证据。只读核对到五份源/测试相同、原 Host 生成语义和权限不变即停，M 展示合流，L main 出口，controller 自查、independent=false。

**写集 / 整体不变量**：原十路径与中英 KNOWN_VULNERABILITIES 的前片终态回填，共十二路径；五份 CLI 源码/测试保留预备片原实现，canonical JSON、loader/validator/compiler、发布、退出码、公开 API、依赖、权限和 CI 不变。台账只改资源工具和 npm 父债第五列，其余行及前四列保持；DCL-52/53 保留历史、追加 [DCL-54](DEBT_CHANGELOG.md#dcl-20261007-54--资源原因说明合流与已验基线确认)。新预备片和 main 在仅两个 append-only 文档发生冲突时原始冲突文件及 native 1 保全，再合并双方新增段落，不把冲突误写成产品测试失败。

**有限验证 / 停止 / 节奏**：新 cwd 串行运行 CLI policy/doctor diagnostics-host 24 项、未知码 1、default 拒绝 3、default/all-features Clippy/fmt，结构核对与分层、中英镜像、默认/改文链接、登记、旧路径、债结构、编码、diff。源码与文档冻结后准备隔离 Tauri bundle，完整本地链通过才 FF/push，一次目标 SHA 正式 CI；不以 e1 的窄测或 1e 的 CI 代替本片终态。说明闭环验够即停，不增加硬件建议或实时 controller，不追全部资源组合。父债仍 Partial。回执在 `.cursor/plans/debt-resource-reasons-closeout-20261007-r0/`，忽略目录不随 Git 转让；旧全链与场景身份不重跑。

### K-RESILIENCE-01 · Remote Prompt 既有入口增量收束（2026-10-07）

**基线 / 授权 / 原因**：main/tracking/实际远端均为干净的 `a679937943b8c079c686f5714ec3aeaa35f08def`；资源原因说明片完整本地 native 0（1310.5125887 秒）及[37526132398](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37526132398) attempt 1、17/17 success 含 ci-gate，原回执保留，随本次实质修改回填台账。维护者选择“增量统一既有 Remote 入口，保持行为”，要求整理收束而非全盘定死。旧 Minimal long-plan 的 Closed 状态和历史 Stage 原样保留；本片按新授权走常规流水线，M 有限整理、L 最终 main 出口，controller 实施与自查，independent=false。父债仍 Partial，Full ResilienceLayer 未定义、未实现。

**闭环 / 两片 / 写集**：生产 PromptAssembler → 原 method/role/scene_id → 原 Remote HTTP/JSON-RPC/授权/超时 → 既有 blocking adapter → Remote/builtin/None → 直接生产消费测试。第一片将 `top_topic_hint` 的手写失败回退改用 `call_with_builtin_fallback`；第二片移除 Prompt 冗余开关引用，本地序列化/形状检查经 adapter 的模块内读取方法使用原闸门。只改 `remote_plugin/{adapter,prompt_http}.rs`、新增 `desktop-tauri/tests/remote_prompt_fallback_roundtrip.rs`、中英 REMOTE_PLUGIN_PROTOCOL、本计划/DCL、主台账两行第五列。不改公开 API/DTO、依赖、持久化、六槽或小 Kernel、调用者、权限、超时和 Host-side 无 retry 的现状。

**冻结行为 / 有限证据**：有效对象/raw string（含空字符串）返回原 hint；成功但无字符串的形状返回 None，即使开关打开也不触发 builtin。HTTP/RPC 失败按原开关回退；高风险授权拒绝不发请求、不回退。`build_prompt` 坏形状与本地序列化分支保留原错误、日志与闸门，decode 失败不由公共 helper 自动重解释。四项表征测试先在未改生产源码上通过，再在两片后分别以相同断言通过；确切请求数量约束无新增 retry。适用既有 Remote 测试、Clippy/fmt、分层/镜像/链接/登记/旧路径/编码/债结构/diff；新 worktree 的 bundle 与 frontendDist 构建前置失败独立保留，不当作产品断言失败。

**节奏 / 止线 / 副作用**：每片仅本地窄测与提交；相关两片及文档冻结成一批后，只跑一次完整本地链，成功才 FF/push 一次并验目标 SHA 正式 CI。本地充分即停止扩异常组合，不能借旧 SHA 的绿灯。Memory 有不同的 empty/error 规则，本批不改，只保留后续单片候选；不扩为全路径统一策略、retry/熔断/框架，也不重启签名/TLS/EventStream/Agent/硬件建议等未授权范围。测试只 loopback、临时合成角色/独立 grant store；零真实模型/TTS/用户库/旧 CP-INT 身份，Cargo 串行，子进程不继承 CARGO_TARGET_DIR。中英文收窄“失败必然回退”的过度描述，不反向要求代码承担更宽承诺。原始回执在 `.cursor/plans/debt-remote-prompt-consolidation-20261007-r0/`，ignored 证据不随 Git 自动携带。

### K-RESILIENCE-01 · Remote Memory 闸门的有限收束（2026-10-07）

**起点 / 问题 / 预算**：干净 `f4ee767bb92c6e7d0ccb6b6770ee41e1d463373a`；上一 Prompt 批完整本地 native 0（2348.8380943 秒）与正式 [37579810413](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/37579810413) attempt 1、17/17 success 已取得。Memory 已使用共同 HTTP 传输，但再次持有 adapter 中同一个运行时开关引用；本片复用已有模块内 getter，减少重复读取入口。M 实施与 controller 自查，independent=false；仅查主调用链与 empty/error、权限两个相关边界，足够施工即停。不扩所有超时、并发切换、取消和故障组合。

**闭环 / 保持语义**：生产 Memory trait → `rank_remote` → 原 `call_plugin_soft` 与 HighRisk grant → 原 `ordered_ids` 解码/补尾/limit → 原非空返回或 empty/error 闸门 → builtin/原错误。只替换开关读取所有者；保留 soft RPC、读开关时机、错误全文、日志、请求体、超时和单次请求，`build_context`/`search_memories` 与其他槽不变。成功 `ordered_ids: []` 在输入非空时仍补原记忆，不等同空结果；输入为空且不允许回退时仍报错。不能强行采用 Prompt 的结果策略。

**精确写集 / 顺序**：`kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/memory_http.rs`、既有 `distros/desktop-tauri/tests/remote_prompt_fallback_roundtrip.rs`（复用同一侧车增加 Memory 表征，保留旧四项 Prompt 断言）、中英 `REMOTE_PLUGIN_PROTOCOL.md`、主台账 K-RESILIENCE-01 第五列、本计划、DCL。先完成同一生产路径表征并在未改 Memory 源码通过，再改字段与读取、复跑相同断言与既有 Remote 回归。协议登记 Memory getter 与原 empty/error 规则；顺带修正英文 §6 尚残留的“unknown shapes 必回退”旧句，避免比代码承诺更宽。

**验收 / CI 节奏 / 停止**：生产表征覆盖远端顺序、空 ID 数组、HTTP 503/坏形状的 live 开关、真实空输入与授权拒绝零请求，准确核请求体/次数；定向两项 Clippy、fmt、分层与适用镜像/链接/登记/旧路径/编码/债结构/diff。开发片只窄测、本地提交；本次不为单一 getter 改动立即再推全量矩阵。后续形成有限 Remote 批冻结点再一次完整链与精确 SHA 正式 CI，当前只可 Locally verified，父债仍 Partial、Full OPEN。权限/公开 API/结果策略需变更则停在相关项；零真实模型/TTS/用户 DB/旧 run ID，仅独立 TempDir grant 与 loopback 侧车。上一批终态随这次实质改动回填，不新建纯绿灯提交。

### DUAL-CORE-FREEZE · 暂停 Beta 维护并保留解冻方向（2026-10-07）

**决定 / 基线 / 归属**：维护者明确选择“只冻结，指明方向和记录文档，继续其他债务”。从干净、已验的 `90b9cf4d3f9a9a17744323d1177f138c90c686ea` 建独立工作树；本片为 M 文档与台账整理，由 controller 实施和自查，independent=false。冻结暂停 v3 双核 Beta / expert_routing 的继续维护与默认开启，不删除已有 opt-in 实现、不变更运行时语义；K-DUAL-ROLLBACK-02 保留未解决事实，状态为 Frozen / Deferred，不是 Done。

**精确写集 / 接手入口**：复用 [DUAL-CORE-FREEZE](long-plans/DUAL-CORE-FREEZE.md)，写明默认 Stable 路径、已知 NULL 补偿缺口、解冻前提及有限首片；更新原 [双核交接](../DUAL_CORE_CURSOR_HANDOFF.md)、中英 developer guide 的短提示、主台账相关状态、DEBT_CHANGELOG 的当前依赖行并追加本次事件。共七份既有 Markdown；不新建状态台账，不改 RFC 的已实现历史、队列 skip 行、源码、配置、feature、依赖或测试。随本次实质冻结记录回填 Remote Memory 已取得的完整本地与目标 SHA 正式 CI 终态，历史 DCL-56 / 原计划保持。

**验收 / 限度 / 后续**：默认及改文链接、docs-only 旧路径、文档登记、中英镜像、债结构、编码和 diff 检查；额外核精确七路径、台账既有五列的前四列、DCL 历史正文、源码/config 零差量及 skip 保持。本片不跑双核场景、不补证明、不分配 run ID、不为冻结文档重复全量 CI；本地干净 checkpoint 随后续实质批次合流。达到冻结范围与接手规则清楚即停，继续有施工前提的其他债务；Beta 解冻或扩大实验能力须由维护者另行决定。

### K-SUPPLY-10 · CLI Rust Action 步骤的单点维护（2026-10-07）

**基线 / 原因 / 归属**：main / tracking / 实际远端干净一致 `90b9cf4d3f9a9a17744323d1177f138c90c686ea`，既有完整本地与目标 SHA CI 已验；干净本地冻结 `3865f5c859ae21d61af1124f478a706c1e67096a` 在其上仅加七份双核维护文档，未 main / push。新工作树从该冻结提交接续，双核保持 Frozen / Deferred。CLI 的同一 Rust Action 步骤在 audit/deny、build、可选 bench/OOCP 重复五处，现有同步合同会拒绝遗漏，却不能减少手动修改点。本片为 M 私有模板整理，controller 实施/自查，independent=false；只查这一生产者与生成消费链，D1 已足够施工，不扩大 CI 架构设计。

**闭环 / 精确写集**：`kernel/crates/oclive-cli/src/ci_cmd.rs` 内一个私有完整 YAML step → 既有 `render_ci_yaml` → library / kernel 两类生成 workflow → `ci init` / `ci check` 与原同步正负合同。复用既有渲染和测试，只把相同步骤插入五个原位置；原 SHA、注释、stable、缩进、换行、jobs、命令、权限与失败策略的生成字节必须完全相同。文档限现有 Actions Wave、中英 SUPPLY_CHAIN §5 第 6 项、主台账 K-SUPPLY-10 第五列、本计划及 DEBT_CHANGELOG，共七个修改路径；不新建第二份 pin 配置、升级 Action、改实际 `.github`、公开 API、Cargo/lock、信任根或六槽。其它消费者无接口变化，明确无需改。

**验证 / 止点 / CI 节奏**：整理前真实 CLI 生成两类隔离项目并保存字节，原 `ci_cmd::tests` 通过；整理后复跑相同测试、实际 init/check，生成字节逐项相等，并把其中一份隔离 workflow 的 pin 改坏，真实 `ci check` 必须非零。CLI crate 全测串行、default/all-features Clippy、fmt、适用分层与文档/债结构/编码/diff；不为这个保持行为的开发片单独推远端矩阵。只在相关批次结束时一次完整本地链和精确 SHA 正式 CI；未取得前仅 Locally verified，K-SUPPLY-10 仍 Partial。若生成行为、安全语义或依赖需改则停在具体问题，不能靠松断言收口；零真实模型/音频/用户 DB/历史业务身份，只在本片忽略目录写合成项目，Cargo 串行且不继承 CARGO_TARGET_DIR。

### D-DEBT-LEDGER-01 · 供应链指南的两处过期现状对账（2026-10-07）

**基线 / 已知反例 / 限度**：从干净本地 `5729fe76a5cebe3fc649ee792d8903b3f8615f9a` 接续；发现中英 SUPPLY_CHAIN 的现状表仍将 K-SUPPLY-12 写成八月全部 audit 为 0 / Done，并将 K-PLUGIN-SEC-01 写成 HTML 仍共享旧 origin，与当前唯一台账的 Partial / 临时 override 待撤销、opaque sandbox + broker Stage 0–3 限定验证矛盾。D1 只核对应权威行、既有 Wave / 安全记录和实际实现入口，足够对账即停止；不扫描全表，不补跑实机、audit 或旧业务场景，不重新裁决签名架构。

**精确写集 / 预期**：中英 SUPPLY_CHAIN §4 两行与该节当前状态引用提示；主台账 D-DEBT-LEDGER-01 仅第五列追加本次局部进展，以及 §2 原“下一工程动作”短段区分八月历史验收与当前未决；本计划与 DEBT_CHANGELOG 追加事件，共五份既有 Markdown。现状行短述当前边界并链接原权威状态 / Wave，八月 SHA 与 CI 已在本文件历史表保留，不删除历史，不复制新扫描数或制造新状态表。其余债状态、旧证据、源码/测试/依赖不动，controller 自查 independent=false。

**验收 / 里程碑**：文档链接、旧路径、登记、镜像、债结构、编码、diff；精确五路径与历史保留核对。保存本地文档 checkpoint 后，双核冻结 + CLI 步骤整理 + 本片构成一个有限维护里程碑，执行一次 `check:ci-local`、冻结目标 SHA、一次合流推送并观察正式 CI；不在三个开发片之间重复全量，不追加只回写绿灯的提交。K-SUPPLY-10 / K-SUPPLY-12 / K-PLUGIN-SEC-01 / D-DEBT-LEDGER-01 不由本片或全量绿灯改为 Done；若前置或语义矛盾需要扩大写集，先停在具体原因。
