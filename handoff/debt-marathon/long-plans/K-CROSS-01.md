# K-CROSS-01（Minimal）

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · G11/G14 · 禁顶层新 md

| 字段 | 值 |
|------|-----|
| **债 ID** | K-CROSS-01 |
| **台账** | Partial P2 · Minimal 文档已有证据，三平台实机仍缺 |
| **标题** | 三平台语音/宿主差异声明 + smoke 入口文档化 |
| **尺寸** | L |
| **Minimal / Full** | Minimal：文档矩阵；不做真跑三平台矩阵 CI |
| **Owner** | main-repo |
| **状态** | Blocked · needs-reconcile（历史文档已合入；等实机范围对账） |
| **更新** | 2026-09-28 |

**接手前提**：核已合入文档与当前平台声明，明确本册历史收口和实机 follow-up 的分界；三平台环境与执行授权未齐不得运行或声称 Full。依据见 [DCL-20260928-01](../DEBT_CHANGELOG.md#dcl-20260928-01--初始化审查与调度对账)。保留 Stage 坐标，禁止重派旧文档切片。

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-CROSS-01",
  "runner": "auto",
  "planStatus": "blocked",
  "parentDebtDisposition": "keep-open",
  "currentStage": 2,
  "prerequisites": ["Reconcile the merged documentation milestone and missing three-platform device evidence; define and authorize the remaining smoke scope before dispatch"],
  "stages": [
    {"id": 0, "title": "Verify platform facts", "files": ["read-only"], "actions": ["Read distro profiles, voice track and existing smoke entry points"], "checks": [{"command": "npm run test:distro-profile-mirror", "why": "The documented platform matrix must start from current profile parity"}], "outputs": ["Source-backed platform capability facts"], "rollback": "No writes; mark unknown hardware facts as human evidence"},
    {"id": 1, "title": "Document platform matrix", "files": ["creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md", "human-docs/team/TRACK_VOICE_RECOGNITION.md"], "actions": ["Add links and a compact platform-capability declaration without claiming unrun smoke"], "checks": [{"command": "node scripts/check-stale-paths.mjs --docs-only", "why": "The matrix links profiles and smoke commands"}], "outputs": ["Platform declaration; parent debt remains OPEN until real three-platform smoke"], "rollback": "Remove unsupported platform claims"},
    {"id": 2, "title": "Partial evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record documentation milestone and missing real-device evidence"], "checks": [{"command": "git diff --check", "why": "The documentation milestone must be whitespace-clean"}], "outputs": ["Wave with human smoke follow-up"], "rollback": "Do not mark K-CROSS-01 Done without three-platform evidence"}
  ]
}
-->

## 目标
- 在既有 DISTRO / VOICE SSOT 增加「平台 × 能力」差异表（Windows/Linux/macOS · ASR/TTS/webview）
- 链已有 smoke 命令；标明哪些仅人工
- 台账可 Minimal Done 或 Partial（若缺实机）

## 非目标
- 一夜补齐 Linux/mac CosyVoice 产品化（属 K-VOICE-03）

## Stages
0 对齐 → 1 写差异表（扩现有 md，禁顶层新文件）→ 2 台账+Wave+PR
