# 第二轮马拉松计划：解除阻断后再偿还

**SSOT 范围**：本文记录第二轮技术债马拉松的阶段安排及启动前复核；债务状态以 [`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md) 为准，自动队列以 [`MARATHON_QUEUE.md`](./MARATHON_QUEUE.md) 为准。
**最后更新**：2026-09-28（启动前基线快照及后续文档治理）。

> 入口门禁：[`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md)。本计划先处理外部阻断，不擅自把 human/skip 项改成 auto。

> **2026-09-28 基线复核快照**：Wave 1 提及的 [#124](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/124)、[#125](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/125)、[#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 均已合并；下方 Wave 1 是历史计划，不是当前待审 PR 清单。复核时队列仍有 `pr-open` 项；后续状态与来源处理见本页“后续文档治理与接手准备”，不将历史快照当开工指令。

## 目标

让下一轮只在证据、权限和运行条件齐备后启动自动 Stage；优先收敛已经形成 PR 栈的事项，再开放新的本地债务。

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

## 阶段

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
