# V-MARKET-01（Minimal）

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · 跨仓改动须用户点名

| 字段 | 值 |
|------|-----|
| **债 ID** | V-MARKET-01 |
| **台账** | Partial P2 · Minimal SCOPE 已有证据，生态实现仍缺 |
| **标题** | 插件市场生态 SCOPE：现状 UI/站 vs 缺口 |
| **尺寸** | L |
| **Minimal / Full** | Minimal = SCOPE 单页；Full = 实现市场 UI |
| **Owner** | main-repo + 可能 `oclive-plugin-market` |
| **状态** | Blocked · needs-reconcile（历史 SCOPE 已合入；姊妹仓 human/cross-repo） |
| **更新** | 2026-09-28 |

**接手前提**：先核本仓已合入 SCOPE，再与 owner 确认剩余生态/姊妹仓任务；不从旧 PR 等待态推断跨仓许可。依据见 [DCL-20260928-01](../DEBT_CHANGELOG.md#dcl-20260928-01--初始化审查与调度对账)。本轮不打开或修改姊妹仓。

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "V-MARKET-01",
  "runner": "auto",
  "planStatus": "blocked",
  "parentDebtDisposition": "keep-open",
  "currentStage": 2,
  "prerequisites": ["Reconcile the merged main-repo SCOPE with remaining ecosystem work; obtain owner decisions and explicit sibling-repository authorization before implementation"],
  "stages": [
    {"id": 0, "title": "Inventory current market surfaces", "files": ["read-only"], "actions": ["Inspect main-repo CLI market code and existing product documentation; do not open sibling repositories"], "checks": [{"command": "rg -n \"market\" kernel/crates/oclive-cli/src handoff creator-docs", "why": "The SCOPE must be grounded in current main-repo surfaces"}], "outputs": ["Current capability and gap inventory"], "rollback": "No writes; sibling-repo facts remain human/cross-repo prerequisites"},
    {"id": 1, "title": "Write main-repo SCOPE", "files": ["handoff/PRODUCT_LINE_TASK_BUCKETS.md"], "actions": ["Add a compact market scope and link existing implementation anchors"], "checks": [{"command": "node scripts/check-stale-paths.mjs --docs-only", "why": "The scope links current CLI and documentation paths"}], "outputs": ["Main-repo market scope; parent ecosystem debt remains OPEN"], "rollback": "Remove duplicated facts and retain only links"},
    {"id": 2, "title": "Partial evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record main-repo scope and mark sibling implementation as human/cross-repo"], "checks": [{"command": "git diff --check", "why": "The SCOPE milestone is documentation-only"}], "outputs": ["Wave and explicit cross-repo follow-up"], "rollback": "Do not mark V-MARKET-01 Done from a SCOPE page"}
  ]
}
-->

## 目标（Minimal）
- 一篇 SCOPE（放 `handoff/` 子树或插件市场仓 README 链回）写清：今日能力、缺口、非目标
- 台账 Partial/Minimal Done 诚实

## 非目标
一夜做完社区上传审核与支付

## Stages
0 → 1 SCOPE（禁顶层乱建；优先手边仓文档）→ 2 Wave · 跨仓项标 human
