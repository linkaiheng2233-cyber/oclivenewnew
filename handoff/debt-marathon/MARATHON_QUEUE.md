# 马拉松总索引（子 Agent 入口）

**用途：** 隔夜 / 长跑子 Agent **只认本索引**。  
**强制门禁：** 任何 Stage 前必读 [`AI_AND_PIPELINE_GATES.md`](./AI_AND_PIPELINE_GATES.md)（AI 限制 + OCLive 七阶段）。  
**覆盖审计：** [`COVERAGE.md`](./COVERAGE.md)  
**状态台账 SSOT：** [`../TECHNICAL_DEBT_INVENTORY.md`](../TECHNICAL_DEBT_INVENTORY.md)  
**规程 Skill：** `.cursor/skills/oclive-debt-marathon/SKILL.md` + `oclive-dev-pipeline`

---

## 硬规则

1. **先 GATES，后计划书，再改代码。**  
2. **一次会话 = 一本债 × 一个 Stage。**  
3. 只跑 `runner=auto`；`human`/`skip` 禁止假装 Done。  
4. 无 Ready 书 → 停。  
5. 默认 **PR 不开合 main**；Done 要 CI 证据。  
6. 严格七阶段与 G1–G16；`todo completed` ≠ Done。  
7. 上下文将满 → 停并写续跑坐标。
8. 父 Agent 是 QUEUE / Wave / session 单写者；子 Agent 禁止自行选下一债或改队列。
9. Cursor IDE 必须使用 clean worktree；禁止用 `git stash`、切分支、reset/clean 隔离 dirty。
10. `npm run check:debt-marathon` FAIL 或 queue/plan/inventory/PR-CI 冲突 → `blocked:needs-reconcile`，不得施工。

---

## 执行队列（seq 升序）

| seq | 债 ID | runner | 计划书 | 进度 |
|-----|-------|--------|--------|------|
| 10 | FOLLOWUP-VOICE-04-PR123 | auto | [long-plans/FOLLOWUP-VOICE-04-PR123.md](./long-plans/FOLLOWUP-VOICE-04-PR123.md) | done |
| 20 | T-DOC-02 | auto | [long-plans/T-DOC-02.md](./long-plans/T-DOC-02.md) | pr-open |
| 30 | D-ROLEVER-01 | auto | [long-plans/D-ROLEVER-01.md](./long-plans/D-ROLEVER-01.md) | pr-open |
| 40 | K-RESILIENCE-01 | auto | [long-plans/K-RESILIENCE-01.md](./long-plans/K-RESILIENCE-01.md) | pr-open |
| 50 | K-SUPPLY-05-Full | auto | [long-plans/K-SUPPLY-05-Full.md](./long-plans/K-SUPPLY-05-Full.md) | pr-open |
| 60 | K-CROSS-01 | auto | [long-plans/K-CROSS-01.md](./long-plans/K-CROSS-01.md) | pr-open |
| 70 | K-PERF-10 | skip | [long-plans/K-PERF-10.md](./long-plans/K-PERF-10.md) | skip |
| 80 | K-SUPPLY-04 | skip | [long-plans/K-SUPPLY-04.md](./long-plans/K-SUPPLY-04.md) | skip |
| 90 | V-VSCODE-PERF-05 | human | [long-plans/V-VSCODE-PERF-05.md](./long-plans/V-VSCODE-PERF-05.md) | human |
| 100 | PE-TURN-01 | human | [long-plans/PE-TURN-01.md](./long-plans/PE-TURN-01.md) | human |
| 110 | PE-UID-01 | human | [long-plans/PE-UID-01.md](./long-plans/PE-UID-01.md) | human |
| 120 | K-DIST-01 | auto | [long-plans/K-DIST-01.md](./long-plans/K-DIST-01.md) | pr-open |
| 130 | V-MARKET-01 | auto | [long-plans/V-MARKET-01.md](./long-plans/V-MARKET-01.md) | pr-open |
| 140 | K-VOICE-02 | skip | [long-plans/K-VOICE-02.md](./long-plans/K-VOICE-02.md) | skip |
| 150 | K-VOICE-03 | skip | [long-plans/K-VOICE-03.md](./long-plans/K-VOICE-03.md) | skip |
| 160 | K-VOICE-05 | skip | [long-plans/K-VOICE-05.md](./long-plans/K-VOICE-05.md) | skip |
| 170 | K-VOICE-06 | auto | [long-plans/K-VOICE-06.md](./long-plans/K-VOICE-06.md) | s1-done |
| 180 | K-VOICE-07 | auto | [long-plans/K-VOICE-07.md](./long-plans/K-VOICE-07.md) | blocked:needs-directive-v2-rfc-anchor |
| 190–330 | （其余 skip/human 同前版） | | 见下表续 |

### 续 · skip / 冻结 / §4

