# 技术债偿还马拉松（debt-marathon）

**状态台账 SSOT**：[`TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md)（OPEN / Done / Verification）  
**流程 SSOT**：通用 [`dev-pipeline`](../../.cursor/skills/oclive-dev-pipeline/SKILL.md) 链 · 项目定制 + 马拉松 Skill：[`oclive-debt-marathon`](../../.cursor/skills/oclive-debt-marathon/SKILL.md)

> `.cursor/` 常被 gitignore；**长流程计划书正文以本目录为准（进 git）**，方便 Cloud / 其他机器 / 人类查阅。Skill 只放规程与本地镜像说明。

---

## 目录

| 路径 | 用途 |
|------|------|
| [`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md) | **AI 限制 + OCLive 七阶段硬门禁（强制先读）** |
| [`MARATHON_QUEUE.md`](./MARATHON_QUEUE.md) | **子 Agent 总索引**（seq · runner · 进度） |
| [`COVERAGE.md`](./COVERAGE.md) | 对照 TECHNICAL_DEBT 的覆盖审计 |
| [`LONG_PLAN_TEMPLATE.md`](./LONG_PLAN_TEMPLATE.md) | 长流程计划书模板 |
| [`WAVE_LOG_TEMPLATE.md`](./WAVE_LOG_TEMPLATE.md) | 波次工作记录模板 |
| [`long-plans/`](./long-plans/) | 一书一债 |
| [`waves/`](./waves/) | 波次日志 |

---

## 怎么用（短）

1. Cursor IDE 选择 **worktree** 启动 Agent；共享 dirty 工作树不得运行马拉松。
2. 运行 `npm run check:debt-marathon`，再运行 `node scripts/cursor-marathon.mjs start --max-turns 30`。
3. 父 Agent 只跑 `runner=auto`；普通实现 Stage 调用 `oclive-debt-stage`，Wave / QUEUE / TECHNICAL_DEBT 证据 Stage 由父 Agent执行；每轮仍是 **一本债 × 一个 Stage**。
4. 父 Agent 校验子 Agent 结构化结果，推进计划契约 `currentStage`，写 `waves/` 和 checkpoint；stop hook 自动进入下一轮。
5. **默认不 push / 不开合 PR / 不合 main**；能力必须在 dispatch 中显式授予。
6. 人工 / skip / blocked 项禁止假装做完；证据齐再改 TECHNICAL_DEBT Done。

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

合法运行态：`running → progress → done|blocked|failed`。两轮没有新 checkpoint、达到 `max-turns`、Cursor 返回 aborted/error，都会自动停机。Stage 完成、Plan Closed、Done-eligible、父技术债 Done 是四个不同层级。

### Checkpoint

```powershell
node scripts/cursor-marathon.mjs claim --debt <ID> --stage <N> --agent oclive-debt-stage --capabilities local-write,test
node scripts/cursor-marathon.mjs checkpoint --claim <CLAIM_ID> --debt <ID> --stage <N> --outcome progress --wave <WAVE_PATH> --last-command "<COMMAND>" --next "<EXACT_NEXT_COMMAND>"
```

默认 capability 只有 `local-write,test`。`commit,push,open-pr,merge,sibling-repo,network,secrets` 必须来自用户对本轮的明确授权，并用 `--authorization` 记录授权引用；缺能力时记录稳定 blocker code，不从自然语言猜权限。

claim lease 默认 30 分钟。长 Stage 用 `heartbeat --claim <CLAIM_ID>` 续租；父/子 Agent 崩溃后必须先检查原 worktree，再用 `recover --claim <CLAIM_ID> --action release|block --reason <...>`，禁止直接重派。

### 运行环境边界

| Cursor 面 | 自动续轮 | 隔离要求 |
|-----------|----------|----------|
| IDE Agent | stop hook 支持；首次使用须确认自动 follow-up smoke，失败则按 Wave 手动续跑并 `finish --outcome failed` | 启动时选择 Cursor worktree |
| Background / Cloud Agent | 不依赖 lifecycle hook；按 Wave 手动续跑 | 远程独立分支；不得假设本机 `.cursor` session 存在 |
| Cursor CLI | 只使用脚本校验/状态；不承诺 stop hook parity | 独立 clean worktree |
