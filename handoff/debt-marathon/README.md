# 技术债偿还马拉松（debt-marathon）

**状态台账 SSOT**：[`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md)（OPEN / Done / Verification）  
**流程 SSOT**：[仓库通用 dev-pipeline](../workflows/dev-pipeline/SKILL.md) → [OCLive 定制](../../.cursor/skills/oclive-dev-pipeline/SKILL.md) → [马拉松 Skill](../../.cursor/skills/oclive-debt-marathon/SKILL.md)
**变动与接手**：[DEBT_CHANGELOG](DEBT_CHANGELOG.md)；当前状态不在这里双写。

> **长流程计划书正文以本目录为准（进 git）**；通用 Skill 正文位于 `handoff/workflows/dev-pipeline/`，项目 Skill 也已跟踪，Cloud/其他机器无需依赖本机副本。

---

## 目录

| 路径 | 用途 |
|------|------|
| [`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md) | **AI 限制 + OCLive 七阶段硬门禁（强制先读）** |
| [`MARATHON_QUEUE.md`](./MARATHON_QUEUE.md) | **子 Agent 总索引**（seq · runner · 进度） |
| [`COVERAGE.md`](./COVERAGE.md) | 对照 TECHNICAL_DEBT 的覆盖审计 |
| [`DEBT_CHANGELOG.md`](./DEBT_CHANGELOG.md) | 变动事件、证据和 AI 接手续跑规则（当前状态仍在主台账） |
| [`LONG_PLAN_TEMPLATE.md`](./LONG_PLAN_TEMPLATE.md) | 长流程计划书模板 |
| [`WAVE_LOG_TEMPLATE.md`](./WAVE_LOG_TEMPLATE.md) | 波次工作记录模板 |
| [`long-plans/`](./long-plans/) | 一书一债 |
| [`waves/`](./waves/) | 波次日志 |
| [`ROUND-01-CLOSEOUT.md`](./ROUND-01-CLOSEOUT.md) | 第一轮马拉松收尾证据 |
| [`ROUND-02-PLAN.md`](./ROUND-02-PLAN.md) | 第二轮解除阻断与再开放计划 |

---

## 怎么用（短）

1. Cursor IDE 选择 **worktree** 启动 Agent；共享 dirty 工作树不得运行马拉松。
2. 先按 DEBT_CHANGELOG 接手顺序对账，运行 `npm run check:debt-marathon`；环境、授权和目标 Stage 确认后才运行 `node scripts/cursor-marathon.mjs start --max-turns 30`。结构 PASS 不代替开工前提。
3. 父 Agent 只跑 `runner=auto`；普通实现 Stage 调用 `oclive-debt-stage`，Wave / QUEUE / TECHNICAL_DEBT 证据 Stage 由父 Agent执行；每轮仍是 **一本债 × 一个 Stage**。
4. 父 Agent 校验子 Agent 结构化结果，推进计划契约 `currentStage`，同步受影响的状态并写 `waves/`、变动事件和 checkpoint；stop hook 自动进入下一轮。
5. **默认不 push / 不开合 PR / 不合 main**；能力必须在 dispatch 中显式授予。
6. 人工 / skip / blocked 项禁止假装做完；证据齐再改 TECHNICAL_DEBT Done。

## 有界清理与恢复开发出口

技术债清理以解除已明确开发路径的真实阻碍为目标；状态、优先级与冻结条件仍归[主台账](../TECHNICAL_DEBT_INVENTORY.md)，证据与变动归[DCL](DEBT_CHANGELOG.md)。以下接续维护者提供的辅助清理标准，引用现有规范，不建立第二份当前状态或完成制度。

1. **先定必修范围**：候选须说明具体受阻动作、触发例子、为何现在处理、有限完成条件和非目标。区分直接阻碍、维护负担、验收缺口、扩展与外部前提；分类不改台账优先级，Unknown/未覆盖不自动成为阻塞。未确定下一产品功能时，可核构建/测试/调试、现行契约和数据/安全的基础阻碍，不能自行声称某产品全面恢复。
2. **证据按范围复用**：目标结论、相关代码/依赖/配置、环境及原件边界相符即沿用；换助手或续会话不自动失效。原件缺失先记缺口，旧 SHA 不冒充新 SHA 正式通过。调查预算与充分即停止沿[核实协议 §2.9–2.10](../AI_VERIFICATION_PROTOCOL.md)，原因、改法和影响已明确即进入有限实施。
3. **按变更选验证**：局部切片使用适用定向检查；必要正式结案批次冻结最终 SHA 后取得相应完整与远端硬门禁。纯文档不因此重复 Rust/前端全量。真实设备或外部条件缺口只阻断对应验收，替身/结构/通用 CI 不能代替；[适用门禁及 Done 出口](AI_AND_PIPELINE_GATES.md)保持。
4. **达到恢复条件即结束专项**：已明确目标路径的本轮必修阻碍均达到相应完成条件，必要出口证据齐全，无未解释的安全/数据风险或硬门禁失败，相关残余项有准确状态与不阻挡路径的依据，即恢复该路径开发。仍阻挡该路径的 Blocked 必须解决，不能改成 Deferred 隐藏问题；不影响它的 OPEN/Partial/Deferred 可保留，不以台账清零或无 runnable auto 当全部债务完成。
5. **后续按触发重开**：新变更、真实故障、现行承诺缺口或已定复评条件出现时，再处理相关历史项；没有施工点或补证收益下降就停止，不不断寻找下一项优化。新架构/权限/执行语义与解冻仍须确认；不自动删除缓存、用户数据或历史证据。原 run 终态和绿灯入账按[现行 CI 节奏](../AI_VERIFICATION_PROTOCOL.md)处理，不为回写结果再造完整验收循环。


