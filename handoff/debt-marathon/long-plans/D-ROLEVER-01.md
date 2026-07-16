# D-ROLEVER-01

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · oclive-dev-pipeline · BOUNDARIES（G11/G14；中英契约走 check-doc-mirror）

| 字段 | 值 |
|------|-----|
| **债 ID** | D-ROLEVER-01 |
| **台账** | OPEN P2 · ROLE_PACK 版本迁移章节 |
| **标题** | ROLE_PACK_SPEC 增加角色包版本迁移规范章节 |
| **尺寸** | L（文档契约） |
| **Minimal / Full** | Minimal：SPEC 章节 + 链 INDEX；不做自动迁移工具 |
| **Owner** | main-repo |
| **状态** | Ready |
| **更新** | 2026-07-16 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "D-ROLEVER-01",
  "runner": "auto",
  "planStatus": "ready",
  "parentDebtDisposition": "done-eligible",
  "currentStage": 0,
  "prerequisites": [],
  "stages": [
    {"id": 0, "title": "Locate canonical migration wording", "files": ["read-only"], "actions": ["Read ROLE_PACK_SPEC and its English mirror; reject duplicate sections"], "checks": [{"command": "npm run check:debt-marathon -- --id D-ROLEVER-01", "why": "The Ready contract must be complete before editing creator contracts"}], "outputs": ["Exact insertion anchors and mirror decision"], "rollback": "No writes; block on conflicting SSOT"},
    {"id": 1, "title": "Write version migration contract", "files": ["creator-docs/role-pack/ROLE_PACK_SPEC.md", "creator-docs-en/role-pack/ROLE_PACK_SPEC.md"], "actions": ["Add equivalent version and migration sections without runtime changes"], "checks": [{"command": "node scripts/check-doc-mirror.mjs", "why": "Creator contract changes require Chinese-English parity"}, {"command": "node scripts/check-stale-paths.mjs --docs-only", "why": "The new section links schema and manifest paths"}], "outputs": ["Mirrored migration contract section"], "rollback": "Revert both mirrored sections together"},
    {"id": 2, "title": "Evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record local checks; update debt Done only after target CI success"], "checks": [{"command": "git diff --check", "why": "The complete documentation diff must be whitespace-clean"}], "outputs": ["Wave and evidence-state transition"], "rollback": "Keep debt OPEN when remote evidence is missing"}
  ]
}
-->

## 目标
- `creator-docs/role-pack/ROLE_PACK_SPEC.md`（及 EN mirror 若 applicable）有「版本 / 迁移」专节：semver 期望、破坏性字段、推荐迁移步骤、与 `schema`/manifest 关系
- TECHNICAL_DEBT → Done + Verification
- 非目标：写迁移 CLI、改所有角色包 JSON

## 分阶段
### Stage 0 · 对齐
读 ROLE_PACK_SPEC 现有版本措辞；确认无重复章节。

### Stage 1 · 写章节
文件：ROLE_PACK_SPEC（+ EN 若 check-doc-mirror 要求）· 短链 DOCUMENTATION_INDEX / human 路径若需  
验收：`check-doc-mirror`（若改契约镜像）· `check-stale-paths --docs-only` · git diff --check

### Stage 2 · 证据
台账 Done · Wave · PR（不合 main  overnight）

## 子 Agent Stage 1
仅写 SPEC 迁移章；禁止改运行时解析代码。
