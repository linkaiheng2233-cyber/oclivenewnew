# T-DOC-02

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · oclive-dev-pipeline · AI_CHANGE_BOUNDARIES

| 字段 | 填写 |
|------|------|
| **债 ID** | T-DOC-02 |
| **台账锚点** | TECHNICAL_DEBT · Theater 状态单页 · OPEN P2 |
| **标题** | Theater STATUS 单页：模式/冻结/playtest 指针 |
| **尺寸** | L（文档债升 Done） |
| **Minimal / Full** | Minimal |
| **Owner / runner** | main-repo / **auto** |
| **状态** | Closed（PR #124 merged · CI success） |
| **最后更新** | 2026-07-18 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "T-DOC-02",
  "runner": "auto",
  "planStatus": "closed",
  "parentDebtDisposition": "done-eligible",
  "currentStage": 3,
  "prerequisites": [],
  "stages": [
    {"id": 0, "title": "Align Theater SSOT", "files": ["read-only"], "actions": ["Read gates, Theater README and status sources; verify STATUS.md is absent"], "checks": [{"command": "npm run check:debt-marathon -- --id T-DOC-02", "why": "The Ready plan contract must be valid before dispatch"}], "outputs": ["Confirmed non-duplicative STATUS scope"], "rollback": "No writes; report needs-reconcile if an existing SSOT already covers the scope"},
    {"id": 1, "title": "Write STATUS and index link", "files": ["handoff/theater/STATUS.md", "handoff/theater/README.md"], "actions": ["Create the one-page status SSOT and add one README link"], "checks": [{"command": "node scripts/check-stale-paths.mjs --docs-only", "why": "The Stage adds documentation paths"}, {"command": "git diff --check", "why": "The Stage is a documentation-only diff"}], "outputs": ["Theater STATUS page and README link"], "rollback": "Remove the new STATUS file and README link; do not update the debt ledger"},
    {"id": 2, "title": "Local evidence and Wave", "files": ["handoff/debt-marathon/waves/", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Write Wave evidence and let the parent mark pr_open only when authorized"], "checks": [{"command": "npm run check:debt-marathon", "why": "Queue and all Ready contracts must remain consistent"}], "outputs": ["Wave with resume coordinates"], "rollback": "Keep the queue pending if no PR capability or URL exists"},
    {"id": 3, "title": "Remote evidence", "files": ["handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["After merge, verify target CI and update the ledger and queue"], "checks": [{"command": "gh run view <RUN_ID> --json headSha,conclusion,url", "why": "A debt Done transition requires target remote CI success"}], "outputs": ["Done-eligible evidence tied to HEAD SHA"], "rollback": "Leave the parent debt OPEN and queue pr_open when remote CI is absent or failing"}
  ]
}
-->

## AI + OCLive

- **相关 G：** G11（不新建仓根 md）· G14（只链不复制 MODE2/ROADMAP 长文）· G3
- **场景路径：** 技术债收口 · `handoff/theater/`
- **流水线：** Stage0=① · Stage1=③⑥ · Stage2=⑤⑥⑦（证据）
- **隔夜：** 可开 PR；**不合 main**；Done Verification 白天合入后再写

## 目标（Done）

1. 新建 [`handoff/theater/STATUS.md`](../../theater/STATUS.md)
2. 内容一页内含：当前产品姿态（模式2）、**模式3仍冻**、链 `MODE2_*` / `PLAYTEST_MATRIX` / `DEVELOPMENT_ROADMAP` / TECHNICAL_DEBT 冻结节
3. [`handoff/theater/README.md`](../../theater/README.md) 增加指向 STATUS 的链接
4. TECHNICAL_DEBT T-DOC-02 → Done + Verification（合 main + CI 后）

## 非目标

- 解冻模式3 · 改 theater 运行时 · 陌生人真人 playtest · 复制 MODULE_MAP

## 影响域

`handoff/theater/STATUS.md`（新）· `handoff/theater/README.md` · `TECHNICAL_DEBT` · `debt-marathon/waves` · `MARATHON_QUEUE`

## 分阶段

### Stage 0 · 对齐
| 项 | 内容 |
|----|------|
| 动作 | 确认 STATUS 不存在；读 theater README；使用 Cursor 独立 worktree；读 GATES |
| 验收 | GATES 已读；无未决产品选择 |

### Stage 1 · 起草 STATUS + README 链
| 项 | 内容 |
|----|------|
| 文件范围 | `handoff/theater/STATUS.md` · `handoff/theater/README.md` |
| 动作 | 写 STATUS；README 加一行链接 |
| 验收命令 | `node scripts/check-stale-paths.mjs --docs-only`（若 applicable）· `git diff --check` |
| 产出 | 分支 `debt/t-doc-02-theater-status` · 提交 |
| 流水线 | ③⑥ |
| 失败回退 | 删新建 STATUS；不改台账 |

### Stage 2 · PR + Wave（隔夜）
| 项 | 内容 |
|----|------|
| 动作 | 开 PR；写 `waves/WAVE-*-T-DOC-02-s2.md`；QUEUE 进度 `pr-open`；**不合 main** |
| 验收 | PR URL；GATES §6 勾选 |

### Stage 3 · 证据（白天合入后）
| 项 | 内容 |
|----|------|
| 动作 | merge 后填 TECHNICAL_DEBT Done + CI SHA；QUEUE `done`；本书 Closed |
| Done-eligible | main 硬门禁 success |

## 子 Agent · Stage 1

```text
按 oclive 债偿还马拉松 · Implementer。先读 AI_AND_PIPELINE_GATES。
仅 Stage 1：STATUS.md + README 链。禁解冻模式3、禁运行时、禁顶层 md、禁合 main、禁改台账 Done。
验收：stale-paths docs（applicable）· diff --check · Wave 草稿字段。
```
