# Technical debt inventory

**Last updated:** 2026-08-12??? 29 ???B M1 ?? 0?4 ????? `60d90d5b` / `1fe96cff` / `906ddf7a` / `a6984d9f` / `3cb6806b`???????????????????????LLM ???????????`FailingLlmClient` ? ???? + ?????host lib 454/454?clippy `-D warnings`?fmt ????K-EMO-02 ????M1 ?? ? ????????K-EMO-03 ???builtin ?? provider ??????????????**?? 29 ?????2026-08-12?**???? playtest ???turn 23/24 `bot_emotion=happy` + `emotion_source=llm` ?? + narrative hint ???? K-EMO-02 ??????release ?????`3cb6806b` ?????? runtime shared + bundled resources??????? `runtime/backups/`??? K-EMO-04?dev bundled ???? + manifest builtAt ?????? 2026-08-11 ??????

**Product freeze (Theater v0):** **Lifted** — 朋友 cohort 产品门通过（7/10 卧槽）；模式 2 playtest 扩展中；**模式 3 仍冻结**。见 [theater/MODE2_UNFREEZE.md](./theater/MODE2_UNFREEZE.md)。

**综合评分：** A− · 冻结实现 `728219e7` 的主 CI [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) **16/16 success** · 完整 Nightly [`30714480898`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714480898) **6/6 success** · 审查数字 SSOT：[AI_VERIFICATION_PROTOCOL.md](./AI_VERIFICATION_PROTOCOL.md)

**下一动作：** 先完成 **K-VOICE-09** 的 30 分钟真实矩阵与人工听感验收，并为 **K-RESOURCE-COORD-01** 的不可控进程/长时硬件 soak 打基础；随后按愿景验证顺序推进 **V-MODULE-QUALITY-01 → V-PORTABLE-01 Full → V-EMBED-01**。发行版仍需 **V-VSCODE-PERF-05** 的姊妹仓 F5 / `.vsix` 实机证据；**K-PLUGIN-SEC-01、K-SUPPLY-09、K-SUPPLY-10 仍须保持显式 OPEN / Partial，不因排期后移而降格或消失。** 短期动作：**情绪词表结构设计（2026-08-12 维护者主导开工，词表=种类辅助建议、复杂情绪层=程度、LLM 主生成零额外调用、降级规则引擎兜底）**；B/C 组按 K-DISTRO-01 / K-VOICE-10 解冻条件恢复。

**马拉松计划书：** [`debt-marathon/`](./debt-marathon/README.md) · 总索引 [`MARATHON_QUEUE.md`](./debt-marathon/MARATHON_QUEUE.md) · **AI+流水线硬门禁** [`AI_AND_PIPELINE_GATES.md`](./debt-marathon/AI_AND_PIPELINE_GATES.md) · 覆盖 [`COVERAGE.md`](./debt-marathon/COVERAGE.md)；Skill：`oclive-debt-marathon`。

**Verification (2026-07-16 · K-VOICE-06 Minimal):** HEAD `b8cb0c48` · CI [`29465172205`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29465172205) **硬门禁 success**（`rust` windows+ubuntu / `dimension5` / `cargo-audit` / `cli` / `oocp` / `cross-host-e2e` / `stale-paths`；`frontend` / `e2e-tauri` / `loom` success）· PLUGIN_V1 双镜 `com.user.tts.*` RPC 契约 · 拒测 `community_tts_plugin_rpc_rejects_undeclared_speak` · **Done（Minimal · VX-10）** · ≠ K-VOICE-02 产品化 · PR [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126)。

