# 波次工作记录模板

> 复制为 `waves/WAVE-YYYYMMDD-<DEBT_ID>[-stageN].md`。链计划书与 PR；**勿复制** TECHNICAL_DEBT 长表。

---

## 摘要

| 字段 | 填写 |
|------|------|
| **债 ID** | |
| **长计划书** | `long-plans/<ID>.md` |
| **执行 Stage** | Stage N · 标题 |
| **分支 / PR** | |
| **日期** | YYYY-MM-DD |
| **执行面** | 本地 Agent · Cloud · Human |
| **状态三态** | Implemented / Locally verified / Done-eligible |
| **变动事件** | DEBT_CHANGELOG 事件 ID；没有事实变化写不适用 |

---

## 证据

| 项 | 值 |
|----|-----|
| HEAD SHA | |
| CI run | URL · 硬门禁结论 |
| 本地命令 | 列表 + PASS/FAIL |
| Base SHA | claim 时基线 |
| Changed / uncommitted files | 精确路径；无则 `none` |
| Claim / attempt | claim id · attempt N |

---

## 做了什么

- …

## 刻意没做什么（对照计划书非目标）

- …

## 阻断 / 下一 Stage

- last completed step：…
- last command + exit：…
- next exact command：…
- blocker code：… / none
- retry safe：yes/no · 理由

## GATES §6 出口

- [ ] 只动了本 Stage「文件范围」
- [ ] 已读 GATES §2–§3
- [ ] applicable 验收命令 PASS/FAIL 已列
- [ ] 未升错误 Done
- [ ] 未合 main（除非授权）
- [ ] 父 Agent 已更新 MARATHON_QUEUE 与 checkpoint
- [ ] 受影响的计划/台账已同步；[DEBT_CHANGELOG](DEBT_CHANGELOG.md) 只链接本 Wave 细节，不另抄状态长表（复制到 waves 后链接用 `../DEBT_CHANGELOG.md`）

## 台账

- 是否已改 TECHNICAL_DEBT：是/否 · 新状态  
- 是否禁止超前 Done：是  
