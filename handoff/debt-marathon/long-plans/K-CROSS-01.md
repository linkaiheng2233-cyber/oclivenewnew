# K-CROSS-01（Minimal）

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · G11/G14 · 禁顶层新 md

| 字段 | 值 |
|------|-----|
| **债 ID** | K-CROSS-01 |
| **台账** | Partial P2 · Minimal 文档已完成，按声明能力的宿主符合性证据仍缺 |
| **标题** | 三平台语音/宿主差异声明 + smoke 入口文档化 |
| **尺寸** | L |
| **Minimal / Full** | Minimal：文档矩阵；不做真跑三平台矩阵 CI |
| **Owner** | main-repo |
| **状态** | Closed · 仅本册 Minimal；父债仍 Partial |
| **更新** | 2026-09-30 |

**对账结论**：本册的发行版/OS 差异声明与 smoke 入口已随 [PR #126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126) 合入；当前 profile 镜像检查通过。本册只记录历史 Minimal 里程碑，不重派旧 Stage。维护者已将父债的后续验收限定为**按各平台已声明的能力核宿主符合性**，不再要求 Linux/macOS 在产品尚标为 `unsupported` 时跑完整内置语音闭环。现行 OS 能力与待测证据见 [语音轨道的平台表](../../../human-docs/team/TRACK_VOICE_RECOGNITION.md#平台--asr--tts--webview差异声明)，裁定见 [DCL-20260930-04](../DEBT_CHANGELOG.md#dcl-20260930-04--k-cross-01-按声明能力划分宿主验收)。队列 `done` 仅关闭本册 Minimal；父债保持 Partial。

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-CROSS-01",
  "runner": "auto",
  "planStatus": "closed",
  "parentDebtDisposition": "keep-open",
  "currentStage": 2,
  "prerequisites": [],
  "stages": [
    {"id": 0, "title": "Verify platform facts", "files": ["read-only"], "actions": ["Read distro profiles, voice track and existing smoke entry points"], "checks": [{"command": "npm run test:distro-profile-mirror", "why": "The documented platform matrix must start from current profile parity"}], "outputs": ["Source-backed platform capability facts"], "rollback": "No writes; mark unknown hardware facts as human evidence"},
    {"id": 1, "title": "Document platform matrix", "files": ["creator-docs/kernel/DISTRO_CAPABILITY_PROFILE.md", "human-docs/team/TRACK_VOICE_RECOGNITION.md"], "actions": ["Add links and a compact platform-capability declaration without claiming unrun smoke"], "checks": [{"command": "node scripts/check-stale-paths.mjs --docs-only", "why": "The matrix links profiles and smoke commands"}], "outputs": ["Platform declaration; parent debt remains OPEN until platform-appropriate conformance evidence"], "rollback": "Remove unsupported platform claims"},
    {"id": 2, "title": "Partial evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record documentation milestone and missing platform-appropriate conformance evidence"], "checks": [{"command": "git diff --check", "why": "The documentation milestone must be whitespace-clean"}], "outputs": ["Wave with supported and unsupported path follow-up"], "rollback": "Do not mark the parent K-CROSS-01 Done from documentation alone"}
  ]
}
-->

## 目标
- 在既有 DISTRO / VOICE SSOT 增加「平台 × 能力」差异表（Windows/Linux/macOS · ASR/TTS/webview）
- 链已有 smoke 命令；标明哪些仅人工
- 本册 Minimal 可关闭；父债在按声明能力的宿主符合性证据齐备前仍 Partial

## 非目标
- Linux/macOS 内置 CosyVoice TTS 产品化（属 K-VOICE-03），以及 sherpa ASR 产品化（须单独确定产品范围）
- 要求 `unsupported` 的 Linux/macOS 执行不存在的内置 ASR→chat→TTS 闭环

## Stages
0 对齐 → 1 写差异表（扩现有 md，禁顶层新文件）→ 2 台账+Wave+PR

## 父债后续验收边界（未执行）

- Windows 已声明交付的内置 ASR/TTS：需在支持的宿主路径取得实际 ASR→chat→TTS 正向 smoke 证据。
- Linux/macOS 已声明 `unsupported` 的内置 sherpa ASR 与 CosyVoice TTS：需核真实宿主/界面按 profile 明确拒绝或禁用，不尝试启动未交付的内置引擎；这不是要求两平台真实出声。
- 用户自建 `local_http` / `cloud` 端点仅核配置/边界合同；外部服务质量及可用性不由 K-CROSS 保证。
- 上述证据、具体环境、失败止点与新身份须在新计划中冻结后执行；本册旧 Stage 与 [历史 Wave](../waves/WAVE-20260716-K-CROSS-01-s2.md) 不作为新测试授权。K-VOICE-03 只拥有 CosyVoice TTS 产品化，跨平台 sherpa ASR 仍待独立产品范围，不借其名关闭。
