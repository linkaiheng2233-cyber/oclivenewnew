# K-VOICE-06（Minimal）

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · 禁顺手实现 K-VOICE-02 全家桶

| 字段 | 值 |
|------|-----|
| **债 ID** | K-VOICE-06 |
| **台账** | OPEN P2 · 社区 TTS 插件白名单 |
| **标题** | `com.user.tts.*` RPC 白名单契约 + 测试钩 |
| **尺寸** | L |
| **Minimal / Full** | Minimal：白名单文档+1 测；非多个社区插件产品化 |
| **Owner** | main-repo |
| **状态** | Ready |
| **更新** | 2026-07-16 |

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-VOICE-06",
  "runner": "auto",
  "planStatus": "ready",
  "parentDebtDisposition": "done-eligible",
  "currentStage": 2,
  "prerequisites": [],
  "stages": [
    {"id": 0, "title": "Locate RPC authorization SSOT", "files": ["read-only"], "actions": ["Read PLUGIN_V1, voice track and plugin_rpc_invoke enforcement code"], "checks": [{"command": "rg -n \"plugin_rpc_invoke|rpcMethods|voice\\.speak\" creator-docs distros/desktop-tauri/src", "why": "The whitelist contract must match implemented authorization"}], "outputs": ["Exact whitelist and enforcement anchors"], "rollback": "No writes; block if the desired method requires a new permission model"},
    {"id": 1, "title": "Document TTS RPC surface", "files": ["creator-docs/plugin-and-architecture/PLUGIN_V1.md", "creator-docs-en/plugin-and-architecture/PLUGIN_V1.md"], "actions": ["Document the allowed community TTS RPC surface in both contract mirrors"], "checks": [{"command": "node scripts/check-doc-mirror.mjs", "why": "PLUGIN_V1 is a mirrored creator contract"}], "outputs": ["Mirrored RPC whitelist contract"], "rollback": "Revert both mirrors together"},
    {"id": 2, "title": "Add rejection contract test", "files": ["distros/desktop-tauri/tests/", "distros/desktop-tauri/src/api/plugin_bridge.rs"], "actions": ["Add or tighten one test proving undeclared TTS RPC methods are rejected without broadening runtime permissions"], "checks": [{"command": "cargo test -p oclivenewnew-tauri plugin_rpc", "why": "The Stage changes the desktop RPC authorization contract"}], "outputs": ["Focused invalid-invoke rejection test"], "rollback": "Revert the test and any behavior change together; do not loosen the whitelist"},
    {"id": 3, "title": "Evidence", "files": ["handoff/debt-marathon/waves/", "handoff/TECHNICAL_DEBT_INVENTORY.md", "handoff/debt-marathon/MARATHON_QUEUE.md"], "actions": ["Record local and remote evidence before a Done transition"], "checks": [{"command": "gh run view <RUN_ID> --json headSha,conclusion,url", "why": "The parent debt Done transition requires target CI success"}], "outputs": ["Done-eligible whitelist evidence"], "rollback": "Keep K-VOICE-06 OPEN if enforcement or CI evidence is incomplete"}
  ]
}
-->

## 目标
- PLUGIN_V1 / VOICE TRACK 或既有 RPC 文档：`plugin_rpc_invoke` 允许的 TTS 方法面
- 1 个契约测或脚本断言非法 invoke 拒绝
- 台账 Minimal Done

## 非目标
实现 ChatTTS/XTTS 全家桶（K-VOICE-02）

## Stages
0 读现有 rpc 白名单 → 1 文档 → 2 测 → 3 证据
