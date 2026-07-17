# K-PLUGIN-SEC-01 · 目录插件 UI 信任边界

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · 安全边界不得以功能回退换取假关闭

| 字段 | 值 |
|------|-----|
| **债 ID** | K-PLUGIN-SEC-01 |
| **台账** | `TECHNICAL_DEBT_INVENTORY.md` · K-PLUGIN-SEC-01 Partial |
| **标题** | 目录插件 opaque-origin、受控桥、full-shell 隔离与签名绑定 |
| **尺寸** | L |
| **Minimal / Full** | Minimal 已阻断发行版 inline Vue；本书推进 P1 Full |
| **Owner** | main-repo |
| **runner** | auto（本地实现）；原生 WebView 与远程 CI 证据不可省略 |
| **状态** | Ready · Stage 0 已完成 |
| **更新** | 2026-07-17 |

## AI + OCLive

- **必读门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md)
- **流水线：** dev-pipeline 七阶段 + oclive-dev-pipeline；尺寸 L 不跳安全审查、纪律、文档与总审
- **相关 G：** G3、G4、G7b、G8、G9、G11、G14；Tauri 命令只留在 `api/*.rs`
- **场景路径：** `AI_READING_INDEX.md` §9 技术债 + 目录插件；契约以 `DIRECTORY_PLUGINS.md` / `BRIDGE_API_REFERENCE.md` 为准
- **证据纪律：** 本地通过只记 Locally verified；没有目标提交远程 CI 与原生 WebView 证据不得写 Done

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "K-PLUGIN-SEC-01",
  "runner": "auto",
  "planStatus": "ready",
  "parentDebtDisposition": "done-eligible",
  "currentStage": 3,
  "prerequisites": [],
  "stages": [
    {
      "id": 0,
      "title": "Inventory plugin UI trust boundaries",
      "files": ["read-only"],
      "actions": ["Inventory inline Vue, embedded iframe, custom protocol, Tauri capability, bridge injection, full-shell and signing boundaries"],
      "checks": [
        {"command": "npm run check:debt-marathon", "why": "The new P1 plan must not weaken or conflict with the marathon queue"},
        {"command": "rg -n \"ocliveplugin|plugin_bridge|vueComponent|sandbox\" distros kernel", "why": "Every executable plugin UI and bridge entry must be included in the threat model"}
      ],
      "outputs": ["Threat-boundary inventory", "Ordered implementation stages", "Stable rollback points"],
      "rollback": "No code writes; keep K-PLUGIN-SEC-01 Partial when the inventory is incomplete"
    },
    {
      "id": 1,
      "title": "Opaque-origin embedded slot broker",
      "files": [
        "distros/shared/src/components/PluginSlotEmbed.vue",
        "distros/shared/src/utils/pluginFrameBridge.ts",
        "distros/shared/src/utils/pluginFrameBridge.test.ts",
        "distros/chat-pro/src/plugin-bridge.js",
        "kernel/crates/oclive_kernel_host/assets/plugin-bridge.iife.js",
        "kernel/crates/oclive_kernel_host/src/infrastructure/plugin_protocol.rs",
        "scripts/verify-frontend-patches.mjs"
      ],
      "actions": ["Sandbox embedded plugin frames without allow-same-origin", "Route bridge requests through a parent broker bound to the exact iframe contentWindow and declared plugin asset", "Reject malformed, cross-frame, replayed and undeclared bridge messages"],
      "checks": [
        {"command": "npm run test:unit -w @oclive/desktop-shared", "why": "The source-bound message broker and rejection cases are frontend security logic"},
        {"command": "node scripts/verify-frontend-patches.mjs", "why": "Release fail-closed and sandbox attributes require a static regression ratchet"},
        {"command": "npm run build -w @oclive/chat-pro", "why": "The broker and sandbox must compile in the production WebView bundle"}
      ],
      "outputs": ["Opaque-origin embedded slots", "Source-bound bridge broker", "Negative cross-plugin tests"],
      "rollback": "Do not enable sandbox in production until the broker and official HTML smoke pass; retain the existing release iframe-only gate"
    },
    {
      "id": 2,
      "title": "Official HTML parity and release compiler removal",
      "files": [
        "distros/chat-pro/plugins/com.oclive.voice.asr/slots/",
        "distros/chat-pro/src/__tests__/voiceHtmlFallback.spec.ts",
        "distros/chat-pro/src/plugin-bridge.js",
        "distros/chat-pro/package.json",
        "distros/desktop-tauri/src/api/plugin_bridge.rs",
        "distros/shared/src/components/AsyncPluginVue.vue",
        "distros/shared/src/components/PluginSlotEmbed.vue",
        "distros/shared/src/utils/pluginFrameBridge.ts",
        "distros/shared/src/utils/pluginFrameBridge.test.ts",
        "distros/shared/src/utils/compilePluginVueSfc.ts",
        "distros/shared/src/build/manualChunks.ts",
        "distros/shared/package.json",
        "kernel/crates/oclive_kernel_host/assets/plugin-bridge.iife.js",
        "scripts/verify-frontend-patches.mjs",
        "package.json",
        "package-lock.json"
      ],
      "actions": ["Make official Voice toolbar and settings HTML usable through the isolated broker", "Remove vue3-sfc-loader from the release dependency graph while retaining an explicit local development path if justified"],
      "checks": [
        {"command": "npm run test:unit", "why": "Official slot behavior and shared routing must retain functional parity"},
        {"command": "npm audit --omit=dev --audit-level=high", "why": "The vulnerable legacy compiler chain must no longer remain a production dependency"},
        {"command": "npm run build", "why": "The release bundle must work without the inline compiler chunk"}
      ],
      "outputs": ["Functional official HTML fallbacks", "No vue3-sfc-loader production dependency"],
      "rollback": "Keep the loader only behind an explicit non-release developer surface; never restore production inline Vue"
    },
    {
      "id": 3,
      "title": "Full-shell WebView isolation and capability narrowing",
      "files": [
        "distros/desktop-tauri/src/lib.rs",
        "distros/desktop-tauri/capabilities/plugin-shell-remote.json",
        "distros/desktop-tauri/tauri.conf.json",
        "distros/shared/src/utils/directoryShellBootstrap.ts",
        "distros/chat-pro/e2e/tauri-native.spec.ts"
      ],
      "actions": ["Move full-shell plugins to a distinct child WebView or an equivalent strong isolation boundary", "Bind Tauri capabilities to the isolated surface and remove broad custom-protocol remote IPC", "Add native tests proving host DOM and another plugin are inaccessible"],
      "checks": [
        {"command": "npm run test:e2e:tauri-native -- --grep \"plugin isolation\"", "why": "Origin and WebView behavior cannot be established by jsdom or static review"},
        {"command": "cargo test --locked -p oclivenewnew-tauri --tests", "why": "Protocol and capability routing are Tauri integration behavior"},
        {"command": "node scripts/dimension5-acceptance.mjs --ci", "why": "Tauri config and release frontend changes affect the L-level composite gate"}
      ],
      "outputs": ["Isolated full-shell surface", "Narrow per-surface capability", "Native isolation evidence"],
      "rollback": "Disable full-shell activation and keep the host UI; do not fall back to same-process shell Vue"
    },
    {
      "id": 4,
      "title": "Bind bridge authority to verified plugin identity",
      "files": [
        "kernel/crates/oclive_kernel_host/src/infrastructure/plugin_installer.rs",
        "kernel/crates/oclive_kernel_host/src/infrastructure/directory_plugins/",
        "distros/desktop-tauri/src/api/plugin_bridge.rs",
        "creator-docs/security/SUPPLY_CHAIN.md"
      ],
      "actions": ["Require a verified installation identity before production bridge authority", "Keep local development behind an explicit opt-out", "Document key rotation and revocation with K-SUPPLY-09"],
      "checks": [
        {"command": "cargo test -p oclive_kernel_host --lib", "why": "Installer identity and bridge authorization are host security invariants"},
        {"command": "cargo test --locked -p oclivenewnew-tauri --test invoke_hotpath_matrix", "why": "Bridge authority changes must not widen invoke access"},
        {"command": "node scripts/check-doc-mirror.mjs", "why": "Supply-chain policy is a mirrored public contract"}
      ],
      "outputs": ["Signature-bound bridge authority", "Rotation and revocation procedure", "Explicit development escape hatch"],
      "rollback": "Keep K-SUPPLY-09 and K-PLUGIN-SEC-01 open; disable unverified production bridge instead of silently trusting it"
    },
    {
      "id": 5,
      "title": "L-level evidence and honest closure",
      "files": [
        "handoff/debt-marathon/waves/",
        "handoff/TECHNICAL_DEBT_INVENTORY.md",
        "handoff/debt-marathon/MARATHON_QUEUE.md"
      ],
      "actions": ["Run applicable release and security gates", "Record target commit and remote CI", "Close only when embedded, full-shell, functional parity and verified identity conditions all hold"],
      "checks": [
        {"command": "npm run check:ci-local", "why": "P1 crosses frontend, Tauri, kernel, docs and supply-chain behavior"},
        {"command": "gh run view <RUN_ID> --json headSha,conclusion,url", "why": "L-level Done requires successful remote CI for the exact target commit"}
      ],
      "outputs": ["Locally verified or Done-eligible evidence", "Updated queue, wave and technical-debt state"],
      "rollback": "Keep the debt Partial and name the missing native, signature or CI evidence"
    }
  ]
}
-->

