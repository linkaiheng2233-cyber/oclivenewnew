# K-SUPPLY-05-Full

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · Cargo.lock→audit+KNOWN · 假 Full Done 禁止

| 字段 | 值 |
|------|-----|
| **债 ID** | K-SUPPLY-05-Full（Minimal 已 Done） |
| **台账** | Minimal Done；Full = 零 `[bans.skip]` |
| **标题** | 消除 skip / 收敛 duplicate 至可无例外 deny |
| **尺寸** | L |
| **Minimal / Full** | **Full** |
| **Owner** | main-repo |
| **状态** | Blocked（零 skip 仍 needs-ecosystem） |
| **更新** | 2026-07-16 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-SUPPLY-05-Full",
  "runner": "auto",
  "planStatus": "blocked",
  "parentDebtDisposition": "done-eligible",
  "currentStage": 3,
  "prerequisites": ["cargo deny bans with empty [bans.skip] requires upstream Tauri/windows/sqlx/toml ecosystem convergence"],
  "stages": [
    {"id": 0, "title": "Baseline duplicate families", "files": ["read-only"], "actions": ["Capture deny skips, duplicate tree and current ratchet"], "checks": [{"command": "cargo tree -d", "why": "Full scope is defined by current duplicate families"}, {"command": "cargo deny check bans", "why": "The initial deny state is the comparison baseline"}], "outputs": ["Prioritized duplicate-family baseline"], "rollback": "No writes"},
    {"id": 1, "title": "Converge one dependency family", "files": ["Cargo.toml", "Cargo.lock", "deny.toml"], "actions": ["Converge exactly one compatible family per dispatch"], "checks": [{"command": "cargo deny check bans", "why": "The family change must not require an undocumented skip"}, {"command": "cargo audit", "why": "Cargo.lock changes require supply-chain verification"}], "outputs": ["One reviewed dependency-family reduction"], "rollback": "Revert the family change when compatibility or audit regresses"},
    {"id": 2, "title": "Remove final skips", "files": ["deny.toml", "Cargo.lock"], "actions": ["Remove skip entries only after all families are compatible"], "checks": [{"command": "cargo deny check bans", "why": "Full closure requires bans with no skip exception"}, {"command": "node scripts/check-cargo-dedup-ratchet.mjs", "why": "Duplicate groups must not regress"}], "outputs": ["Empty bans.skip and green deny gate"], "rollback": "Restore the documented skip and keep Full open if the ecosystem cannot converge"},
    {"id": 3, "title": "Remote evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record audit, deny and target CI evidence"], "checks": [{"command": "gh run view <RUN_ID> --json headSha,conclusion,url", "why": "Full Done requires target remote CI success"}], "outputs": ["Done-eligible Full evidence or honest remaining skip"], "rollback": "Keep Full OPEN on any remaining skip or missing CI"}
  ]
}
-->

## 目标
- `deny.toml` multiple-versions=deny 且 **skip 列表为空**（或仅保留有期限的紧急项并台账注明）
- `cargo deny check bans` PASS · dedup ratchet 不升
- TECHNICAL_DEBT 注明 Full Done 或仍 Partial+剩余理由

## 非目标
- 为消重破坏 Tauri/sqlx 功能 · 盲 pin 导致漏洞

## 分阶段
### Stage 0 · 导出当前 skip 与 cargo tree -d
### Stage 1 · 按族收敛（可多 PR）：每次只动一小族依赖 · audit
### Stage 2 · 删除 skip 条目 · deny 绿
### Stage 3 · 证据

## 停条件
生态不可消 → 停止 · Wave 写「仍须 skip: …」· **不准假 Full Done**。