## 通用流水线安装副本

仓库是正式来源，本机 `~/.cursor/skills/dev-pipeline/` 只供技能加载。维护仓库三文件后再同步副本；不要仅改本机。以下在仓库根运行，遇到已有不同字节先对账，不能静默覆盖机器私有规则；无需安装副本即可读取仓库规则。

```powershell
$pipelineSource = Join-Path (Get-Location).Path 'handoff/workflows/dev-pipeline'
$pipelineMirror = Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cursor/skills/dev-pipeline'
$pipelineFiles = @('SKILL.md', 'task-sizing.md', 'plan-template.md')
foreach ($name in $pipelineFiles) {
    $sourceFile = Join-Path $pipelineSource $name
    $mirrorFile = Join-Path $pipelineMirror $name
    if (-not (Test-Path -LiteralPath $sourceFile -PathType Leaf)) { throw "Missing source: $sourceFile" }
    if ((Test-Path -LiteralPath $mirrorFile) -and
        ((Get-FileHash -LiteralPath $sourceFile -Algorithm SHA256).Hash -ne
         (Get-FileHash -LiteralPath $mirrorFile -Algorithm SHA256).Hash)) {
        throw "Reconcile local differences before syncing: $mirrorFile"
    }
}
[void][System.IO.Directory]::CreateDirectory($pipelineMirror)
foreach ($name in $pipelineFiles) {
    $sourceFile = Join-Path $pipelineSource $name
    $mirrorFile = Join-Path $pipelineMirror $name
    Copy-Item -LiteralPath $sourceFile -Destination $mirrorFile
    if ((Get-Item -LiteralPath $sourceFile).Length -ne (Get-Item -LiteralPath $mirrorFile).Length -or
        (Get-FileHash -LiteralPath $sourceFile -Algorithm SHA256).Hash -ne
        (Get-FileHash -LiteralPath $mirrorFile -Algorithm SHA256).Hash) {
        throw "Mirror mismatch: $mirrorFile"
    }
}
```

## Cursor IDE 长跑协议

```text
父 Agent（单写者 + stop hook）
  → reconcile QUEUE / plan / inventory / Git / PR-CI
  → 派发一个 oclive-debt-stage 子 Agent
  → 校验 debt_id / stage_id / base_sha / diff / checks
  → 写 Wave + checkpoint
  → stop hook followup_message 进入下一轮
```

本机运行态是 `.cursor/oclive-marathon-session.json`，由 `scripts/cursor-marathon.mjs` 原子写入并绑定 Cursor `conversation_id`。它只负责续轮与熔断，不是技术债真值；跨机器恢复仍以 Git SHA、long-plan、Wave 和 TECHNICAL_DEBT 为准。

**Stop hook 续轮要点（2026-07-16 修复）**：首轮父 Agent 常 >5 分钟才第一次 `stop`；因此 **不按墙钟超时解绑**，在首次 `status=completed` 时绑定 `conversation_id`。`.cursor/hooks.json` 设 `loop_limit: 50`（Cursor 默认仅 5）。hook 脚本失败时 **fail-open**（不 `finish` 杀掉 session）；诊断写入 `.cursor/oclive-marathon-hook.log`。自检：`npm run test:cursor-marathon-hook`。

合法运行态：`running → progress → done|blocked|failed`。两轮没有新 checkpoint、达到 `max-turns`、Cursor 返回 aborted/error，都会自动停机。Stage 完成、Plan Closed、Done-eligible、父技术债 Done 是四个不同层级。

每次 claim 前控制器重新检查 clean worktree；checkpoint 从 claim `baseSha` 核对已提交、未提交和未跟踪文件，因此先 commit 不能绕过 Stage scope。最终收口必须先写 `--outcome done` terminal checkpoint；控制器还会拒绝仍有 runnable auto 的 `finish done`。`pr-open` 表示等待审查/合入，不是可重复执行态。

### Checkpoint

```powershell
node scripts/cursor-marathon.mjs claim --debt <ID> --stage <N> --agent oclive-debt-stage --capabilities local-write,test
node scripts/cursor-marathon.mjs checkpoint --claim <CLAIM_ID> --debt <ID> --stage <N> --outcome progress --wave <WAVE_PATH> --last-command "<COMMAND>" --next "<EXACT_NEXT_COMMAND>"
```

最后一个 Stage 将上述 `--outcome progress` 改为 `--outcome done`，再执行 `finish --outcome done`。

默认 capability 只有 `local-write,test`。`commit,push,open-pr,merge,sibling-repo,network,secrets` 必须来自用户对本轮的明确授权，并用 `--authorization` 记录授权引用；缺能力时记录稳定 blocker code，不从自然语言猜权限。

claim lease 默认 30 分钟。长 Stage 用 `heartbeat --claim <CLAIM_ID>` 续租；父/子 Agent 崩溃后必须先检查原 worktree，再用 `recover --claim <CLAIM_ID> --action release|block --reason <...>`，禁止直接重派。

### 运行环境边界

| Cursor 面 | 自动续轮 | 隔离要求 |
|-----------|----------|----------|
| IDE Agent | stop hook 支持；首次使用须确认自动 follow-up smoke，失败则按 Wave 手动续跑并 `finish --outcome failed` | 启动时选择 Cursor worktree |
| Background / Cloud Agent | 不依赖 lifecycle hook；按 Wave 手动续跑 | 远程独立分支；不得假设本机 `.cursor` session 存在 |
| Cursor CLI | 只使用脚本校验/状态；不承诺 stop hook parity | 独立 clean worktree |