## Stage 0 结论

- 发行版同进程 Vue 已在前置提交中 fail-closed，但 `vue3-sfc-loader` 仍在生产依赖图。
- embedded iframe 没有 `sandbox`，所有插件资产共享 `https://ocliveplugin.localhost`；桥脚本直接调用 Tauri。
- Rust 端会校验请求中声明的 `plugin_id` / `asset_rel` / command，但当前没有浏览器 frame 身份可与该声明绑定，存在 confused-deputy 面。
- full-shell 直接替换主 WebView，并由 `plugin-shell-remote.json` 对 custom-protocol URL 开放远程 IPC；不能把 embedded iframe 修复冒充 full-shell 完成。
- 官方 Voice 的 HTML toolbar/settings 目前仅占位，因此安全默认已经造成真实功能回退，Stage 2 必须补齐。

## 硬边界

- 不通过简单改 hostname、随机 token 或静态扫描宣称隔离；必须证明调用者 frame 身份。
- sandbox 不允许 `allow-same-origin`；若 custom-protocol 子资源或 Tauri 平台不兼容，转 child WebView 设计并保持债 Partial。
- 不恢复发行版 inline Vue，不把本地开发逃生口默认为生产配置。
- 不在缺少原生 WebView E2E、签名撤销流程或远程 CI 时写 Done。

## 下一跳

Stage 1：先实现可独立单测的 source-bound parent broker，再启用 iframe sandbox；两者必须在同一 Stage 内形成不可绕过的闭环。
