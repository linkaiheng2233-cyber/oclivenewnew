# K-RESILIENCE-01（Minimal）

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · 禁无 RFC 发明大层 · 尺寸 L · 一 Stage 一会话

| 字段 | 值 |
|------|-----|
| **债 ID** | K-RESILIENCE-01 |
| **台账** | OPEN P2 · Remote 弹性分散 |
| **标题** | Minimal：Remote 超时/重试调用点清单 + 单一入口约定（非整库重写） |
| **尺寸** | L |
| **Minimal / Full** | **本册=Minimal**。Full ResilienceLayer 另开书 |
| **Owner** | main-repo |
| **状态** | Ready |
| **更新** | 2026-07-16 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-RESILIENCE-01",
  "runner": "auto",
  "planStatus": "ready",
  "parentDebtDisposition": "keep-open",
  "currentStage": 3,
  "prerequisites": [],
  "stages": [
    {"id": 0, "title": "Inventory remote resilience", "files": ["read-only"], "actions": ["Map timeout, retry and fallback call sites against REMOTE_PLUGIN_PROTOCOL"], "checks": [{"command": "rg -n \"timeout|retry|fallback\" kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin", "why": "The inventory must be derived from current source"}], "outputs": ["Verified call-site inventory and candidate canonical helper"], "rollback": "No writes; block if the scope requires an unapproved architecture decision"},
    {"id": 1, "title": "Document the inventory", "files": ["creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md"], "actions": ["Add a compact code-anchor inventory and a new-code canonical entry rule"], "checks": [{"command": "node scripts/check-stale-paths.mjs --docs-only", "why": "The inventory contains source paths"}], "outputs": ["Remote resilience inventory"], "rollback": "Remove the new section without changing the debt state"},
    {"id": 2, "title": "One representative wiring", "files": ["kernel/crates/oclive_kernel_host/src/infrastructure/remote_plugin/", "kernel/crates/oclive_kernel_host/src/infrastructure/remote_fallback_policy.rs"], "actions": ["Consolidate one behavior-equivalent call path and add a focused test; do not introduce a Full layer"], "checks": [{"command": "cargo test -p oclive_kernel_host remote_plugin", "why": "The Stage changes one host remote path"}, {"command": "node scripts/check-domain-layering.mjs", "why": "The Stage touches host infrastructure boundaries"}], "outputs": ["One tested canonical example; parent debt remains OPEN"], "rollback": "Revert the representative wiring and retain the inventory as Partial evidence"},
    {"id": 3, "title": "Partial evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record Partial evidence and leave Full ResilienceLayer OPEN"], "checks": [{"command": "npm run check:debt-marathon", "why": "The parent-open disposition must remain machine-valid"}], "outputs": ["Wave and explicit Full follow-up"], "rollback": "Do not mark the parent debt Done"}
  ]
}
-->

## 目标（Minimal Done）
- `handoff/` 或既有 remote 文档下 **Inventory 表**：列出 Remote 超时/重试/fallback 代码锚点（文件:行级）
- 约定「新代码必经」的单一帮助函数或 module 路径（可先文档约定 + 1 个现有路径示范接线，**禁止**大爆炸重写全部调用点）
- 1 个定向测试或契约测证明示范路径行为
- 台账：可标 **Partial**（若仅清单）或 Minimal Done（若示范接线+测）；文中写明 Full 仍 OPEN

## 非目标
- 一次替换全部 HTTP 客户端策略 · 改六槽 · 新依赖框架

## 分阶段
### Stage 0 · 对齐
grep remote timeout/retry；读 REMOTE_PLUGIN_PROTOCOL。

### Stage 1 · Inventory 文档
新建或扩已有 handoff 文（**禁止顶层新 md**；优先扩 MODULE_MAP 短锚或 creator-docs remote 节）列出锚点表。

### Stage 2 · 示范接线 + 测（可选升 Minimal Done）
抽公共 helper（若易）· 1 call site · 测试。

### Stage 3 · 证据
台账诚实状态 · Wave · PR

## 停条件
若 Full 层设计需 RFC → Stage0 后 `blocked:needs-RFC`，勿自行发明大层。