**Verification (2026-07-16 · K-VOICE-04 inherit-provider → main):** 已合 `origin/main` · merge `262c9ac4` · CI [`29432835462`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29432835462) **硬门禁 success**（`rust` windows+ubuntu / `dimension5` / `cargo-audit` / `cli` / `oocp` / `cross-host-e2e` / `stale-paths`；`frontend` / `e2e-tauri` / `loom` success）· 继承 TTS 时保留 settings `synth_provider` · K-VOICE-04 仍 **Done**（不降级）· 下一动作仍 **V-VSCODE-PERF-05** · PR [#123](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/123)。

**Verification (2026-07-17 · local security/documentation pass):** 本地 `cargo test -p oclive_validation`（95 tests）、`cargo test -p oclive_kernel_host --lib`（283 tests）、Tauri `http_api_high_risk_auth`（2）与 `invoke_hotpath_matrix`（5）通过；全量 Tauri 集成编译曾因 Windows 页面文件耗尽（`os error 1455`）中止，**不视为远端 CI 结论**。本轮收敛 HTTP API 默认令牌、角色/插件/ZIP 路径 containment、活跃文档链接与边界叙述。

**Verification (2026-07-17 · local P0 security follow-up):** 高置信 secret scan 定位 `.continue/agents/new-config.yaml` 自初始提交 `cdfa20e6` 起含同一枚 API 密钥；工作树已改用 `${{ secrets.N1N_API_KEY }}`。维护者已于 2026-07-17 确认在 N1N 提供商侧彻底销毁旧密钥，并明确保留 Git 历史，因此 K-SECRET-01 关闭；历史明文仍可见，但已不再具备服务端权限。发行构建现禁止目录插件 Vue 在主 WebView 同进程执行；不安全 inline Vue 仅限 Vite DEV + `VITE_OCLIVE_UNSAFE_INLINE_PLUGIN_VUE=1` 双重显式 opt-in。独立插件 origin / 签名默认开启尚未完成，见 K-PLUGIN-SEC-01 / K-SUPPLY-09。

**Verification (2026-07-18 · local engineering closeout):** `npm run lint` 已覆盖 Chat Pro、shared、Theater、Playwright E2E 与主要 Vite/Vitest 配置并通过，且已接入日常 `check` 与 `check:release`；Vite 生产构建 **784 modules**；前端 shared **18 files / 70 tests**、Chat Pro（含 Theater）**21 files / 83 tests**、浏览器 preview E2E **6 tests** 全绿；`npm run check`、`npm run check:release`（含 workspace doctest）、`npm run check:rust:integration`、Dimension 5 **PASS (24 checks)**。文档镜像、stale paths、注册表（**43** 个 handoff 根文档）、canonical blocks（**5**）、Markdown links（**49** 个文档）均通过；生产依赖 `npm audit --omit=dev --audit-level=high` 为 **0 vulnerabilities**。本条仅代表本地证据，未声称远端 CI / 原生 WebView E2E。

**Verification (2026-07-17 · local closeout):** `cargo clippy --workspace --all-targets --all-features -- -D warnings`、宿主 lib **285/285**、validation **95/95**、workspace doctest **6/6**、Chat Pro **77/77**、shared **63/63**、Theater **52/52**、Dimension 5 **PASS (23 checks)**；`npm audit --omit=dev --audit-level=high` 为 **0 高危 / 4 中危 / 0 低危**，均在 `vue3-sfc-loader` 的旧 Vue 编译依赖链，未当作已清零。

**Verification (2026-07-15 · K-VOICE-04 Minimal):** HEAD `2d5123af` · CI [`29408795870`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29408795870) **硬门禁 success**（`rust` windows+ubuntu / `dimension5` / `cargo-audit` / `cli` / `oocp` / `cross-host-e2e` / `stale-paths`；`frontend` / `e2e-tauri` / `loom` success）· 删除 `useRoleVoiceProfileSync` · `voiceTtsRouting` 任务级路由 · Warm 按 profile 隔离 · **Done（Minimal · VX-11）** · 下一动作仍 **V-VSCODE-PERF-05** · PR [#122](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/122)。

**Verification (2026-07-15 · K-SUPPLY-05 Minimal → main):** 已合 `origin/main` · merge `1857dbb5` · CI [`29391724148`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29391724148) **硬门禁 success**（`rust` windows+ubuntu / `dimension5` / `cargo-audit` / `cli` / `oocp` / `cross-host-e2e` / `stale-paths`；`frontend` / `e2e-tauri` / `loom` success）· `multiple-versions = deny` + `[bans.skip]` · ratchet **80** · **Done（Minimal · on main）** · PR [#121](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/121)。

**Verification (2026-07-15 · K-SUPPLY-05 Minimal):** HEAD `e294dc12` · CI [`29386960532`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29386960532) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`/`stale-paths`；`frontend`/`e2e-tauri`/`loom` success）· `multiple-versions = deny` + `[bans.skip]` · ratchet **80** · **Done（Minimal）** · K-VOICE-04 纠偏为 **Partial**（`voiceTtsRouting` 未合入；`useRoleVoiceProfileSync` 仍写全局配置）· PR [#121](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/121)。

**Verification (2026-07-15 · K-PLATFORM-01c → main):** 已合 `origin/main` · HEAD `30140ee2` · CI [`29362067494`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29362067494) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`/`stale-paths`；`frontend`/`e2e-tauri`/`loom` success）· CONTRIBUTING/setup 叙事 Tauri **2** / `webkit2gtk-4.1` · dimension5 `tauri major 2 + docs narrative`（**22** checks）· inventory 父可关 · **01c Done** · 父 **K-PLATFORM-01 Done** · 下一动作 **V-VSCODE-PERF-05**。

**Verification (2026-07-15 · K-PLATFORM-01b → main):** 已合 `origin/main` · HEAD `bd99175b` · CI [`29354276811`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29354276811) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`/`stale-paths`；`frontend`/`e2e-tauri`/`loom` success）· inventory §6 Frontend IPC · 生产残留 **0** · preview `send-message` 经 `frontend` job · **01b Done** · 父当时 **OPEN**（已由 01c 关闭）。

**Verification (2026-07-15 · K-PLATFORM-01a Full → main):** 已合 `origin/main` · HEAD `751a8319` · CI [`29349270841`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29349270841) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`/`stale-paths`；`e2e-tauri`/`frontend`/`loom` success）· `LAYERING_BASELINE.json` `cargo_duplicate_groups` **92→80**（`cargo tree -d` 实测；Tauri2 副产品）· **01a Done** · 父 **OPEN** · 下一动作 **01b** · K-SUPPLY-05 **仍 Partial**。

**Verification (2026-07-14 · K-PLATFORM-01a Full):** HEAD `3c08cb5e` · CI [`29344170555`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29344170555) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`/`stale-paths`；`e2e-tauri`/`frontend`/`loom` success）· inventory [`distros/TAURI_V2_MIGRATION_INVENTORY.md`](./distros/TAURI_V2_MIGRATION_INVENTORY.md) Full · **npm 最小齐步 ≠ 01b** · 父 **K-PLATFORM-01 仍 OPEN**（已由上条覆盖「合入 main」）。

**Verification (2026-07-14 · K-PLATFORM-01a Partial):** HEAD `031fc0e6` · CI [`29334069010`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29334069010) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`/`stale-paths`；`e2e-tauri` success）· inventory Partial · **零** Tauri bump（已被 Full 取代）。

**Verification (2026-07-14 · K-LLM-01b / K-LLM-01 Done):** HEAD `3b6e2a5e` · CI [`29328015057`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29328015057) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`）· 选型 SSOT [REMOTE_PLUGIN_PROTOCOL.md §2.0](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)「本机 OpenAI 兼容第二本地」· registry 测 [`openai_compatible_llm_http_roundtrip.rs`](../distros/desktop-tauri/tests/openai_compatible_llm_http_roundtrip.rs) `openai_compatible_llm_via_registry_remote`。

**Verification (2026-07-14 · K-LLM-01a Done):** HEAD `16781309` · CI [`29323544103`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29323544103) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`）· env SSOT [REMOTE_PLUGIN_PROTOCOL.md §2.0](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) · mock HTTP [`openai_compatible_llm_http_roundtrip.rs`](../distros/desktop-tauri/tests/openai_compatible_llm_http_roundtrip.rs)。

**Verification (2026-07-14 · K-SUPPLY-05 leaf dedup):** local ratchet **92**（dialoguer **0.12** · proptest **&lt;1.7**）· HEAD `68befa20` · CI run [`29314455743`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29314455743) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`）· `deny.toml` multiple-versions **仍 warn** → K-SUPPLY-05 **仍 Partial**（不升 Done）。

**Verification (2026-07-14 · K-SUPPLY-05 dedup wave):** local ratchet **97** · HEAD `07fc5573` · CI run [`29271807644`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29271807644) **硬门禁 success**（`rust`/`dimension5`/`cargo-audit`/`cli`/`oocp`/`cross-host-e2e`）· `e2e-tauri` **soft failure**（仍 Partial）· `deny.toml` multiple-versions **仍 warn** → K-SUPPLY-05 **仍 Partial**。

**Verification (2026-07-14 · D-I18N-02 Done):** HEAD `0be7f2df` · CI [`29278403237`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29278403237) · `dimension5-acceptance` **success** · `--warn-drift-high-traffic` 独立硬门禁 · HIGH_TRAFFIC **8** creator-docs · `--self-test` · EN CLI/`COMPATIBILITY`/`PLUGIN_V1`/`ROLE_PACK_SPEC` condensed+ZH 锚点 · 同跑 `e2e-tauri` 曾 soft fail（卡在 `POST /session`）。

**Verification (2026-07-14 · CI-E2E-TAURI-01 Done):** HEAD `8988f49d` · CI [`29311703046`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29311703046) · `e2e-tauri` **success**（session 已创建 · `getTitle`=`OCLIVE — Desktop AI companion` · `.left-pane` / role select `waitForDisplayed`）· 根因：CRLF shell → dual WebKit `:4445` bind → 空 dbus → 过短 `connectionRetryTimeout`；仍为 soft gate（`continue-on-error`），未升硬门禁。

---

## §1 活跃台账（OPEN · 开工清单）

| ID | 项 | 优先级 | 解冻/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **P0-STRANGER** | Theater 朋友 cohort 试玩（10 人） | **P0** | ≥60% 通过 · [`theater/PLAYTEST_MATRIX.md`](./theater/PLAYTEST_MATRIX.md) | **Done**（朋友 cohort 7/10 · 2026-06-25） |
| **K-DOC-17** | 注释英文化 batch 3 | P1 | `slot_runner.rs` · `kernel_strategy.rs` 等 | **Done**（轮次 16 复核：上述文件已为英文 `//!`/`///`） |
| **V-VSCODE-PERF-05** | VS Code F5 / `.vsix` 实机 | P1 | 姊妹仓 `oclive-vscode` 人工排期 | **OPEN**（cross-repo） |
| **K-CONTRACT-WIRING-01** | `extra_sections` 生产接线 | P2 | 首个外部插件作者 or Phase 5 通过后 | **Done**（`config.json` → `prompt_extra_sections` · `co_present` 2026-07-10） |
| **D-DOCDRIFT-01** | 重组后 normative 文档路径漂移（旧布局引用） | P0 | `check-stale-paths` 硬门禁绿 + `migrate-doc-paths` 路径存在性全过 | **Done**（轮次 17） |
| **D-SCRIPT-02** | `check-stale-paths.mjs` 误报/漏报（反例说明与行内路径） | P1 | 扩范围 + 修 pattern + 挂 dimension5 | **Done**（轮次 17） |
| **D-ORPHAN-04** | 残留空目录 `kernel/crates/models/` | P2 | 目录删除 + workspace 无引用 | **Done**（轮次 17） |
| **O-1** | `oclive_kernel_host` 编译期 `include_str!` 耦合 `distros/desktop-tauri/assets/plugin-bridge.iife.js` | P1 | 资产迁入 `kernel/crates/oclive_kernel_host/assets/` + copy 脚本改指向 | **Done**（轮次 18） |
| **O-2** | expert 孤儿前端（Vue/lib/test/i18n/API re-export，零 import） | P2 | 删除 + `role.ts`/locales 同步 + stale 文档措辞 | **Done**（轮次 18） |
| **D-DOC-RELOC-01** | 三份名实不符文档仍在 `creator-docs/`（VS Code 契约 / Studio 指南 / mumu 验收） | P2 | 物理迁至 `handoff/{vscode,studio,distros}/` + 原位 stub + 入链更新 | **Done**（轮次 18） |
| **K-SUPPLY-02** | Release 预编译内核 **SHA256SUMS**（防换包） | P1 | workflow + `bundle-kernel-for-tauri.mjs` 钩子已入库；tag `oclivenewnew-v*` 触发 CI artifact | **Done**（轮次 22） |
| **K-SUPPLY-03** | 插件安装后「请审本地源码」固定提示 | P2 | 市场/git/zip + CLI | **Done**（轮次 19） |
| **K-SUPPLY-04** | 前端 `npm-audit` 仅可见性（`continue-on-error`） | P2 | 连续 2 个复核周期生产依赖零漏洞，或出现高危时升格硬门禁 / 文档豁免 | **Done · remote verified**（2026-08-01：两轮生产扫描均为 **0 vulnerabilities**，CI `continue-on-error` 已移除并升为高危硬门禁；远端 [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) `npm-audit` 通过。完整 dev graph 风险不并入本结论，见 K-SUPPLY-12） |
| **K-SUPPLY-05** | `deny.toml` `multiple-versions` warn→deny | P2 | Minimal：`deny` + 有理由 `[bans.skip]`；Full 零 skip 另战役 | **Done**（Minimal · 2026-07-15）· **Full Partial**（2026-07-16 · workspace `toml` 0.8→1 · ratchet **75**；`[bans.skip]` 仍须保留 · 不准假 Full Done · PR [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126)） |
| **K-SECRET-01** | 已跟踪 N1N API 密钥进入 Git 历史 | **P0** | 服务端撤销旧密钥；新密钥只进 Continue secrets；维护者已明确接受保留历史的残余可见性 | **Done · owner confirmed**（2026-07-17：维护者确认已在 N1N 提供商侧彻底销毁旧密钥；Git 历史按决定保留） |
| **K-PLUGIN-SEC-01** | 目录插件 UI 信任边界：同进程 Vue / 共享 custom-protocol origin | **P1** | Minimal：发行构建禁 inline Vue；Full：每插件独立 origin 或等价强隔离 + 原生 E2E + 可信签名绑定 + 官方 HTML fallback 功能对等，并将旧 SFC loader 移出发行依赖图 | **Partial**（Stage 0–3 已实现：发行禁 inline Vue；embedded / full-shell 使用 opaque iframe sandbox + parent broker；能力令牌绑定插件并在导航时撤销；Voice HTML fallback 已补功能对等；K-SUPPLY-12 已移除 `vue3-sfc-loader`，替换路径只在显式不安全 DEV 模式动态加载且仅允许 `vue` import。冻结实现的通用主 CI 与 Ubuntu Nightly 原生窗口 smoke 已远端通过，但这不等于完整隔离证明；仍缺 Windows `tauri-driver` 原生实跑证据与可信签名/身份绑定，见 [`K-PLUGIN-SEC-01` 计划](./debt-marathon/long-plans/K-PLUGIN-SEC-01.md)） |
| **D-QUALITY-LINT-01** | 根 lint 曾漏扫 Theater / Playwright / 配置文件，且未进入 `check` / `check:release`，长期积压可自动修复与少量死代码 | P1 | 全维护面 lint 绿；生成器与漂移门禁兼容；日常 / 发版检查强制执行 | **Done**（2026-07-18：扩展 lint 范围并接入两级门禁；移除未使用聊天重建函数与无效局部变量；修复错误码生成器单引号输出和漂移解析兼容） |
| **K-I18N-HTML-01** | 静态本地化文案含 `<strong>` / `<code>` 并经受控 `v-html` 渲染，Vitest 全键翻译测试持续输出 vue-i18n HTML 警告 | P2 | 将富文本拆为组件插槽，或建立只允许静态受信 key 的集中 allowlist + 注入拒绝测试；不得直接全局关闭 HTML 警告掩盖新增入口 | **Done**（2026-07-18 本地实现；后续远端 CI [`30683169339`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30683169339) 在包含该实现的 HEAD 上 **22/22** jobs success） |
| **CI-E2E-TAURI-01** | e2e-tauri WebDriver smoke（sidebar/title） | P2 | 远程 `e2e-tauri` 过；勿弱化断言 | **Done**（2026-07-14 · CI [`29311703046`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29311703046) / HEAD `8988f49d` · session+title+sidebar 全绿；仍 soft gate） |
| **K-CHATPRO-01** | Chat Pro 流式取消 UX | P2 | `AbortController` 打断上一轮 + 清理 `streaming` 气泡；设置可关流式 | **Done**（Chat Pro 正式启用 · 2026-06-26） |
| **D-ORDER-01** | monorepo `roles` 路径 SSOT（27 集成测 + oclive-cli `join("roles")`） | P0 | `chat_pro_roles_dir()` / `tests/common` / `resolve_project_roles_dir()` | **Done**（条理优化 Wave A · 2026-06-24） |
| **D-ORDER-02** | `roles_dir.rs` debug 回退、`test_oocp.rs` 旧 `src-tauri` 路径 | P0 | 指向 `distros/chat-pro/roles` + `distros/desktop-tauri/Cargo.toml` | **Done**（Wave A） |
| **D-ORDER-03** | CI `cd fuzz`、Playwright `testDir`、`check:license` 插件路径、examples `../../roles` | P1 | 与 monorepo 布局一致 | **Done**（Wave B1/A5） |
| **D-ORDER-04** | `check-stale-paths` 仅扫 `.md` | P1 | 扩展 `.rs/.mjs/.sh/.yml` + dimension5 代码 ratchet | **Done**（Wave B2/B4） |
| **D-DOC-DRIFT-02** | AI 入口文档（rules/AGENTS/THREE_DISTRO/invoke 条数） | P1 | 与 BUS_FACTOR / INVOKE_HOTPATH_MATRIX 对齐 | **Done**（Wave C · 2026-06-24） |
| **D-DOC-DRIFT-03** | `KNOWN_VULNERABILITIES` quinn-proto 0.11.15 | P2 | 台账 + 扫描日期 | **Done**（Wave C4） |
| **D-AI-VERIFY-01** | AI 审查/汇报无核实纪律 → 误报入账 | P1 | [`AI_VERIFICATION_PROTOCOL.md`](./AI_VERIFICATION_PROTOCOL.md) + AGENTS/BOUNDARIES/Playbook 挂链 | **Done**（轮次 20） |
| **H-DOC-01** | human-docs-en 缺 L5/L6 英文摘要（06/07 仍链中文） | P2 | 补 `human-docs-en/06` 摘要或链 MODULE_MAP EN 段 | **Done** |
| **H-DOC-02** | 文档进度双轨（human README vs TECHNICAL_DEBT）须同轮更新 | P2 | 改 MODULE_MAP/架构时同步 human-docs README §进度日期 | **Done**（2026-06-26 · 文首徽标已落地） |
| **H-DOC-04** | 人类模块化开工包 `human-docs/modules/` | P2 | 全类 module pack + EN 选择器摘要 | **Done**（2026-06-26） |
| **D-MAINT-01** | 远程 dependabot 陈旧分支（实测 **39**，**9** 含 `src-tauri`） | P2 | `gh api` 列表 + 批量 `git push origin --delete` | **Done**（轮次 22 · 维护者确认后清理） |
| **BUILD-TAURI-01** | `tauri.conf.json` `beforeBuildCommand` 误写 `../../scripts` | **P0** | 改为 `node scripts/tauri-run.cjs` + dimension5 ratchet | **Done**（轮次 22） |
| **BUILD-TAURI-02** | `tauri.conf` roles 污染（`resources/roles` 误提交 / shell-dist 累积） | **P0** | canonical 单条 `../chat-pro/roles` + shell-dist 去重 + tauri-run restore + dimension5 ratchet | **Done**（2026-06-29） |
| **D-DOC-EN-01** | `creator-docs-en/security/KNOWN_VULNERABILITIES.md` 扫描日期滞后中文 | P2 | 对齐 `creator-docs/security/` 日期与命中条数 | **Done**（2026-07-13 · warning **4** 含 `anyhow` · 三条 2026-07  advisory 对齐） |
| **D-ORDER-05** | `desktop-tauri/src/lib.rs` L203 仍写 `src-tauri/src/api/` | P2 | 改注释为 `distros/desktop-tauri/src/api/`；评估移出 stale-path 豁免 | **Done**（Wave 1） |
| **D-ORDER-06** | `distributions/vscode/out/` 与 `distros/` 命名并存 | P3 | gitignore 或删除构建产物 | **Done**（根 `.gitignore` 已含 `distributions/`） |
| **D-AI-VERIFY-02** | AGENTS 测试段链 `AI_VERIFICATION_PROTOCOL` + `check:rust` vs `check:release` doctest | P2 | AGENTS §测试体系 | **Done**（Wave 1） |
| **D-DOC-LINK-01** | 活跃文档链到已迁 `handoff/archive/` 的 closure / 发版清单（根路径断链） | P1 | `check-stale-paths` archive ratchet + dimension5 十八检口径 + BOUNDARIES 文档纪律节 | **Done**（2026-06-29） |
| **K-CI-01** | GitHub CI main 红：doctest 漂移 | **P0** | 修 doctest；`cargo test --workspace` 绿 | **Done**（Wave 0 · doctest 三处） |
| **D-READ-05** | `backend_registry.rs` 拆 `directory_slots` | P2 | 零语义变更 | **Done**（Wave 4 · `directory_slots_impl.rs`） |
| **D-PORT-02** | `PluginBackendRegistryPort` 拆窄 trait | P1 | `MemoryBackendPort` phase 1 | **Done**（`memory_backend_port.rs` + `SlotBackendFactoryPort` 组合 · 2026-07-10） |
| **D-SLOT-01** | BuiltinV1/V2 选择收敛到 resolver | P2 | 四槽不再保留双实现；`builtin_v2` 仅作读兼容 alias 并统一归一为 `builtin` | **Done**（现行源码 + `plugin_backends_v2_resolve` / `slot_resolver_v3`；已由 CI [`29465172205`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29465172205) 覆盖） |
| **D-TRAIT-01** | 单实现 trait 合并 | P3 | 仅明显 DI 噪音 | **Observe** |
| **K-VOICE-01** | CosyVoice2 `stream=True` 在部分 Windows/模型组合曾因上游 worker 异常无限等待 | P2 | 继续实机 soak；worker 必须在异常时结束等待，安全完整短句 prime 后才走真实流式；`OCLIVE_COSYVOICE_STREAM=0` 保留诊断回退 | **Implemented，继续 Observe**（2026-07-24：默认真实 PCM 流式 + worker 结束态保护 + 安全 prime；CPU 分阶段混合 FP16 冷加载 + driver-wide VRAM admission，失败回退 FP32 也须再次准入。2026-08-01 五分钟共享 GPU 复测完成 46 对生成、318 次 GPU 采样，采样线程和 LLM/Voice 子进程均回收，未再出现无限等待；但时长仍不足以替代长时间硬件 soak，尾延迟另见 K-VOICE-09。排查见 [`TRACK_VOICE_RECOGNITION.md`](../human-docs/team/TRACK_VOICE_RECOGNITION.md) §10） |
| **K-VOICE-02** | Tier-2 TTS（ChatTTS · XTTS · Bark · VITS · 国内云 API · Piper 产品化） | P3 | VX-9 generic pack 模板或社区 adapter | **OPEN** |
| **K-VOICE-03** | Linux/macOS CosyVoice2 产品 profile | P2 | 随上游 CosyVoice 跨平台稳定后解冻 `asr_profiles.json` platforms | **OPEN** |
| **K-VOICE-04** | 角色包可选 TTS 覆盖与全局配置冲突 | P3 | 全局 profile 必须服务所有角色；角色 `synth_profile` 只覆盖播报任务，不得在切换角色时改写设置 | **Done**（Minimal · VX-11 · `2d5123af` · CI [`29408795870`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29408795870) · PR [#122](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/122)；inherit-provider 跟随 [#123](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/123) → `262c9ac4`） |
| **K-VOICE-05** | Qwen3-TTS 官方 REST 契约稳定化（社区 server 方言多） | P2 | 随上游收敛后收紧 adapter；Fish 默认端口已改 **9881** 避免与 Qwen **8080** 冲突 | **Observe** |
| **K-VOICE-06** | 社区 directory 插件 `com.user.tts.*`（自带 sidecar/RPC） | P2 | VX-10 · `plugin_rpc_invoke` 白名单 | **Done**（Minimal · 2026-07-16 · `b8cb0c48` · CI [`29465172205`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29465172205) · PR [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126)） |
| **K-VOICE-07** | `voice_directive` v2 + `engine_extras` 透传 bag | P2 | RFC §4.1 小节后实现 | **OPEN** |
| **K-VOICE-08** | 全引擎统一流式 playback contract | P2 | 非 CosyVoice chunked audio 抽象 | **Deferred** |
| **K-VOICE-09** | 共享 GPU 下 CosyVoice2 语音 TTFC 长尾仍会越过 8 秒门禁 | **P2** | 分离 warm、首个热请求与稳态片段，剖析文本前端/推理/首块编码；保留 8 秒绝对门禁并同时跟踪 p50/p95/max，不以放宽阈值或降低显存安全线掩盖；优化后复跑至少 30 分钟实机矩阵 | **In progress · bounded first-chunk policy implemented**（2026-08-02：分段计时与跨请求 hop 恢复已落地；维护者确认首声优先后，有限策略只对不少于 10 字符的首段采用高优先级 CUDA streams + 20-token 初始 hop，短句保持上游 25-token 路径。长段 10 组 TTFC p50/p95(max) **4135/4473→3643/3826ms**，峰值余量 **1394MiB**、稳态增长 **0MiB**、进程全回收；4 字短句 10/10 保持单块路径。CUDA-only、降低 llama 层数/进程优先级/poll 与 8-token 短句档均无可靠收益或构成负优化，未作为默认捷径。8 秒门禁不变；仍欠 **30 分钟真实矩阵 + 用户听感验收**，历史五分钟 46 对的 **9514ms** 最大值继续作为未清尾延迟证据；详见 [`TTFT_BENCHMARK.md`](./TTFT_BENCHMARK.md)） |
| **K-DISTRO-01** | 内核打包资源化（B 组暂存）：`kernel_lifecycle/spawn.rs` 内核旁迁移目录发现 + spawn env 注入 + Windows PATH 大小写修复 + `bundle-kernel-for-tauri.mjs` / `e2e-tauri-bundled-kernel.mjs` / `with-windows-rc-path.mjs` 扩展 + `sql_migrate.rs` 迁移源 + `.gitignore` | **P1** | 解冻 stash 后按开发流水线收口并整体提交；`distros/desktop-tauri/resources/migrations/` 为 bundle 可再生产物，恢复时重新生成即可 | **OPEN · stashed**（2026-08-11：维护者决策暂存；B/C 组修改在 `stash@{1}`，migrations 产物在 `stash@{0}`，恢复时 `git stash pop` 两次，先 pop `stash@{0}`） |
| **K-VOICE-10** | 语音/门禁脚本未定稿（C 组暂存）：`check-voice-tts-ratchet.mjs` / `stress-voice-gpu-runtime.py` / `dimension5-acceptance.mjs` / `compilePluginVueSfc.test.ts` 调整 | **P2** | 解冻 stash 后按开发流水线收口：评审语音 ratchet、GPU stress 与 dimension5 调整，与 K-VOICE-09 的 30 分钟矩阵/听感验收节奏对齐 | **OPEN · stashed**（2026-08-11：维护者决策暂存，与 B 组同在 `stash@{1}`） |
| **K-EMO-01** | 英文词表扩充通道暂放（VADER MIT / AFINN Apache-2.0 参考），中文完成后处理 | P3 | 中文词表完成后扩充英文词表 | **OPEN**（2026-08-11 已登记） |
| **K-EMO-02** | ???? A ??????? JSON ? + ?????`bb70cbbd` ????B M1 ????[EMO] ?? + ? policy hold + ? post ?? + ??????? 0?4 `60d90d5b`?`3cb6806b`? | P2 | ???????????????max ? `low_confidence_hold_threshold` ??????`neutral_hold` ?? M1 ??????????/? Ollama ?????????????????????????????????? | **???**?2026-08-12??? 0?4 ?????LLM ?????? `main_llm_failure_returns_fallback_reply_and_keeps_previous_emotion` ?? ? host lib 454/454 ? clippy/fmt ???**??? playtest ??**??`chat_messages` turn 23/24 `bot_emotion=happy`?`emotion_source=llm`?`emotion_labels=[joy,surprise]` ?? + narrative hint ???**???????/? Ollama???????? M2 ????**? |
| **K-EMO-03** | `BuiltinKeywordComplexEmotionProvider` ??? remote/directory ??????? fast ????????`complex_emotion.rs:36` ?? / `:76` impl?`backend_registry.rs:61` / `co_present.rs:34` / `remote_plugin` adapter ?????**????**?v1.8 ???????? | P3 | ?????????????????????????????????? | **OPEN**?2026-08-12 ???????? |
| **K-EMO-04** | ????/?????? `distros/desktop-tauri/src/desktop_host.rs:30` `bundled_kernel_binary` ? join `resource_dir` ??Tauri dev ?????? `resources/` ??? ? dev ?? bundled ????????2026-08-12 ???spawn ?? shared/dev??? manifest `builtAt` ?????????? `OCLIVE_KERNEL_BUILT_AT` / `OCLIVE_KERNEL_GIT_COMMIT`??`should_promote_binary` ????????? 15:20 release/bundled ??????? 3 ???15:20:55????? emotion_source ?? | **P2** | ? dev ???? `resources/` ????? ?????? builtAt/gitCommit??? `KernelBinaryManifest::from_compile_time_env`??? ???? release ???`3cb6806b`????? runtime + bundled ?????? `--version` ???? | **OPEN**?2026-08-12 ?????? M2 ????? |



---

## 前瞻性结构风险（2026-07-12 审查增补 · 2026-07-13 拆项）

| ID | 项 | 优先级 | 解决/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **K-PLATFORM-01** | Tauri v1 → v2 迁移 | **P1** | **契约**：权限/capability schema 对照 + **测试**：最小 smoke + **改动面**：desktop-tauri / 三 distro 分 PR | **Done**（01a+01b+01c · 2026-07-15 · CI [`29362067494`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29362067494)） |
| **K-LLM-01** | LLM 后端单一依赖 Ollama | **P1** | **契约**：`LlmBackend` env 矩阵 + **测试**：1 API + 1 本地 mock + **改动面**：adapter 接线 PR | **Done**（01a+01b · 2026-07-14；第二本地 = Remote+localhost OpenAI-compat ∥ directory/llamacpp；无新 enum） |
| **K-CROSS-01** | 跨平台系统策略缺失 | **P2** | 三平台语音 smoke + distro profile 差异声明 | **Partial**（Minimal 文档 · 2026-07-16 · DISTRO+TRACK 平台矩阵 · 缺三平台实机 smoke · PR [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126)） |
| **K-DIST-01** | 分发体验缺口 | **P2** | 签名/updater/Linux 包/macOS dmg | **Partial**（Minimal 文档 · 2026-07-16 · `handoff/distros/README.md` gaps · Full `blocked:needs-signing-secrets` · PR [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126)） |
| **D-I18N-02** | creator-docs-en 镜像滞后 | **P2** | **契约**：`check-doc-mirror` 扩展 + **测试**：mirror ratchet 样例 + **改动面**：creator-docs-en 补链 PR | **Done**（2026-07-14 · HEAD `0be7f2df` · CI [`29278403237`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29278403237) dim5 success · HIGH_TRAFFIC×8 + `--self-test`） |
| **V-MARKET-01** | 插件市场生态 | **P2** | 市场 UI + 社区插件 | **Partial**（Minimal SCOPE · 2026-07-16 · `PRODUCT_LINE_TASK_BUCKETS` · 姊妹仓 human/cross-repo · ≠ Full Done · PR [#126](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/126)） |
| **V-MODULE-QUALITY-01** | “上限 = 模块上限”缺少组件质量对比 harness | **P1** | 固定角色/场景/replay 输入；可复现比较 memory / emotion / prompt / LLM 模块的质量与行为指标，而非只比延迟 | **OPEN**（[Full 计划](./debt-marathon/long-plans/V-MODULE-QUALITY-01.md) · Stage 1 **Locally verified** · `675561d5`：版本化固定用例、严格观察合同、四维离线评分与回归自测已落地；Stage 2 待接既有内核采集链，未升 Done） |
| **V-PORTABLE-01** | 同一角色包跨发行版通用的分层验收 | **P1** | Minimal：同包通过 desktop / vscode / theater profile 的 load + mock chat；Full：各发行版 UI、资产、独立通道与降级差异形成 capability-conformance 报告 | **Partial**（Minimal implemented · 2026-07-18 · `e2e-distro-kernel --scenario role-portability` 本地三 profile 全绿；Full 未完成） |
| **V-EMBED-01** | 跨平台 / 硬件的完整内核嵌入证明 | **P2** | host-independent `library` 提供完整 `process_message` 对称 API；至少一个 Linux/ARM 或硬件网关真实靶验证角色包、持久化、插件与资源预算 | **Partial**（无头 HTTP / CLI / ARM64 cross 已有；纯 library 仅暴露 runtime/DTO，完整编排仍在 host；参考 §2 冻结项与 K-CROSS-01） |
| **D-BLUEPRINT-CONTRACT-02** | JSON Schema、Rust serde、`includes` 与插件自动挂载口径漂移 | **P1** | 未知字段策略一致；v2 `policy` / v3 `zone` 契约对齐；include mode/缺失行为与文档一致；自动挂载 backend 通过最终蓝图校验 | **Done**（2026-07-28 · 主仓实现 `f9d0a701`，验证收口 HEAD `43db1d20`，CI [`30373054084`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30373054084) `21/21` jobs success；编写器实现 `5a88227`，验证收口 HEAD `0d8164c5`，CI [`30364549132`](https://github.com/linkaiheng2233-cyber/oclive-pack-editor/actions/runs/30364549132) success；本地 Rust workspace fmt/clippy/lib/doctest、validation `92 + 18 + 4`、编写器 `38 files / 166 tests`、官方 `8` 角色包、module-compat 与 Dimension 5 `26 checks` 通过） |
| **V-BLUEPRINT-EXT-01** | 第三方能力会推动蓝图根 schema 无界增长 | **P1** | Stable v4 实现最小 `extensions` 外壳、命名空间、required/optional、安全 `config_ref`、未知可选 round-trip、Capability Registry 与跨发行版诊断 | **Done**（2026-07-29：外壳/round-trip 基础之上，目录 Provider Registry、确定性解析、权限/依赖/启停诊断、required 激活门禁、Tauri/CLI 只读计划及同包跨发行版差异测试已落地；`ExecutionPlan` 不启动 Provider、不写回角色包；主仓实现 `56ad4f5f`、SSOT 收口 `43a51afc`，CI [`30421846109`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30421846109) `21/21` jobs success；[RFC](../creator-docs/rfc/RFC_BLUEPRINT_EXTENSION_AND_RESOURCE_COORDINATION.md)） |
| **K-RESOURCE-COORD-01** | llama-server / Ollama / CosyVoice 与未来 Live2D 各自决策资源，缺少统一预算和租约 | **P1** | 内部 `ExecutionPlan` + Resource Coordinator；LLM/Voice 两适配器先闭环 snapshot/admission/lease/pressure/degrade/release；覆盖不可控外部进程、取消、故障恢复与真实共享显存压力测，再接第三类渲染适配器 | **Partial · Stage 3 verified**（2026-07-31：资源诊断升至 v5，快照与租约统一覆盖 NVIDIA GPU、系统 RAM 和 CPU；公平准入队列实现优先级、等待老化防饥饿、超时与取消安全清理。Performance llama 新增 `gpu_full` / `gpu_balanced` / `cpu_compatibility` 三个真实档位，实际改变 `llama-server --n-gpu-layers` 并按准入结果降级；外部 Ollama 仍保持 observe-only。自动抢占只选择低优先级、明确声明可逆动作并具有精确 requester → target → operation 授权的 managed 适配器，失败逆序回滚，成功调用结束后逆序恢复。新增 owner namespace 约束的进程内 `ResourceAdapterRegistrar`，可登记第三方 adapter facts 与单写者 controller，但不解释目录 manifest、不自动授权跨适配器控制。契约加入 `render` / `compute` / `hybrid`，第三方 Render 容量与抢占恢复测试验证控制面没有写死 LLM/Voice；实际 bundled Live2D runtime 仍未交付。当前证据：资源定点 **59/59**、Performance LLM **16/16**、受影响 Rust 库 **634/634**、doctest **6/6**、前端单测 **269/269**；`check:release` 与串行构建版 `check:ci-local` 均退出码 **0**，Dimension 5 **26/26**，workspace Clippy、格式、ESLint、生产构建、locked 集成测试、模块兼容 **10 slots / 9 manifests / 7 UI contributions**、中英文镜像与 changelog parity 均通过。`cargo audit` 漏洞级 **0**、已允许警告 **8**；`cargo deny check` 仍因既有 Tauri/Linux GTK3 与 `unic-*` unmaintained advisories 失败，非本轮新增依赖。PR [#147](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/147) 首轮远端主 CI [30621565681](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30621565681) 与严格审计 [30621565688](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30621565688) 在实现提交 `42f3ff7f` 上 **22/22** 通过。既有 RTX 5060 Laptop 8GB direct runtime 共存 10 轮峰值 **6759/8151MiB**、最小余量 **1392MiB**、稳态增长 **17MiB**，详见 [`TTFT_BENCHMARK.md`](./TTFT_BENCHMARK.md)。仍缺目录 manifest 资源声明/自动装配、实际 bundled Live2D Provider/runtime、不可控外部进程完整故障矩阵与长时间真实进程/硬件 soak；未升 Done） |
| **D-SCAFFOLD-RESOURCE-01** | `oclive-cli` 脚手架尚无统一资源调度的开发者配置与诊断界面 | **P1** | 在独立轮次为脚手架增加策略/有限约束编辑、适配器档位浏览、候选计划预览、原因码解释与硬件建议；必须复用 `oclive_kernel_types` / Host Plan Compiler，不得另造资源 schema 或第二套解析器 | **OPEN · maintainer scheduled**（2026-07-31：维护者明确要求作为关键开发工具独立更新；本轮只入账，不与 K-RESOURCE-COORD-01 的内核计划/执行器实现混改。验收至少覆盖交互与非交互输出、生成项目 round-trip、旧 profile 兼容、无 GPU/observe-only/冲突计划诊断及 CLI 文档镜像） |
| **K-CI-IMPACT-01** | 单一全量 CI 不理解 OCLive 模块语义，重复构建且无法为新增模块解释验证范围 | **P1** | Stage 1：版本化模块描述、中央影响图与受信验证目录；Rust 确定性规划器提供 `plan/explain`、未知范围全量回退；GitHub 影子 job 只产报告、不跳主 CI 硬门禁。Stage 2 以实际结果度量漏选/过选；模拟语料只做规则回归；证据成熟后才允许低风险 PR 选择性执行 | **In progress · Stage 1 Shadow + real Compare samples 2 + simulation corpus**（2026-08-01：Stage 1 基线远端 [`30656699601`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30656699601) 通过；首个真实对照 [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) 完成 **21/21** job，两轮前置失败均落在已选 `frontend`，本样本无已知漏选但类别仍单一。2026-08-02 第二个真实对照 [`30708363964`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30708363964) 绑定语音分段提交 `6164d6eb`，完成 **16/16** job；targeted 计划选择 11 个 validator、10 类 workflow job（Rust/前端矩阵展开后 **12** 个实际验证 job），未选择的 `cli`、`kernel-host-dual-core`、`rust-arm64-cross` 也全部通过，本样本未观察到漏选；全量墙钟约 **48 分 47 秒**，说明选择性执行存在实际收益，但两类样本仍不足以改变门禁。新增严格 11 场景模拟语料：**8 targeted / 3 fail-safe** 全部匹配；docs 选 2、scaffold 选 4、kernel Nightly 选 18 个 validator。shared/role/plugin 均选 8 个 validator（含 Rust），记录为前端影响环的过选候选，不凭模拟删边。本轮远端 `plan.json` SHA-256 `C7A644497389ECC0461AAB4F57E8F323422CE1A43C18BAD08BAB83A1B8D14BE2`；本机模拟 `shadow-samples.evidence.json` 绑定 source commit `46b323d1`，SHA-256 `A786CA825B084DDBC2F6C3497C1244D69F26BF3412C8B60D1100DE890C7C07A7`，明确标记 `authoritative_ci_comparison=false`。脚手架继续不拥有工作流、验证器、Runner、Secret、门禁或第二套解析器。实施 SSOT：[`SOMEDAY_TOOLCHAIN_CI.md`](../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)） |
| **D-CI-AI-REVIEW-03** | CI 尚无代码特化审查专家模型辅助发现模块元数据遗漏、跨模块语义影响与契约/测试漂移 | **P2** | 先从影子规划器积累版本化数据集：diff、模块归属、`plan/explain`、全量 CI 结果、漏选/过选、人工裁决与对应修复；训练集与冻结评测集须按提交/事件隔离，避免泄漏。数据质量达标后再基准评估约 **1–2B** 的本地小模型候选，要求结构化输出证据、置信度、受影响模块与建议验证器。模型只可告警或建议**扩大**验证范围，不得跳过/降级确定性门禁、改写受信验证目录，亦不得控制 Runner、Secret 或流水线编排 | **OPEN · data collection first**（2026-08-01：先复用 K-CI-IMPACT-01 Shadow/Compare 证据，不立即训练或接入 CI。是否采用 1–2B 取决于独立评测中的漏影响召回率、误报率、延迟、内存占用和可解释证据质量；确定性规划器与固定安全下限继续拥有最终门禁权） |
| **D-SCAFFOLD-CONTRACT-02** | 官方配方、旧 template 归档和自定义脚手架缺少统一发现、命名空间、来源锁定与兼容边界 | **P1** | Stage 2A：Scaffold Package v1 schema；project/user/official 可配置发现；第三方命令仅声明且必须 namespace；`ci.*` 硬拒绝；确定性 lock/source record；组合字段只预留 | **Done · Stage 2A implemented**（2026-08-01：完成 25 个旧顶层命令与 5 条生成路径审查；新增严格 schema、四个编译内置官方兜底包、本地发现/配置覆盖/版本拒绝/路径逃逸检查/确定性 SHA 与原子 lock；`oclive scaffold list/inspect/validate/resolve` 只诊断，Stage 2A 不联网、不执行第三方 entry、不解析组合；默认帮助收口为 15 个稳定入口，10 个试验命令与旧 `template` 保持可调用但隐藏。SSOT：[`RFC_SCAFFOLD_PACKAGE_V1.md`](../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)） |
| **D-SCAFFOLD-GENERATION-03** | 已发现的第三方脚手架缺少可审计、可回滚且不升级为任意代码执行的生成闭环 | **P1** | Stage 2B：摘要固定的声明式 text/copy；精确 lock + 逐次确认；只写全新目录；内存预渲染、同父目录暂存与一次落位；value-free provenance；官方 builtin 委托既有领域命令 | **Done · Stage 2B remote verified**（2026-08-01：scaffold contract 升至 1.1.0，旧 v1.0 无摘要包保持可发现并获得迁移提示；`scaffold generate` 覆盖成功、dry-run、不可信确认、缺失/陈旧 lock、指令/源文件摘要漂移、变量错误、已有输出保护和官方委托。未开放市场、联网安装、第三方 command/script/hook、组合执行或 CI 控制；远端 CI [`30683169339`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30683169339) 在实现 HEAD `7d3d1eca` 上完成并通过。SSOT：[`RFC_SCAFFOLD_PACKAGE_V1.md`](../creator-docs/rfc/RFC_SCAFFOLD_PACKAGE_V1.md)） |
| **D-SCAFFOLD-EVOLUTION-04** | Stage 2A/2B 已冻结发现与声明式生成边界，但测试期后的下一项脚手架能力尚未完成需求证据、兼容策略与信任边界决策 | **P1** | Stage 2C 先经历实际使用与问题收集，再以单独决策门冻结一个有界目标、schema/min-reader 迁移、来源/权限/沙箱、回滚与契约测试；组合/依赖解析、namespace 命令执行、离线包生命周期、资源配置诊断只能作为候选方向逐项取舍，不得合并成一次无界扩张 | **OPEN · testing/decision gate**（2026-08-01：当前进入使用观察期，不实现 Stage 2C。`D-SCAFFOLD-RESOURCE-01` 保持独立；市场、联网安装、第三方 CI 编排权、Runner/Secret/门禁控制继续是非目标，除非维护者另行授权并冻结新契约） |
| **K-LLM-ENV-02** | `apply_user_llm_env` 在 DB snapshot 读取后才取得进程环境锁，并把调用结束时的最新版本直接标记为已应用；并发旧调用可能覆盖新环境却清除 dirty 标记 | **P1** | 串行化完整的“读设置 → token/cache → env/provider → 版本提交”事务，或改为不可变配置快照；版本变化时必须重试而非误报已应用；用可控交错测试证明 last-writer-wins，覆盖保存设置、chat、theater 与 canonical sync 调用链 | **Remote verified · stress pending**（2026-08-01：全事务改由单一异步 mutex 串行化；只提交实际读取的起始版本，期间若出现新版本则恢复 dirty，等待调用会继续应用；稳定快照/新版本保留 dirty 单测及 Host 定向测试通过，远端 [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) Linux/Windows Rust 全绿。尚缺更长时间、可控高并发交错/进程级压力证据，不提前升 Done） |
| **K-FRONTEND-TYPECHECK-01** | Vite/ESLint/Vitest 均不执行 Vue/TypeScript 类型检查，CI 可在真实分支错误存在时保持全绿 | **P1** | 引入 `vue-tsc` 与正确的 shared/chat-pro/theater project/alias 边界；先修零或建立只降不升 ratchet，再接 `check`/PR 硬门禁；为插件分享、协议安装与 Theater 大纲生成补行为测试 | **Done · remote verified**（2026-08-01：引入 `vue-tsc` 并接入本地 `check`、`check:release` 与 PR frontend job；修清 shared/Chat Pro/Theater 的真实诊断与跨发行版类型边界，补齐插件分享、协议安装复核提示和 Theater cast 行为回归。远端 [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) 的 Ubuntu/Windows frontend job 均完成 lint、typecheck、**272** 条前端单测、**53** 条 Theater 测试与构建） |
| **D-CI-EXECUTION-02** | Stage 1 影子规划器尚未减少现有全量 job，通用 Rust job 又重复 CLI 嵌套 Cargo E2E；可见性 job 与硬门禁混居主 workflow | **P1** | 先按所有权去重 workspace/CLI/audit，串行或离线化嵌套 Cargo build 并稳定缓存；再把 soft visibility job 明确迁至 nightly/dispatch 或升为硬门禁；选择性执行仍须等待 Shadow 漏选/过选证据 | **Done · remote verified**（2026-08-02：workspace/CLI/audit 唯一所有权与主 CI 硬门禁已由冻结实现 `728219e7` 的 [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) **16/16** job 验证；迁出的 `visual-presentation-smoke`、`fuzz`、`loom`、`cli-bench`、`e2e-tauri` 及汇总由完整 Nightly [`30714480898`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714480898) **6/6** job 验证，失败不再被吞掉。去重前后既有远端总 job-seconds 为 **10491→8586（−18.2%）**；Loom 运行包级 `loom-tests` 的两个有界真模型，`ci init` 不再生成主仓专属 Loom 路径，tier→workflow 归属有契约测试。此项关闭只代表执行所有权与 Nightly 分流完成；Stage 1 继续 Shadow，选择性执行仍归 **K-CI-IMPACT-01**） |
| **D-DEBT-LEDGER-01** | 技术债 SSOT 同时承担活跃清单、完成历史与长验证日志，重复 ID 和状态复述增加漂移概率 | P2 | 每个 ID 只保留一个权威状态行；历史验证移入归档/波次记录并以链接引用；增加重复 ID/冲突状态检查，允许显式 cross-reference 但禁止双写状态 | **OPEN · measured**（2026-08-01 本轮入债后：表格 **148** 行、**128** 个唯一 ID、**18** 个重复 ID，文首另有 **20** 段 `Verification`；本轮已发现多处“远端已过、状态仍待 CI”语义漂移） |
| **D-ASSET-FOOTPRINT-01** | 官方角色图片直接进入 Git 与发行资源，资产体积已成为仓库主要组成 | P2 | 先建立视觉质量/解码兼容基线，再按角色包格式、loader、CSP、编写器与模块兼容链评估 WebP/AVIF 或 PNG 量化；不得只改单端后缀；发版体积和冷加载有实测收益才迁移 | **OPEN · measured**（2026-08-01：tracked working tree **98.9 MiB**；角色图片 **71** 张 / **85.7 MiB**，占 **86.6%**。这是优化候选，不等于立即引入 Git LFS） |

**K-PLATFORM-01 子项**

| 子 ID | 契约 | 测试 | 改动面 | 状态 |
|-------|------|------|--------|------|
| K-PLATFORM-01a | Tauri v2 permission / ACL + bump | 单命令 invoke smoke | `tauri.conf.json` + `capabilities/` + Cargo | **Done**（2026-07-15 **已合 main** · 分支 HEAD `3c08cb5e` · CI [`29344170555`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29344170555) · inventory Full · smoke `tauri_invoke_smoke` · npm 最小齐步 ≠ 01b） |
| K-PLATFORM-01b | `@tauri-apps/api` v2 迁移表 | chat-pro preview 发消息 E2E（`frontend` job） | `distros/shared` IPC · inventory §6 | **Done**（2026-07-15 · HEAD `bd99175b` · CI [`29354276811`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29354276811) · §6 · 生产残留 0 · `send-message` via `frontend`） |
| K-PLATFORM-01c | 人类/门禁叙事 + dimension5 口径 | CONTRIBUTING/setup 无 `webkit2gtk-4.0`；dim5 `tauri` major 2 | CONTRIBUTING · human-docs setup · `dimension5-acceptance.mjs` · inventory | **Done**（2026-07-15 · HEAD `30140ee2` · CI [`29362067494`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29362067494)） |

**K-LLM-01 子项**

| 子 ID | 契约 | 测试 | 改动面 | 状态 |
|-------|------|------|--------|------|
| K-LLM-01a | OpenAI-compatible API env SSOT（[REMOTE_PLUGIN_PROTOCOL §2.0](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md)） | [`openai_compatible_llm_http_roundtrip.rs`](../distros/desktop-tauri/tests/openai_compatible_llm_http_roundtrip.rs) | `openai_compatible_llm.rs`（本波未改生产码） | **Done**（2026-07-14 · HEAD `16781309` · CI [`29323544103`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29323544103)） |
| K-LLM-01b | 第二本地后端选型（§2.0 SSOT；无新 enum / 空壳 adapter） | [`openai_compatible_llm_via_registry_remote`](../distros/desktop-tauri/tests/openai_compatible_llm_http_roundtrip.rs)（registry Remote） | 既有 `openai_compatible_llm` + `examples/directory-plugin-llamacpp` | **Done**（2026-07-14 · HEAD `3b6e2a5e` · CI [`29328015057`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29328015057)） |

**系统性债务（2026-07-12 审查增补）**

| ID | 项 | 优先级 | 解决/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **D-ARCH-01** | 六槽解析链 SSOT | **P1** | MODULE_MAP §3.2 + 集成测试（legacy/v2 + session override + host ceiling） | **Done**（2026-07-13 · `slot_resolution_chain.rs` 四测 + MODULE_MAP §12.5） |
| **K-MEM-01** | STM→LTM 生命周期分散 | **P1** | CHAT_STORAGE 表 + 集成测试 | **Done**（2026-07-13 · `memory_lifecycle_integration.rs` 六测 · merge/strong_only/prompt 读取） |
| **K-FREEZE-01** | 冻结状态不透明 | **P1** | 技术债 §2 收敛 | **Done**（2026-07-12） |
| **K-TEST-01** | check:rust 仅 --lib | **P2** | check:rust:integration；**盲区在本地而非 CI** | **Done**（2026-07-12） |
| **K-CONFIG-01** | 配置无诊断 | **P2** | oclive-cli doctor config-resolve + `--json` + 自动测试 | **Done**（2026-07-13 · runtime `plugin_resolution` 纯路径默认 · `diagnostics-host` feature 可选 host · `cargo tree` 无 sqlite/axum/tauri） |
| **K-ERR-01** | 热路径错误码 | **P2** | 插件/manifest/迁移结构化码 + 契约测 | **Done**（2026-07-13 · `KernelErrorBody.context` · `kernelErrorCodes.ts` · dimension5 drift 门禁） |
| **D-ROLEVER-01** | 角色包版本迁移 | **P2** | ROLE_PACK_SPEC 章节 | **Done**（`6bab6276` · PR [#125](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/125) 已合入；后续 CI [`29465172205`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29465172205) success） |
| **T-DOC-02** | Theater 状态单页 | **P2** | theater STATUS | **Done**（`e541f805` · PR [#124](https://github.com/linkaiheng2233-cyber/oclivenewnew/pull/124) 已合入；CI [`29441239048`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/29441239048) success） |
| **K-RPP-01** | RPP 无契约 | **P2** | PLUGIN_V1 或 RPP_CONTRACT | **Done**（2026-07-13 · `rpp_contract_audit.rs` 四测 · manifest/RPC/roundtrip/mumu 默认关） |
| **K-RESILIENCE-01** | Remote 弹性分散 | **P2** | ResilienceLayer | **Partial**（Minimal · 2026-07-16 · PROTOCOL 锚点 + `prompt_http`→`call_with_builtin_fallback` · Full 仍 OPEN） |

## §1.5 供应链安全（Supply Chain · 2026-06-24）

**策略 SSOT**：[`creator-docs/security/SUPPLY_CHAIN.md`](../creator-docs/security/SUPPLY_CHAIN.md)

### 基线（已落地 · 非债）

| 护栏 | 说明 |
|------|------|
| `cargo audit` 0.22.1 | 主 `ci.yml` 由 Dimension 5 唯一持有；`Cargo.lock` / audit policy 另走 `cargo-audit-lockfile.yml` 硬门禁 |
| `Cargo.lock` ratchet | dimension5 禁止 `sqlx-mysql` / `rsa` |
| `KNOWN_VULNERABILITIES.md` | 漏洞级 SSOT；`Cargo.lock` PR 须滚动日期 |
| `deny.toml` + `oclive lint --deny` | 许可证允许表 · Apache-2.0 工作区 |
| 插件权限 A4 | manifest / runtime / 集成测三面一致 |
| SQL 迁移 checksum | 防迁移文件静默篡改 |

### 台账（OPEN / Observe / Deferred）

| ID | 项 | 优先级 | 状态 |
|----|-----|--------|------|
| **K-SUPPLY-01** | `cargo deny` 进 dimension5 / CI 硬门禁 | P1 | **Done**（轮次 19） |
| **K-SUPPLY-02** | Release SHA256SUMS | P1 | **Done** — `generate-sha256sums.mjs` · `release-kernel-checksums.yml` · `bundle-kernel-for-tauri.mjs` |
| **K-SUPPLY-03** | 插件安装审源码提示 | P2 | **Done**（轮次 19） |
| **K-SUPPLY-04** | npm-audit 升格策略 | P2 | **Done · remote verified** — 2026-07-18 与 2026-08-01 两轮生产依赖扫描均为 **0 vulnerabilities**；远端 [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) 硬门禁通过 |
| **K-SUPPLY-05** | deny 重复依赖 warn→deny | P2 | **Done**（Minimal · 2026-07-15）— `multiple-versions = deny` + documented `[bans.skip]` · ratchet **80** · **Full Partial**（2026-07-16 · toml workspace→1 · ratchet **75** · 零 skip 仍 blocked:needs-ecosystem） |
| **K-SUPPLY-06** | 位级可重复构建（reproducible） | — | **Deferred** · 见 SECURITY_AUDIT_SCOPE 局限 |
| **K-SUPPLY-07** | SBOM（CycloneDX/SPDX） | — | **Deferred** · 政企/校企采购需求触发 |
| **MEGA-SD-01** | `scene_director.rs` 巨无霸拆分 | 见 §2 解冻条件；零语义变更 PR |
| **MEGA-TS-01** | `useTheaterShell.ts` 巨无霸拆分 | 见 §2；`mapTheaterInvokeError` 已先行减负（轮次 22） |
| **K-SUPPLY-08** | crate 作者信誉 / 发布历史系统审计 | — | **Observe** · 无成熟自动化方案 |
| **K-SUPPLY-09** | 插件签名严格模式默认关闭 | **P1** | 官方/市场安装默认要求可验证签名；本地开发保留显式 opt-out，并补签名轮换/撤销流程 | **OPEN**（当前仅 `OCLIVE_PLUGIN_SIGNATURE_STRICT=1` 时校验 sidecar SHA-256；不能把源码提示当供应链证明） |
| **K-SUPPLY-10** | GitHub Actions 仅固定可变 tag（`@v*` / `@stable`） | P2 | 所有外部 action 固定完整 commit SHA，并由 Dependabot/Renovate 维护升级 | **OPEN**（`actions/*`、`dtolnay/rust-toolchain`、`Swatinem/rust-cache` 均未 pin SHA） |
| **K-SUPPLY-11** | `event-listener 5.4.1` 命中 RUSTSEC-2026-0221（`StackSlot` 可跨线程携带 `!Send` tag） | **P1** | 追踪 SQLx 与 zbus/Tauri 两条传递路径，优先升级到修复版本；若上游暂时阻塞，记录实际可达性与版本约束，不得仅加入 ignore；更新中英 KNOWN_VULNERABILITIES | **Done · remote verified**（2026-08-01：锁文件升级至 **5.4.2**，SQLx 与 zbus/Tauri 均解析到修复版；`cargo audit` 漏洞级 **0**、allowed warnings **9→8**；远端 [`30692428026`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30692428026) Dimension 5 与 Linux Rust 均通过） |
| **K-SUPPLY-12** | npm 开发工具链 audit 命中与 ESLint peer 契约漂移 | **P1** | 对 ESLint/`brace-expansion`、WebDriver/`fast-xml-parser`、旧 Vue/PostCSS SFC loader 逐条做可达性与升级/移除；`npm ls eslint eslint-plugin-unicorn` 退出 0，完整/生产 `npm audit` 无 high，lint/typecheck/unit/build 与 Linux/Windows CI 全绿；禁止 `--force` 或无证据 override | **Done · remote verified**（2026-08-02：ESLint **10.8.0** + Antfu **9.2.0** + Unicorn **72.0.0** peer 合法；WebDriverIO **9.30.0** 解析 fixed `fast-xml-parser` **5.10.1**；旧 `vue3-sfc-loader` / Vue 2 / PostCSS 链已由官方 compiler 的受限 DEV-only 路径替代。完整与生产 audit 均为 **0 vulnerabilities**，目录插件 SFC 回归、lint、typecheck、前端单测与生产 build 本地通过；冻结实现 `728219e7` 的远端 [`30714475985`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30714475985) 中 `npm-audit`、Ubuntu/Windows frontend 及相关主门禁全部成功。该 0 是冻结时点实测，不是永久无风险保证） |

**现在就能做（低成本）**：维持 dimension5 全检绿（项数以脚本输出为准）· `Cargo.lock` PR 更新 KNOWN_VULN · 发版前本地 `oclive lint --deny` · 校企仓要求组员 `npm ci && cargo build` 从源码跑通。

**下一工程动作**：K-SUPPLY-12 与 Nightly 分流已在冻结实现 `728219e7` 完成远端验收；11 场景影子模拟仍只是路由回归，不能替代真实 Compare。后续实机性能工作以 **K-VOICE-09** 的 30 分钟 TTFC 矩阵/人工听感和 K-RESOURCE-COORD-01 的长时间、不可控进程矩阵为主，不能把五分钟样本扩写成 72 小时认证；插件发布信任仍以 **K-SUPPLY-09** 为主要 P1。

---

## §2 冻结 / registry（明确「不动」）

| ID | 项 | 解冻条件 |
|----|-----|----------|
| **Deep / deep_capsule** | Turn Thinking Deep 路由 + deep_capsule 资产 | **已交付**（K-PERF-D1 / K-TURN-F1 Done；非冻结） |
| **dual_core** / **expert_routing** / **blueprint v3** | 实验管线 | **可选启用 · 默认仍关**（2026-07-24 LoRA directory LLM 选择链已接通；仍须 Cargo feature + 蓝图 `dual_core.enabled` + 角色包 `expert_routing.json` 显式配置） |
| **D-READ-03** | `dual_pipeline` 表驱动 | 随 `dual_core` opt-in |
| **D-PORT-03** | `BackendRegistry` UFCS 转发层 | 等第二 remote policy 实现或对应 RFC 再评估；D-PORT-02 / D-SLOT-01 已完成，不再以旧双实现为由解冻 |
| **§3.1** | 纯 library API 对称化 | 历史 [`RFC_OCLIVE_KERNEL_LIBRARY.md`](./archive/RFC_OCLIVE_KERNEL_LIBRARY.md) T0 |
| **模式 3** | 用户大纲演绎 / Mode 3 `send_message` 长对话 | 模式 2 playtest 扩展后另开计划 |
| **MEGA-SD-01** | `scene_director.rs` 拆 `theater/parse/` + `theater/modes/` | 模式 2 playtest 稳定 **或** 生产段 >2500 行 **或** 第二 remote 剧场插件 |
| **MEGA-TS-01** | `useTheaterShell.ts` 拆 poke/cast/outline composable | 同上；Shell 仅编排 |
| ~~**模式 2**~~ | — | **已解冻** · [`MODE2_RFC.md`](./theater/MODE2_RFC.md) · `outline_rewrite` |

**Phase 5 结论（2026-06-25 更新）：** 朋友 cohort 产品门通过 → **模式 2 开工**；`dual_core` / `expert_routing` **机制可选、默认关**。详见 [`theater/DEVELOPMENT_ROADMAP.md`](./theater/DEVELOPMENT_ROADMAP.md) §5.5。

---

## §3 观测台账（Observe · 无排期）

| ID | 项 | 说明 | 触发条件 | 下一动作 |
|----|-----|------|----------|----------|
| **D-PORT-03** | `BackendRegistry` UFCS 转发层 | D-PORT-02 已拆窄；collapse 等 remote policy RFC | 第二 remote 插件后端落地 or D-PORT-02 解冻 | 起草 remote policy RFC；评估 UFCS 层删除 |
| **D-READ-05** | `backend_registry` directory 子模块 | 机械拆文件；810 行可接受 | 文件 >1200 行 or 新 directory 后端类型 | 按子目录拆 `directory/` 模块 |
| **D-TRAIT-01** | 28 trait 单实现裁决 | 已裁决表保留；Repository 五件套合并等长期 | 外部贡献者要求合并 trait | 单 PR 合并一对 trait + 文档 |
| **D-POLICY-01** | Policy 三 trait 第二实现 | 等 remote policy RFC | remote policy RFC 合并 | 实现第二 `Policy*` 后端 |
| **D-ORPHAN-02** | `oclive_schema` 微型 crate | wasm 边界评估后再定 | wasm 宿主立项 | 评估合并进 `oclive_kernel_types` |
| **F4 / V2-remote** | remote 缺 env 静默回退 builtin | 已有 `startup_warnings`；矩阵诚实标 ⚠️ | 用户报告 silent fallback | 补 startup warning + 文档矩阵 |
| **K-PERF-D1** | Wave D · small-model Deep capsule | **Done** — `deep_capsule` 资产 + `PromptBuilder` · [`DEEP_PROMPT_DISTILLATION.md`](./DEEP_PROMPT_DISTILLATION.md) |
| **K-PERF-C1** | Wave C · Chat Pro 流式 UI | **Done** — `/chat/stream` + `chatStoreSend` · [`PERF_PHASES.md`](./PERF_PHASES.md) |
| **K-PERF-E1** | Wave E · Fast 持久化分流 `strong_only` | **Done** — `fast_persistence` · [`RFC_TURN_THINKING_PERSISTENCE.md`](../creator-docs/rfc/RFC_TURN_THINKING_PERSISTENCE.md) · `desktop` + `desktop-latency` profile |
| **K-TURN-F1** | Wave F · 角色包 `turn_thinking` 策略（Deep 路由 + Deep latch 直到和解） | P1 | RFC 定稿 + `config.json` schema + 内核 merge HostProfile；**无 UI 开关** | **Done** |
| **PE-TURN-01** | 编写器 · Turn Thinking / 对话节奏编辑（阈值、关键词、latch、可选 AND 规则） | P2 | 依赖 K-TURN-F1 schema · 简单/高级分档 | **OPEN**（姊妹仓 `oclive-pack-editor`） |
| **PE-UID-01** | 编写器 · `user_identities/` 可视化编辑（模板正文、`maps_to_relation_id`、与 `meta.relations.prompt_hint` 对齐预览） | P2 | ROLE_PACK_SPEC §1.1 · mumu `father.md` 手写 SSOT 已落地 | **OPEN**（姊妹仓 `oclive-pack-editor`） |
| **K-CONTINUITY-01** | 运行时叙事连续性微状态机 | 与核心 / 可变 / 短期情绪档案彻底分离；数据库按 `srid` 保存 `scene_id + state_id + revision`，位置、锚点、姿态、活动从角色包解析。运行时默认保持，仅在最终可见回复命中显式动作标记时转移，并通过动态 Prompt 段服务 Fast / Deep，不在热路径临时调用 LLM 生成状态 | **Partial**（2026-07-23：schema / validation / loader / CAS 持久化 / Prompt / 回复后迁移 / 场景失效 / Mumu 23 个候选已实现；本地门禁通过，后续远端 CI [`30683169339`](https://github.com/linkaiheng2233-cyber/oclivenewnew/actions/runs/30683169339) **22/22**；仍待同场景保持、显式动作迁移与切场景重选的人工 playtest） | 完成三类 playtest 并记录可复现证据后关闭 |
| **PE-CONTINUITY-01** | 编写器 · 场景初始状态候选生成 | 在**创作期**让模型根据场景描述生成 **3～8** 个候选初始状态，由创作者审核、修改、排序并写入角色包；运行期不临场生成，避免首字延迟和世界事实漂移 | **K-CONTINUITY-01** 可选 schema 冻结并进入编写器更新批次 | 增加生成 / 审核 / 默认项 / 条件与权重编辑；补 import-export roundtrip、非法锚点校验及旧角色包无字段兼容；姊妹仓 `oclive-pack-editor` 实施 |
| **K-ADULT-01** | R18 角色扩展 · 运行时成人链契约 | 角色成人表现、普通场景的 R18 走向、成人对话、动作流程、边界与会话状态需要一条独立于基础场景 Prompt 和模型/LoRA `ContentRating` 的正式契约；须按 G17 覆盖生产者、schema/validation、权限与确认、Prompt 适配、运行时状态/回退、角色/身份/场景切换及测试 | **v1 主链与 D25～D29 已实现并实机验证（2026-07-27；远端 CI 2026-08-01）**：除既有三重门、结构化双气泡、成人记忆分区与自动节拍外，现已增加 durable staged beat、显式 begin/stage/commit/cancel/list、提交幂等键、进程内取消令牌、全局有界公平队列、后台只缓存文本、前台逐拍提交/语音、重启恢复及用户输入抢占。预生成阶段不写聊天、短期记忆、关系、事件或人格；只有前台 commit 才写入。真实 7B GGUF 深度 1/2/4/8 与共享显存语音矩阵均通过；远端通用门禁已过，跨硬件档位与更长 soak 继续保留为发布验证，不回退本地主链状态 |
| **PE-ADULT-01** | 编写器 · R18 角色包额外拓展编写 | 在编写器空余更新中为成人角色表现、场景成人走向、成人对话与动作流程等提供独立创作面；不得退化为把长成人 Prompt 塞进现有 `scenePrompt`，也不得先于运行时契约自行发明第二套格式 | **Done（2026-07-27）**：姊妹仓已提供独立成人扩展页；完整基础包校验通过后方可进入，简单/高级/成人页共享同一草稿与导入导出链，扩展按 v1 校验并随完整角色包合并导出；旧包无扩展时保持兼容 |
| **K-UID-POST-01** | mumu 可选 `reply_post_processor` profile（care-package 句级裁剪 · remote/builtin） | P3 | 主链已用 `trim_template_repeat_reply` + Prompt 上一轮约束兜底；见历史 [Phase 2 记录](./archive/USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md) | **Deferred**（不默认开启 post-processor） |
| **K-PERF-10** | Chat chrome 懒加载 | **Partial** — overlay 已 lazy；chat chrome 仍 eager | 真人 playtest 归因首屏慢 **或** perf mark 超阈值 | 激活 chat chrome lazy PR |

### K-ADULT-01 / PE-ADULT-01 决策记录（逐项确认）

- **D1 · 打包与发行版边界（已确认 · 2026-07-26）：** R18 作为同一角色包内的可选扩展随包合并分发，内容以普通场景的 R18 走向为主，可附加成人状态下的人设与对话配置；它不是大型 DLC，也不属于通用根本角色包的必需格式。Chat Pro 决定是否加载该扩展；其他发行版可以按自身需要实现扩展能力，但无强制兼容义务。所有发行版必须能够忽略不支持的扩展并继续运行简单、通用的基础角色包。
- **D2 · Chat Pro 启用范围与管理入口（已确认 · 2026-07-27）：** 采用两级开关：先开启 Chat Pro 全局成人功能，再为具体角色单独启用 R18；新增角色 R18 管理界面，供用户集中查看并管理每个角色的开关。运行时只有两级均开启时才允许加载该角色的 R18 扩展；任意一级关闭时不得注入对应成人配置。
- **D3 · 成年确认重试与保存位置（已确认 · 2026-07-27）：** 尚无本机成年确认记录的用户每次尝试开启 Chat Pro 全局成人功能时都必须回答成年提问；回答未成年则本次拒绝启用 R18，且不写入“已成年”，下次尝试仍重新询问。回答成年后将结果保存在本机，之后关闭再重新开启时不重复确认。该记录只代表使用成人功能的资格确认，不自动代表对具体角色或具体互动的同意。
- **D4 · 角色成年声明与责任告知（已确认 · 2026-07-27）：** 含 R18 扩展的角色只需由创作者明确声明“角色为成年人”，不强制填写具体年龄。编写器与 Chat Pro 应分别向创作者和启用用户作出足够醒目、清楚、可理解的提示，说明角色成人声明、内容来源、本地运行、用户主动启用及创作者/用户各自责任，并留存本地确认状态，以尽可能降低误用与法律风险。产品文案不得把该提示表述为 OCLive 对不可依法免除义务的“一概免责”，也不得把违法内容风险仅凭“责任自负”转移给创作者或用户；本地运行减少服务端处理，并不当然免除软件分发、内容传播或其他适用义务，最终发行文案须经目标发行地区的法律审查。
- **D5 · 用户身份不构成额外授权层（已确认 · 2026-07-27）：** 用户完成本机成年确认后，不再要求对每个 `user_identity` 单独确认、授权或由创作者逐个加入白名单；运行时只认本机成年确认、全局开关与角色开关。
- **D6 · `adult_eligible` 降为兼容提示元数据（后续决定覆盖早期提案 · 2026-07-27）：** 早期“按身份阻止并提示切换”的构想已被随后“保持简单、相信用户自觉”的明确决定覆盖。字段继续可读以兼容旧角色包并供创作者表达提示，但不得作为隐藏的第四开关阻止成人扩展、成人记忆或分拍队列。
- **D7 · 导入含成人内容角色包时询问与启用顺序（已确认 · 2026-07-27）：** Chat Pro 导入检测到角色包包含 R18 扩展时，直接询问用户是否进入/启用 R18 模式。选择进入时严格按“全局开关 → 分角色开关”的顺序执行：若本机尚无成年确认记录，先询问；确认成年后开启 Chat Pro 全局成人功能，再开启当前导入角色的 R18 开关。回答未成年或取消时两个开关均不启用，角色包仍按普通内容运行。选择不进入时保持该角色 R18 关闭，但在设置的角色 R18 管理界面保留入口，用户之后可自行决定。
- **D8 · 关闭全局开关时保留角色选择（已确认 · 2026-07-27）：** 关闭 Chat Pro 全局成人开关时保留所有分角色 R18 开关的已保存状态，但运行时暂时不加载任何成人扩展；之后重新开启全局开关时恢复原有分角色选择，不要求用户逐个重新开启。
- **D9 · 两级开启后立即全量注入（已确认 · 2026-07-27）：** Chat Pro 全局成人开关与当前角色 R18 开关同时开启后，立即将该角色扩展中的成人状态人设、调情对话、动作流程提示及场景 R18 走向全部纳入普通聊天的 Prompt 组装，不再要求用户点击第三个入口或先手动选择成人场景。设置中的 R18 入口仅承担开关管理和状态说明，不作为额外加载门。
- **D10 · 角色可主动推进成人走向（已确认 · 2026-07-27）：** 两级开关同时开启后，视为用户允许角色依照成人扩展人设与当前场景主动将普通聊天推进到明确成人互动，不再为每次推进增加固定弹窗或单独确认步骤。该授权不推定为不可撤回；用户拒绝、停止或退出时的具体运行时行为仍须单独确认。
- **D11 · 宽松拒绝识别与自然退出（已确认 · 2026-07-27）：** 运行时以偏宽松的语义识别用户拒绝、停止或退出意图，优先避免漏判；命中后立即清除当前成人互动状态，并让角色以符合人设的方式自然收束、回到普通聊天，而非生硬截断。Chat Pro 同时提供固定“退出当前 R18 互动”按钮作为确定性兜底，避免自然语言识别迟钝。自然语言或按钮退出均保留全局及当前角色 R18 开关状态。
- **D12 · 退出后依人设自然再次推进（已确认 · 2026-07-27）：** 退出当前成人互动后不设置系统级冷却期，也不在本次聊天内禁止角色再次主动；后续是否以及多久再次推进，由角色成人扩展人设、当前场景和新的对话上下文自然决定。高主动性角色可以较快再次尝试，克制型角色也可以不再主动；拒绝识别与退出按钮始终继续生效。
- **D13 · 当前互动按聊天会话持久化（已确认 · 2026-07-27）：** 正在进行的 R18 互动状态按聊天会话独立持久保存；关闭应用、切换角色后返回或重新打开同一聊天时，恢复该会话原有互动状态和进度。不得仅依赖模型从聊天记录临时猜测；聊天记录、互动状态、全局开关及分角色开关须作为不同状态分别管理。
- **D14 · 切换场景自然结束当前互动（已确认 · 2026-07-27）：** 正在进行 R18 互动时切换场景，先清除旧场景中的当前成人互动状态，再让新场景从普通聊天状态开始；不把动作进度直接迁移到新地点。角色进入新场景后应依据自身人设、刚发生的转场上下文和新场景自然吐槽、回应或说一至数句，不使用生硬的统一结束模板。全局及角色开关继续保持，后续可以在新场景重新自然发展。
- **D15 · 切换用户身份自然结束当前互动（已确认 · 2026-07-27）：** 正在进行 R18 互动时切换 `user_identity`，采用与切换场景相同的收束规则：先让角色依人设自然结束并清除当前成人互动状态，再让新身份从普通聊天开始；角色应依据人设和身份变化自然回应。身份切换本身不新增成人授权层，后续仍按 D3～D6 的三项运行时条件处理。
- **D16 · 关闭开关立即中断并回到普通聊天（已确认 · 2026-07-27）：** 用户在当前 R18 互动中关闭全局或当前角色开关前，Chat Pro 必须醒目提醒“关闭会立即中断当前互动”。用户确认关闭后开关立即生效，清除该聊天会话的当前 R18 状态并卸载对应成人配置，不额外生成角色收束回复，直接进入普通聊天。关闭全局开关仍按 D8 保留各角色选择；关闭角色开关仅关闭该角色。
- **D17 · 角色对话与旁白结构化双气泡（已确认 · 2026-07-27）：** R18 回复使用结构化输出，至少将“角色对话”与“旁白/动作描写”作为两个独立字段。Chat Pro 先渲染干净的角色回复气泡，再在其后追加独立旁白气泡；不得把动作描写混入角色对话文本。该消费与展示契约属于 Chat Pro 成人扩展，不改变其他发行版和通用角色包的基础 `reply` 格式；不支持该扩展的消费者只需继续处理基础回复。
- **D18 · 语音只朗读角色对话（已确认 · 2026-07-27）：** R18 结构化回复进入语音链时只传递“角色对话”字段，旁白/动作气泡保持文字显示且静音，不得由角色声线朗读。独立旁白语音不进入首期范围，作为产品反馈触发的后续候选；只有实际用户反响证明有需求时再决定是否立项。
- **D19 · 创作者建议、用户覆盖与定时自动续拍（已确认原则 · 2026-07-27）：** 创作者可在 R18 扩展中提供互动节奏建议，或选择由 AI 根据人设、场景和当前进度决定动作节拍；Chat Pro 用户设置拥有最终覆盖权，并允许用户决定每拍等待间隔。每拍结构化回复完成后进入等待：用户在间隔内回复则取消自动触发并根据回复生成下一拍；无回复则间隔结束后自动生成下一拍。自然语言拒绝、固定退出按钮、关闭任一级开关、切换场景或切换用户身份必须立即取消尚未触发的续拍计时任务。设置作用域、应用后台行为和无人响应时最大自动续拍数仍须逐项确认。
- **D20 · 节拍间隔为 Chat Pro 全局用户设置（已确认 · 2026-07-27）：** 用户设置的每拍等待间隔作为 Chat Pro 全局值，对所有支持 R18 自动续拍的角色及聊天统一生效；角色包节奏字段只提供创作者建议或 AI 节拍策略，不保存强制的单角色覆盖值，聊天会话也不持久保存独立间隔。用户修改全局间隔后，后续新节拍按新值执行。
- **D21 · 双气泡完整显示后起算且不等待语音（已确认 · 2026-07-27）：** 每拍等待间隔在角色对话气泡和其后的旁白气泡都完整显示后立即开始计时；不以角色语音合成或播放完成作为起算条件。语音关闭、仍在合成或仍在播放均不延后倒计时。
- **D22 · 倒计时到期后等待本拍语音完成再续拍（已确认 · 2026-07-27）：** 倒计时到期时若当前拍角色语音尚未播放完成，不生成下一拍文本，而是将续拍标记为待触发；当前语音完成后立即生成下一拍。语音关闭时不应用该门，倒计时到期即可续拍。若语音合成或播放明显拖慢整体体验，Chat Pro 应提示用户可以关闭语音；提示阈值及语音超时/失败后的续拍行为仍须确认。
- **D23 · 语音失败后角落提醒并自动转纯文本（已确认 · 2026-07-27）：** 语音合成或播放失败时，在 Chat Pro 角落显示不遮挡聊天的错误提醒，自动跳过当前拍语音、解除语音完成续拍门并继续生成下一拍文本；不得自动反复重试同一语音。后续进入纯文本续拍，不再因语音阻塞。
- **D24 · 纯文本回退仅持续到当前 R18 互动结束（已确认 · 2026-07-27）：** 语音失败后的纯文本回退状态只属于当前这一次 R18 互动；当前互动自然结束、被用户退出或因既定状态迁移规则而终止时，必须随互动状态一并清除，不得关闭或覆写用户的 Chat Pro 全局语音设置。下次进入新的 R18 互动时，若语音仍由用户启用，则重新尝试语音；若再次失败，则重新显示错误提醒，并再次仅对该次互动降级为纯文本。
- **D25 · 后台连续生成、返回后逐拍展示（已确认原则 · 2026-07-27）：** 用户切换到其他聊天、Chat Pro 窗口失去前台焦点或窗口最小化时，只暂停当前聊天的新气泡展示，不暂停模型侧自动续拍；允许模型在后台连续生成多拍，并按所属聊天与互动顺序进入待展示队列。用户回到当前聊天且 Chat Pro 恢复前台后，必须依照原有节拍逐拍展示队列内容，不得一次性倾倒全部缓存。队列的具体允许范围、默认值与恢复生成时机仍须逐项确认。
- **D26 · 后台只缓存文本，返回后按拍顺序生成语音（已确认 · 2026-07-27）：** 当前聊天位于后台时，只生成并缓存每拍的结构化文本，不在后台为待展示队列提前批量合成或播放语音。用户返回且某一拍进入前台展示流程后，若用户已启用语音且语音链路没有阻塞或故障，则按照待展示队列顺序为该拍角色对话生成并播放语音，旁白仍保持静音；每拍继续遵守 D22 的语音完成续拍门。语音发生阻塞或故障时，按 D23、D24 跳过、角落提醒并仅将当前 R18 互动降级为纯文本，不得阻塞后续文本展示。
- **D27 · 后台待展示队列上限由用户配置并提供说明（已确认 · 2026-07-27）：** Chat Pro 为用户提供后台待展示队列上限选项，达到用户选择的上限后暂停该聊天的后台自动生成，保留已有队列并等待后续恢复，不丢弃或覆盖已生成节拍。该值由用户根据设备性能自行填写，产品不预设强制硬上限；输入至少须为可验证的正整数。设置项旁提供“？”帮助说明，明确该值的作用、资源占用影响、达到上限后的行为，以及过高数值可能造成的性能风险。
- **D28 · 后台队列上限为 Chat Pro 全局设置（已确认 · 2026-07-27）：** 用户选择的后台待展示队列上限作为 Chat Pro 全局值，对所有支持 R18 后台自动续拍的角色和聊天统一生效；角色包不得强制覆盖该值，也不为单个角色或单个聊天保存独立覆盖值。用户修改后，所有聊天后续进入队列的节拍均按新上限判断；既有队列调整行为按 D29 的工程收口原则处理。
- **D29 · 队列建议值由实现后的性能测试给出（本机矩阵 Done · 2026-07-27）：** 已新增可复现脚本 `node scripts/measure-adult-stage.mjs --base http://127.0.0.1:8430 --caps 1,2,4,8`，只记录耗时、字段完整性与资源数据，不记录生成正文。参考机为 RTX 5060 Laptop 8GB、Qwen2.5 7B Q4_K_M + 消融 LoRA、`llama-server --n-gpu-layers 99`：深度 1/2/4/8 共 15 拍全部结构化成功、零 LLM 回退，热态单拍 min/p50/p95/max 为 **1497/1754/2112/2195ms**；显存 **6208～6228MiB**，深度 8 未出现延迟或显存递增。共享显存档以 `--gpu-layers 24` + CosyVoice2 mixed-fp16 连续 5 组通过：峰值 **6751/8151MiB**、余量 **1400MiB**、稳态增长 **0MiB**，热态 LLM TTFT p50 **142ms**、语音 TTFC p50 **4293ms**。因此产品默认仍为 `2`，8GB + 7B Q4 建议 `2～4`；`8` 在参考机技术上稳定，但主要增加持续功耗和用户新输入导致缓存作废/剧情偏离的风险，而非显存线性增长。设置帮助已写入该建议；其他 GPU/模型档位继续由发布前远端/实机矩阵补证。
- **D30 · 模型可按剧情自然结束互动，用户退出始终保留（已确认 · 2026-07-27）：** 模型可以根据角色人设、场景与剧情进展自然判断本次 R18 互动已经结束；运行时以结构化终止状态接收该决定，在最终节拍完成后停止自动续拍、清除当前互动状态并回到普通聊天，不以自由文本关键词猜测结束。自然结束不关闭 Chat Pro 全局成人开关或角色 R18 开关，之后仍可依 D12 再次自然发展。用户的宽松自然语言退出识别和固定“退出当前 R18 互动”按钮在全过程保持可用，并优先于模型续拍或自然结束判断。
- **D31 · 用户输入抢占后台队列并在启用缓存时提示（已确认 · 2026-07-27）：** 若当前聊天存在待展示节拍或后台在途生成，用户一旦发送新消息，必须立即取消在途自动生成、废弃所有尚未展示的旧节拍，并以已经展示的聊天历史和该条新输入作为下一次生成上下文；不得让旧队列在用户回复之后继续插入。用户启用后台缓存能力时应主动提示这一行为，队列设置旁的“？”帮助信息持续说明“用户输入会使未展示缓存失效”，并建议高频参与回复的用户适当降低队列值、偏向连续观看的用户再结合设备性能调高。提醒呈现频率与控件细节由实现阶段按清晰且不过度打扰的原则确定。
- **D32 · 成人与普通记忆分区，并以非露骨摘要桥接连续性（已确认 · 2026-07-27）：** 成人内容记忆与非成人内容记忆必须使用可独立识别和管理的存储分区，不得混为一组无标签记忆。运行时先依据 Chat Pro 全局成人开关、当前角色 R18 开关和当前聊天语境限定可访问分区，再由 AI/检索层按相关性选择性读取，不允许把整个成人记忆库无条件注入 Prompt。普通聊天记忆仍应以简短、非露骨方式带过已经发生的暧昧或性关系事实及必要的关系变化，避免角色出现记忆断层；具体动作、露骨过程和仅适用于成人状态的细节留在成人记忆分区。分区须支持分别检查、清理和测试，关闭 R18 加载时不得读取成人细节，但不删除既有成人记忆。
- **D33 · 地点与成人题材不作为本地运行时审查开关（已确认 · 2026-07-27）：** 对通过成年确认且两级开关均开启的成人虚构创作，OCLive / Chat Pro 编排层不因场景位于公园、图书馆等公开地点，也不因具体成人题材类别而额外阻断、改写或降级；地点风险、第三方反应、隐蔽程度与现实后果属于角色包创作者的叙事空间。第三方本地角色包的导入校验聚焦结构、版本、成人声明与安全解析，不对其成人剧情做官方发行标准式的语义审核。此决定不撤销 D3～D6 已确认的成年确认与两级开关流程，也不要求底层模型绕过其自身能力或策略。
- **D34 · 本地创作自由与官方发布责任分层（已确认 · 2026-07-27）：** OCLive 的完全本地运行能力与官方宣传、第一方角色包、演示内容、文档及官方分发渠道必须分层治理：本地运行时以用户控制和创作自由为主；官方渠道则清楚声明成人功能边界、第三方内容归属、启用责任和适用规则，并只对官方实际发布的内容执行相应选材与审查。不得把官方渠道规则伪装成对全部本地创作的技术封锁，也不得宣传“本地运行即可当然免责”或保证任何用法均合法；正式发布文案仍需按目标地区做法律复核。
- **D35 · 独立 R18 扩展页、完整基础包前置与简单/高级编辑解耦（已确认 · 2026-07-27）：** 编写器在角色包工程内提供独立的“R18 扩展”页面，页面分为成人状态人设与对话、各基础场景对应的 R18 走向、动作节拍与节奏建议三个主要区域；不得把这些字段混塞进通用基础人设或基础场景表单。创作者只有在导入或完成一份完整且通过基础校验的角色包后，才能进入 R18 扩展编写流程；不提供脱离基础角色包的独立 R18 包创作与导出。现有简单编辑和高级编辑在界面、操作流程与职责上解耦：简单编辑提供受约束的引导式常用字段，高级编辑负责完整结构和精细控制；二者必须共享同一规范化领域模型、校验器和导入导出链路，不得复制数据或形成两套不兼容格式。最终产物仍按 D1 将基础内容与 R18 扩展合并为同一角色包。
- **实施收口（2026-07-28；远端证据 2026-08-01）：** D1～D35 的 v1 运行时/角色包/编写器链已实现并完成本地验证。D25～D28 的前置条件不再借用正式 turn：迁移 `038_adult_staged_beats.sql` 持久保存未展示结构化文本，stage 路径跳过聊天、短期记忆、关系、事件、人格与叙事连续性写入；前台按序 commit 后才进入历史与成人短期记忆，重复 commit 使用稳定幂等键。取消与新 generation 会删除未提交拍，生成完成和取消竞态经压力用例验证不残留 pending；Chat Pro 以所有角色/聊天共享的正整数容量暂停/恢复生成，降低上限不丢已有拍，重启后可恢复或取消。D29 的参考机真实模型、队列深度与共享显存语音矩阵已完成，默认值继续为 `2`；后续远端 CI 已通过，其他硬件档位与更长时间 soak 仍是发布验证项，不回退本地主链状态。

### K-PERF-10 条件门（2026-06-18）

| 信号 | 结果 | 处置 |
|------|------|------|
| 工程代理 15s 通过率 | **100%** | 不激活 chat chrome lazy |
| 首屏 perf mark（[`PLAYTEST_MATRIX.md`](./theater/PLAYTEST_MATRIX.md) §性能） | 无真人失败数据 | 维持 **Partial / Deferred** |
| 真人 <60% 且归因首屏慢 | 未发生 | 待 P0-STRANGER 后复评 |

**结论：** K-PERF-10 **不启动**；待真人测试若首屏 perf 失败再激活。

---

## §4 长期 Deferred（战略 · 不阻塞当前）

| ID | 项 | 说明 |
|----|-----|------|
| **K-PERF-15** | 记忆候选池语义变更 | 产品确认召回语义 |
| **V-FUSED-01** | 多 `slot_registry` 实例融合 | Phase 3 |
| **§3.5–3.7** | 多模态 / 参考硬件 / Edge OTA | 路线图 |
| **§5.3** | 插件市场 UGC | 路线图 |
| **V-LORA-WORKSHOP-01** | 创作者微调工坊（T0–T3）+ `slot.lora.apply` 运行时 | **Partial（2026-07-25）**：运行时已按 `plugin_id` 选择预声明、授权的 directory LLM并回流 Stable completion；本地 performance 路径另已接通单个 llama.cpp LoRA GGUF。仍 OPEN：训练工坊、专家路由绑定 UI、真实模型评测矩阵 |
| **V-LORA-FORGE-02** | 角色 LoRA「炼丹炉」产品层 | **Deferred · 后续追加开发**：不复制上游 WebUI、不在 OCLive 内核重写训练算法；定义独立 `TrainingProvider` 边界，首个 provider 优先封装 LLaMA-Factory CLI，可选 Unsloth 加速，未来允许 Axolotl/远端 GPU。OCLive 只拥有角色语料转换、训练预设、任务状态、角色一致性评测和产物打包 |
| **V-LORA-PACK-03** | `.ocadapter` 契约与导入/管理 UI | **Partial · 本地 v1 已交付（2026-07-25）**：原始 llama.cpp LoRA GGUF 与 `.ocadapter` 已打通安全导入、SHA/GGUF/architecture 校验、原子替换、成人确认、管理 UI、`llama-server --lora` 和失败回滚。仍 OPEN：revision/tokenizer/chat-template 指纹、签名/发布者信任、多 adapter/scale、角色/专家路由绑定与真实模型远端 CI smoke |
| **V-LORA-PEFT-04** | Hugging Face/PEFT adapter 导入转换插件 | **Deferred · 必须独立插件**：插件拥有 Hub/本地 PEFT `safetensors`、Python/Transformers/转换工具依赖与 GGUF 产出；稳定内核只接收本文 v1 GGUF/`.ocadapter`，禁止把 PEFT 运行时与 llama.cpp 本地加载链耦合 |
| **D-OPUS-05 Phase 2** | re-export import 清零 | ratchet ≤76 只降不升 |
| **K-SUPPLY-06** | 位级可重复构建 | 内核 `kernel-v0.x` tag 稳定 + 专用 CI 镜像 |
| **K-SUPPLY-07** | SBOM 导出 | 校企/商业客户采购或合规要求 |

---

## §5 历史归档

Done 项（K-PERF-01~26、D-READ-01/02/04、K-ROBUST-01~03、Opus 4.8 Wave 0–4、Fable 5 M0–M4、K-DOC-15/16 等）见：

- [RECURRING_OPTIMIZATION_PLAYBOOK.md §8](./RECURRING_OPTIMIZATION_PLAYBOOK.md) 巡检日志
- [CHANGELOG.md](../CHANGELOG.md) `[0.5.0]` · `[Unreleased]`
- git log `handoff/` · `kernel/crates/oclive_kernel_host`

### 轮次 16 Done（2026-06-18）

| ID | 项 | 说明 |
|----|-----|------|
| **T-LAYER-16** | Theater 测迁出 domain | `theater_director_resolver` → `distros/desktop-tauri/tests/theater_director_resolver.rs` |
| **T-DOC-TD-01** | `theater_director` 文档扫尾 | DISTRO / ARCHITECTURE / NAMING / ROADMAP §7 / IA 头注 / domain README |
| **T-MINIMAL-TD-01** | minimal 插件自包含 | `examples/directory-plugin-theater-director-minimal/prompts/` 本地 `buildTheaterPrompt` |
| **T-CI-DRIFT-01** | prompt drift 门禁 | `dimension5-acceptance.mjs` + `test:theater:smoke` 双挂 |

### 轮次 17 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| **D-DOCDRIFT-01** | monorepo 后文档路径机械迁移 | 一次性迁移脚本已在完成后删除（历史见 `git log -- scripts/migrate-doc-paths.mjs`）；现由 `check-stale-paths.mjs` 持续门禁 |
| **D-SCRIPT-02** | `check-stale-paths.mjs` 扩范围 | dimension5 十一检 |
| **D-ORPHAN-04** | 删 `kernel/crates/models/` 空目录 | workspace 无引用 |

### 轮次 18 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| **O-1** | plugin-bridge 资产内核化 | `kernel/crates/oclive_kernel_host/assets/plugin-bridge.iife.js`；删 desktop-tauri 副本 |
| **O-2** | expert 孤儿前端清理 | 10 文件删；Tauri expert API / validation / dual_core 链保留 |
| **D-DOC-RELOC-01** | 文档名实归位 | `VSCODE_DISTRIBUTION` → `handoff/vscode/`；`USER_GUIDE` → `handoff/studio/`；`MUMU_UI_ACCEPTANCE` → `handoff/distros/` |

### 轮次 19 Done（2026-06-24）

| ID | 项 | 说明 |
|----|-----|------|
| **K-SUPPLY-01** | `cargo deny` 硬门禁 | dimension5 检查项（licenses+bans）· `ci.yml` dimension5 job 安装 cargo-deny |
| **K-SUPPLY-02** | Release SHA256 | `generate-sha256sums.mjs` · `release-kernel-checksums.yml` · bundle 钩子 |
| **K-SUPPLY-03** | 插件审源码 toast | `installPath` DTO · 市场/git/zip · CLI · i18n |
| **K-SUPPLY-DOC-01** | 供应链策略 SSOT | `creator-docs/security/SUPPLY_CHAIN.md` + 本文件 §1.5 |

轮次 1–15 明细表已从本文件移除以降低噪音；需要历史格查 git `handoff/TECHNICAL_DEBT_INVENTORY.md` @ 2026-06-15。

---

## 速查坐标

| 用途 | 路径 |
|------|------|
| 编排 SSOT | `kernel/crates/oclive_kernel_host/src/domain/chat_engine/process_message.rs` |
| 槽态矩阵 | [SLOT_BACKEND_REALITY_MATRIX.md](./SLOT_BACKEND_REALITY_MATRIX.md) |
| Theater 验收 | [PLAYTEST_MATRIX.md](./theater/PLAYTEST_MATRIX.md) |
| Theater 模式 2 解冻 | [MODE2_UNFREEZE.md](./theater/MODE2_UNFREEZE.md) |
| 分层 ratchet | `handoff/LAYERING_BASELINE.json` |
| Theater director 集成测 | `distros/desktop-tauri/tests/theater_director_resolver.rs` |
| 供应链策略 | [SUPPLY_CHAIN.md](../creator-docs/security/SUPPLY_CHAIN.md) |