| seq | 债 ID | runner | 计划书 |
|-----|-------|--------|--------|
| 190 | K-VOICE-01 | skip | [K-VOICE-01.md](./long-plans/K-VOICE-01.md) |
| 200 | K-VOICE-08 | skip | [K-VOICE-08.md](./long-plans/K-VOICE-08.md) |
| 210 | D-SLOT-01 | skip | [D-SLOT-01.md](./long-plans/D-SLOT-01.md) |
| 220 | D-TRAIT-01 | skip | [D-TRAIT-01.md](./long-plans/D-TRAIT-01.md) |
| 230 | D-PORT-03 | skip | [D-PORT-03.md](./long-plans/D-PORT-03.md) |
| 240 | D-POLICY-01 | skip | [D-POLICY-01.md](./long-plans/D-POLICY-01.md) |
| 250 | D-ORPHAN-02 | skip | [D-ORPHAN-02.md](./long-plans/D-ORPHAN-02.md) |
| 260 | F4-V2-remote | skip | [F4-V2-remote.md](./long-plans/F4-V2-remote.md) |
| 270 | K-SUPPLY-06 | skip | [K-SUPPLY-06.md](./long-plans/K-SUPPLY-06.md) |
| 280 | K-SUPPLY-07 | skip | [K-SUPPLY-07.md](./long-plans/K-SUPPLY-07.md) |
| 290 | K-SUPPLY-08 | skip | [K-SUPPLY-08.md](./long-plans/K-SUPPLY-08.md) |
| 300 | MEGA-SD-01 | skip | [MEGA-SD-01.md](./long-plans/MEGA-SD-01.md) |
| 310 | MEGA-TS-01 | skip | [MEGA-TS-01.md](./long-plans/MEGA-TS-01.md) |
| 320 | MODE3 | skip | [MODE3.md](./long-plans/MODE3.md) |
| 330 | P0-STRANGER-EXT | human | [P0-STRANGER-EXT.md](./long-plans/P0-STRANGER-EXT.md) |
| 340 | K-PERF-15 | skip | [K-PERF-15.md](./long-plans/K-PERF-15.md) |
| 350 | V-FUSED-01 | skip | [V-FUSED-01.md](./long-plans/V-FUSED-01.md) |
| 360 | V-LORA-WORKSHOP-01 | skip | [V-LORA-WORKSHOP-01.md](./long-plans/V-LORA-WORKSHOP-01.md) |
| 370 | D-OPUS-05-P2 | skip | [D-OPUS-05-P2.md](./long-plans/D-OPUS-05-P2.md) |
| 380 | K-UID-POST-01 | skip | [K-UID-POST-01.md](./long-plans/K-UID-POST-01.md) |
| 390 | ROADMAP-MODAL-EDGE | skip | [ROADMAP-MODAL-EDGE.md](./long-plans/ROADMAP-MODAL-EDGE.md) |
| 400 | DUAL-CORE-FREEZE | skip | [DUAL-CORE-FREEZE.md](./long-plans/DUAL-CORE-FREEZE.md) |
| 410 | D-READ-03 | skip | [D-READ-03.md](./long-plans/D-READ-03.md) |

---

## 子 Agent 启动粘贴块（隔夜总控）

```text
按 oclive 债偿还马拉松 · 隔夜总控。严格遵守 OCLive 开发流水线与 AI 辅助限制。

必读（顺序不可跳）：
1. handoff/debt-marathon/AI_AND_PIPELINE_GATES.md
2. .cursor/skills/oclive-dev-pipeline/SKILL.md
3. .cursor/skills/oclive-debt-marathon/SKILL.md
4. handoff/debt-marathon/MARATHON_QUEUE.md
5. AGENTS.md · handoff/AI_CHANGE_BOUNDARIES.md

每个 Stage：
A. 父 Agent 先跑 check:debt-marathon，并 reconcile queue / plan / inventory / Git / PR-CI
B. 取 seq 最小 auto + Ready + 非 done/blocked；调用自定义子 Agent oclive-debt-stage
C. 普通实现 Stage 由子 Agent 做；Wave/QUEUE/TECHNICAL_DEBT 证据 Stage 由父 Agent 做，子 Agent 不得改 QUEUE
D. 父 Agent 校验返回 JSON 与实际 base SHA / diff / checks
E. 父 Agent 写 waves/WAVE-YYYYMMDD-<ID>-sN.md（勾选 GATES 清单）
F. 父 Agent 更新本表进度并写 cursor-marathon checkpoint；外部写入仅按 capability snapshot
G. 需人类决策/权限/RFC → blocked:<稳定错误码>；达到预算/无进展 → 写坐标并 finish

禁止：跳 Stage、扩 Full、动 skip/human「做完」、无 CI 写 Done、改无关 process_message、新建顶层 md、混入无关 dirty。
```

Cursor IDE 启动：

```powershell
npm run check:debt-marathon
node scripts/cursor-marathon.mjs start --max-turns 30
```
