---
name: oclive-debt-marathon
description: >-
  OCLive technical-debt marathon: long-form phased plan books, staged sub-agent
  execution under oclive-dev-pipeline gates, AI change boundaries, wave logs.
  Use when user says 债偿还马拉松, 技术债马拉松, 长流程计划书, debt marathon,
  or multi-stage tech-debt clearance.
---

# OCLive 技术债偿还马拉松

**不替代** [`dev-pipeline`](~/.cursor/skills/dev-pipeline/SKILL.md) 与 [`oclive-dev-pipeline`](../oclive-dev-pipeline/SKILL.md)。  
本 Skill = 长计划书 + 分阶段子 Agent；**必须同时加载二者 + [`AI_AND_PIPELINE_GATES`](../../../handoff/debt-marathon/AI_AND_PIPELINE_GATES.md)**。

## 存放点

| 层 | 路径 |
|----|------|
| 门禁（AI+流水线） | `handoff/debt-marathon/AI_AND_PIPELINE_GATES.md` |
| 总索引 | `handoff/debt-marathon/MARATHON_QUEUE.md` |
| 覆盖审计 | `handoff/debt-marathon/COVERAGE.md` |
| 一书一债 | `handoff/debt-marathon/long-plans/<ID>.md` |
| 波次 | `handoff/debt-marathon/waves/` |

## 触发语

「债偿还马拉松」「技术债马拉松」「长流程计划书」「按马拉松还债」「debt marathon」

## 启动（强制顺序）

1. `AI_AND_PIPELINE_GATES.md`  
2. `oclive-dev-pipeline` + `discipline-checklist` + 通用 `dev-pipeline`  
3. `MARATHON_QUEUE.md` + `COVERAGE.md`  
4. `AGENTS.md` · `AI_CHANGE_BOUNDARIES.md` · `AI_VERIFICATION_PROTOCOL.md`  
5. 打开目标 `long-plans/<ID>.md` **仅当前 Stage**

缺任一门禁阅读记录 → 不得改代码。

## 隔夜总控

见 `MARATHON_QUEUE.md` 粘贴块。摘要：

- 只跑 `auto` · 一债一 Stage · 默认 PR 不合 main
- **Cursor IDE 父子模型**：父 Agent 单写状态；每 Stage 调用自定义子 Agent `oclive-debt-stage`；子 Agent 不选下一债
- 启动前在 Cursor 下拉选择 **worktree**，工作树必须 clean；禁止用 stash 隔离 dirty
- `npm run check:debt-marathon` 后执行 `node scripts/cursor-marathon.mjs start --max-turns 30`，由 stop hook 续轮
- 每轮结束前必须写 Wave，并执行 `cursor-marathon checkpoint`；终态执行 `finish`，无进展两轮自动熔断
- Wave 必须勾选 GATES §6
- `human`/`skip` 禁止伪造成绩
- 上下文满则停并写续跑坐标

### Cursor 运行边界

- 自动续轮仅保证 **Cursor IDE**；Cursor Background/Cloud Agent 当前不依赖 lifecycle hook，按 Wave 手动续跑。
- 父 Agent 每次最多派发一个会写同一 worktree 的 Implementer；只有 Cursor 原生 worktree 隔离后才可并行。
- 每次 `claim` 都重新要求 clean worktree；checkpoint 以 claim `baseSha` 对比 **已提交 + 未提交 + 未跟踪**文件，commit 不能绕过 Stage 文件范围。
- `.cursor/oclive-marathon-session.json` 是本机运行态，不进 Git；长久恢复真值仍是 Wave + Git SHA。
- stop hook 只在 `cursor-marathon start` 激活后续轮，并在**首次** `status=completed` 时绑定 `conversation_id`（不按墙钟 5 分钟超时）；普通聊天不受影响。
- hook 失败 fail-open（不杀 session）；`hooks.json` `loop_limit` 须 ≥ `max-turns`；自检 `npm run test:cursor-marathon-hook`。

### Cursor 父 Agent 每轮

1. 读本机 session + 最近 Wave，reconcile QUEUE / long-plan / TECHNICAL_DEBT / Git / PR-CI。
2. 若有冲突，只修状态或 `blocked:needs-reconcile`，禁止施工。
3. 选择最小 `auto + Ready` Stage 并先 claim。普通实现 Stage 调用 `oclive-debt-stage`；若 Stage 文件范围包含 Wave / QUEUE / TECHNICAL_DEBT，则由父 controller 自己执行，禁止派给子 Agent。dispatch 带 claim、Stage、文件范围、base SHA 与 capability snapshot。

```powershell
node scripts/cursor-marathon.mjs claim --debt <ID> --stage <N> --agent oclive-debt-stage --capabilities local-write,test
```

默认 capability 只有 `local-write,test`；`commit,push,open-pr,merge,sibling-repo,network,secrets` 必须按用户授权显式加入，并用 `--authorization <用户消息/决策引用>` 留痕。
4. 校验子 Agent JSON 的 claim/debt/stage/base SHA 与实际 diff；父 Agent独自更新 Wave/QUEUE，并把计划契约 `currentStage` 推进到下一 Stage。
5. 记录 checkpoint：

```powershell
node scripts/cursor-marathon.mjs checkpoint --claim <CLAIM_ID> --debt <ID> --stage <N> --outcome progress --wave <WAVE_PATH> --last-command "<COMMAND>" --next "<EXACT_NEXT_COMMAND>"
```

最终一轮须用 `--outcome done` 写 terminal checkpoint；`finish --outcome done` 会拒绝 `progress` checkpoint，并确认不存在 queue=`pending|ready|implemented|locally-verified` 的 Ready auto Stage。`pr-open` 是等待外部审查/合入的暂停态，不得重复 claim。

6. 完成/阻断/失败时：

```powershell
node scripts/cursor-marathon.mjs finish --outcome done --reason "<REASON>"
```

子 Agent 运行超过 20 分钟时，父 Agent 应 heartbeat：

```powershell
node scripts/cursor-marathon.mjs heartbeat --claim <CLAIM_ID>
```

父/子 Agent 崩溃或 lease 过期后，先核对 worktree、base/head 与 diff，再显式释放或阻断；禁止直接重派：

```powershell
node scripts/cursor-marathon.mjs recover --claim <CLAIM_ID> --action release --reason "<RECONCILE_RESULT>"
```

## 写书纪律

- 一文一债；Minimal/Full 分册  
- 每书必须链 GATES；模板见 `LONG_PLAN_TEMPLATE.md`  
- G11：书只在 `debt-marathon/`  
- 无书不开工；覆盖见 `COVERAGE.md`

## 子 Agent Implementer 块（最低集）

```
必读 GATES → oclive-dev-pipeline → 本书当前 Stage → BOUNDARIES
遵守：reply · 六槽命名 · 禁生产 unwrap · Cargo.lock audit · 错误码链
禁：扩 Full · 顶层新 md · 无 CI Done · 乱动 process_message · 混 dirty
```

## 验收收工

台账 Done ⟺ Verification（SHA+CI）+ Wave + QUEUE 进度 `done`；与 GATES / 七阶段一致。
