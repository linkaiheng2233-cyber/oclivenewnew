# FOLLOWUP-VOICE-04-PR123

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · 隔夜默认检查 PR 不开合 main（除非用户授权）

| 字段 | 填写 |
|------|------|
| **债 ID** | FOLLOWUP-VOICE-04-PR123（跟随 K-VOICE-04） |
| **台账锚点** | K-VOICE-04 已 Done；本项收口 inherit-provider |
| **标题** | 合入 PR #123：继承全局 TTS 时保留 settings `synth_provider` |
| **尺寸** | L（合 main + CI） |
| **Minimal / Full** | Minimal |
| **Owner 轨道** | main-repo |
| **状态** | Closed |
| **最后更新** | 2026-07-16 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "FOLLOWUP-VOICE-04-PR123",
  "runner": "auto",
  "planStatus": "closed",
  "parentDebtDisposition": "done-eligible",
  "currentStage": 2,
  "prerequisites": [],
  "stages": [
    {
      "id": 2,
      "title": "Reconcile merged evidence",
      "files": ["handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"],
      "actions": ["Verify PR 123 merge and target CI, then reconcile plan and queue without changing product code"],
      "checks": [{"command": "gh pr view 123 --json state,mergeCommit,statusCheckRollup", "why": "The close decision depends on current remote PR and CI evidence"}],
      "outputs": ["Closed plan and done queue entry"],
      "rollback": "Restore pending only if remote evidence contradicts the inventory"
    }
  ]
}
-->

## 目标
- [PR #123](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/123) MERGED 或等价 commit 在 main
- main 硬门禁绿；TECHNICAL_DEBT Verification 补一行「inherit-provider 已合」
- VOICE-04 保持 Done（不降级）

## 非目标
- 重写 voiceTtsRouting 大逻辑 · VX-12 · 新引擎

## OCLive
- G11 台账证据 · AI_VERIFICATION_PROTOCOL
- 影响域：merge · `voiceTtsRouting` / `useVoiceAutoTts`（已在 PR）· TECHNICAL_DEBT

## 分阶段

### Stage 0 · 对齐
确认 PR OPEN/MERGEABLE、硬门禁绿；与 origin/main 无冲突。

### Stage 1 · 合并（人类白天默认；隔夜仅开检查）
- 隔夜 `auto`：**复核 checks 仍绿**；若用户授权合则 `gh pr merge 123`；否则只记录「可合」并开/确认 PR，进度 `pr-open`
- 白天：merge + pull

### Stage 2 · 证据
main CI success · TECHNICAL_DEBT 补合入句 · Wave log · 本索引进度 done · 本书 Closed

## 子 Agent · Stage 1（隔夜默认）
```text
仅 Stage 1：gh pr checks 123；勿 merge（除非用户曾授权合 main）。
产出：checks 摘要 · Wave · MARATHON_QUEUE 进度=pr-open 或 blocked。
```
