# K-VOICE-07

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · **无 RFC §4.1 → Stage0 blocked**，禁自创协议

| 字段 | 值 |
|------|-----|
| **债 ID** | K-VOICE-07 |
| **台账** | OPEN P2 · voice_directive v2 |
| **标题** | directive v2 + engine_extras（**依赖 RFC §4.1**） |
| **尺寸** | L |
| **Minimal / Full** | 实现仅在 RFC 小节存在后 |
| **Owner** | main-repo |
| **状态** | Blocked |
| **更新** | 2026-07-16 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-VOICE-07",
  "runner": "auto",
  "planStatus": "blocked",
  "parentDebtDisposition": "keep-open",
  "currentStage": 0,
  "prerequisites": ["A specific RFC file and anchor must normatively define voice_directive v2 and engine_extras; the existing voice.asr section is not sufficient"],
  "stages": [
    {"id": 0, "title": "Verify exact directive-v2 RFC anchor", "files": ["read-only"], "actions": ["Resolve an exact normative file and anchor for voice_directive v2; do not accept an unrelated section numbered 4.1"], "checks": [{"command": "rg -n \"voice_directive.*v2|engine_extras\" creator-docs/rfc creator-docs/plugin-and-architecture", "why": "Implementation is forbidden until the exact directive-v2 contract exists"}], "outputs": ["Exact RFC link or blocked:needs-directive-v2-rfc-anchor"], "rollback": "No writes and no automatic retry while the prerequisite is unchanged"}
  ]
}
-->

## 目标
- RFC 语音相关 §4.1 落地后：schema + 透传 + 1 测
- 若无 RFC 小节：**Stage 0 即以 blocked:needs-RFC 结束**，勿发明协议

## 非目标
无 RFC 时自制 v2 破坏兼容

## Stages
0 查 **明确写出 `voice_directive` v2 + `engine_extras` 的具体 RFC 锚点** → 无则 blocked；不得用无关的 `voice.asr` §4.1 充当前置条件
