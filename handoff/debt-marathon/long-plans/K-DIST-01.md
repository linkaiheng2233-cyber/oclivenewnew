# K-DIST-01（Minimal）

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · 禁夜间玩证书/商店上传

| 字段 | 值 |
|------|-----|
| **债 ID** | K-DIST-01 |
| **台账** | OPEN P2 · 签名/updater/包 |
| **标题** | 分发缺口清单与分期路线（非一次上签名） |
| **尺寸** | L |
| **Minimal / Full** | Minimal = 缺口 STATUS；Full = 真签名+updater 另册 |
| **Owner** | main-repo（文档）；Full 或需 Human 证书 |
| **状态** | Ready |
| **更新** | 2026-07-16 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-DIST-01",
  "runner": "auto",
  "planStatus": "ready",
  "parentDebtDisposition": "keep-open",
  "currentStage": 1,
  "prerequisites": [],
  "stages": [
    {"id": 0, "title": "Inventory distribution gaps", "files": ["read-only"], "actions": ["Inspect current Tauri bundling, updater and platform packaging declarations"], "checks": [{"command": "npm run check:debt-marathon -- --id K-DIST-01", "why": "The documentation milestone must not be confused with Full distribution closure"}], "outputs": ["Verified signing, updater and package gap list"], "rollback": "No writes"},
    {"id": 1, "title": "Write distribution status", "files": ["handoff/distros/README.md"], "actions": ["Add a bounded gap and human-dependency section to the existing distro SSOT"], "checks": [{"command": "node scripts/check-stale-paths.mjs --docs-only", "why": "The gap list references packaging paths"}], "outputs": ["Distribution milestone and explicit human prerequisites"], "rollback": "Remove the section if it duplicates another distro SSOT"},
    {"id": 2, "title": "Partial evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record the documentation milestone; leave signing, updater and packages OPEN"], "checks": [{"command": "git diff --check", "why": "The documentation-only milestone must be clean"}], "outputs": ["Wave with blocker codes for signing secrets and real platform builds"], "rollback": "Never mark the parent debt Done from documentation alone"}
  ]
}
-->

## 目标（Minimal）
- `handoff/distros/` 或既有分发文：缺口表（签名 · updater · Linux 包 · dmg）· 优先级 · 依赖人工项
- 台账 Partial 或 Minimal Done（文案诚实）

## 非目标
夜间采购证书、上传商店、改用户安装体验大重构

## Stages
0 → 1 缺口文档 → 2 台账+Wave · Full 标 blocked:needs-signing-secrets
