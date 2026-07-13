# Technical debt inventory

**Last updated:** 2026-07-13（技术债收敛波收尾 · 错误码 SSOT 门禁 · 契约边界文档 · supply-chain EN 镜像对齐）

**Product freeze (Theater v0):** **Lifted** — 朋友 cohort 产品门通过（7/10 卧槽）；模式 2 playtest 扩展中；**模式 3 仍冻结**。见 [theater/MODE2_UNFREEZE.md](./theater/MODE2_UNFREEZE.md)。

**综合评分：** A− · 本地 dimension5 **十九检** PASS（--ci 十八检执行 + 1 项 SKIP 仍计入）· workspace **doctest** 见 check:release · 审查汇报 SSOT：[AI_VERIFICATION_PROTOCOL.md](./AI_VERIFICATION_PROTOCOL.md)

**下一动作：** **P1** — 模式 2 playtest 扩展至陌生人 cohort；**Observe** K-SUPPLY-05 依赖去重

**Verification (2026-07-13 · 内核解耦波收尾)：** `plugin_resolution` runtime 单测 + `slot_resolution_chain` / `memory_lifecycle` / `rpp_contract_audit` / `theater_director_resolver` 集成测 + `oclive-cli`（`diagnostics-host` 默认 off）+ dimension5 二十检 PASS + 远程 CI run **`29238028141`** 绿 · HEAD **`8f1a9b99`**。

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
| **K-SUPPLY-04** | 前端 `npm-audit` 仅可见性（`continue-on-error`） | P2 | 连续 2 周期高危命中 → 升格硬门禁或文档豁免 | **Observe**（2026-07-12：`defu` override **6.1.5** 清零高危；`npm audit --omit=dev` **0 高危 / 4 中危 / 1 低危** — `vue3-sfc-loader` 链待专项） |
| **K-SUPPLY-05** | `deny.toml` `multiple-versions = warn` | P2 | 依赖树去重后改 `deny` | **Partial**（2026-07-13 · `check-cargo-dedup-ratchet.mjs` baseline **103** · dimension5 门禁；`deny` 仍 warn） |
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
| **D-SLOT-01** | BuiltinV1/V2 选择收到 resolver | P2 | 依赖 D-PORT-02 后续 | **Observe** |
| **D-TRAIT-01** | 单实现 trait 合并 | P3 | 仅明显 DI 噪音 | **Observe** |
| **K-VOICE-01** | CosyVoice2 `stream=True` 在 Windows 死锁（侧车多进程 worker）→ 默认非流式合成，牺牲首字流式增益（整句 ~3s 出声） | P2 | 上游修复 or `num_workers=0`/spawn 规避经 `OCLIVE_COSYVOICE_STREAM=1` 实测不卡后解冻 | **Deferred**（默认非流式已上线 · 排查见 [`TRACK_VOICE_RECOGNITION.md`](../human-docs/team/TRACK_VOICE_RECOGNITION.md) §10） |
| **K-VOICE-02** | Tier-2 TTS（ChatTTS · XTTS · Bark · VITS · 国内云 API · Piper 产品化） | P3 | VX-9 generic pack 模板或社区 adapter | **OPEN** |
| **K-VOICE-03** | Linux/macOS CosyVoice2 产品 profile | P2 | 随上游 CosyVoice 跨平台稳定后解冻 `asr_profiles.json` platforms | **OPEN** |
| **K-VOICE-04** | 角色包 `preferred_tts_profile` 可选字段 | P3 | ROLE_PACK_SPEC §10 已增补字段 · 设置页默认联动待实现 | **Done**（`voice.read_role_profile` + `useRoleVoiceProfileSync` · 2026-07-10） |
| **K-VOICE-05** | Qwen3-TTS 官方 REST 契约稳定化（社区 server 方言多） | P2 | 随上游收敛后收紧 adapter；Fish 默认端口已改 **9881** 避免与 Qwen **8080** 冲突 | **Observe** |
| **K-VOICE-06** | 社区 directory 插件 `com.user.tts.*`（自带 sidecar/RPC） | P2 | VX-10 · `plugin_rpc_invoke` 白名单 | **OPEN** |
| **K-VOICE-07** | `voice_directive` v2 + `engine_extras` 透传 bag | P2 | RFC §4.1 小节后实现 | **OPEN** |
| **K-VOICE-08** | 全引擎统一流式 playback contract | P2 | 非 CosyVoice chunked audio 抽象 | **Deferred** |


---

## 前瞻性结构风险（2026-07-12 审查增补 · 2026-07-13 拆项）

| ID | 项 | 优先级 | 解决/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **K-PLATFORM-01** | Tauri v1 → v2 迁移 | **P1** | **契约**：权限/capability schema 对照 + **测试**：最小 smoke + **改动面**：desktop-tauri / 三 distro 分 PR | **OPEN** |
| **K-LLM-01** | LLM 后端单一依赖 Ollama | **P1** | **契约**：`LlmBackend` env 矩阵 + **测试**：1 API + 1 本地 mock + **改动面**：adapter 接线 PR | **OPEN** |
| **K-CROSS-01** | 跨平台系统策略缺失 | **P2** | 三平台语音 smoke + distro profile 差异声明 | **OPEN** |
| **K-DIST-01** | 分发体验缺口 | **P2** | 签名/updater/Linux 包/macOS dmg | **OPEN** |
| **D-I18N-02** | creator-docs-en 镜像滞后 | **P2** | **契约**：`check-doc-mirror` 扩展 + **测试**：mirror ratchet 样例 + **改动面**：creator-docs-en 补链 PR | **Partial**（2026-07-13 · `--warn-drift-high-traffic` 四路径硬门禁 + ROLE_PACK_SPEC §9.7 EN 补链） |
| **V-MARKET-01** | 插件市场生态 | **P2** | 市场 UI + 社区插件 | **OPEN** |

**K-PLATFORM-01 子项（立项条件，本批不实施）**

| 子 ID | 契约 | 测试 | 改动面 |
|-------|------|------|--------|
| K-PLATFORM-01a | Tauri v2 permission manifest 对照 | 单命令 invoke smoke | `tauri.conf.json` + ACL |
| K-PLATFORM-01b | `@tauri-apps/api` v2 迁移表 | chat-pro 发消息 E2E | `distros/shared` IPC |
| K-PLATFORM-01c | CI 镜像 + dimension5 口径 | workflow 绿 | `.github/workflows` |

**K-LLM-01 子项**

| 子 ID | 契约 | 测试 | 改动面 |
|-------|------|------|--------|
| K-LLM-01a | OpenAI-compatible API env SSOT | mock HTTP 集成测 | `openai_compatible_llm.rs` |
| K-LLM-01b | 第二本地后端选型 | 可选 feature gate 测 | 新 adapter 模块 |

**系统性债务（2026-07-12 审查增补）**

| ID | 项 | 优先级 | 解决/完成条件 | 状态 |
|----|-----|--------|----------------|------|
| **D-ARCH-01** | 六槽解析链 SSOT | **P1** | MODULE_MAP §3.2 + 集成测试（legacy/v2 + session override + host ceiling） | **Done**（2026-07-13 · `slot_resolution_chain.rs` 四测 + MODULE_MAP §12.5） |
| **K-MEM-01** | STM→LTM 生命周期分散 | **P1** | CHAT_STORAGE 表 + 集成测试 | **Done**（2026-07-13 · `memory_lifecycle_integration.rs` 六测 · merge/strong_only/prompt 读取） |
| **K-FREEZE-01** | 冻结状态不透明 | **P1** | 技术债 §2 收敛 | **Done**（2026-07-12） |
| **K-TEST-01** | check:rust 仅 --lib | **P2** | check:rust:integration；**盲区在本地而非 CI** | **Done**（2026-07-12） |
| **K-CONFIG-01** | 配置无诊断 | **P2** | oclive-cli doctor config-resolve + `--json` + 自动测试 | **Done**（2026-07-13 · runtime `plugin_resolution` 纯路径默认 · `diagnostics-host` feature 可选 host · `cargo tree` 无 sqlite/axum/tauri） |
| **K-ERR-01** | 热路径错误码 | **P2** | 插件/manifest/迁移结构化码 + 契约测 | **Done**（2026-07-13 · `KernelErrorBody.context` · `kernelErrorCodes.ts` · dimension5 drift 门禁） |
| **D-ROLEVER-01** | 角色包版本迁移 | **P2** | ROLE_PACK_SPEC 章节 | **OPEN** |
| **T-DOC-02** | Theater 状态单页 | **P2** | theater STATUS | **OPEN** |
| **K-RPP-01** | RPP 无契约 | **P2** | PLUGIN_V1 或 RPP_CONTRACT | **Done**（2026-07-13 · `rpp_contract_audit.rs` 四测 · manifest/RPC/roundtrip/mumu 默认关） |
| **K-RESILIENCE-01** | Remote 弹性分散 | **P2** | ResilienceLayer | **OPEN** |

## §1.5 供应链安全（Supply Chain · 2026-06-24）

**策略 SSOT**：[`creator-docs/security/SUPPLY_CHAIN.md`](../creator-docs/security/SUPPLY_CHAIN.md)

### 基线（已落地 · 非债）

| 护栏 | 说明 |
|------|------|
| `cargo audit` 0.22.1 | dimension5 + `ci.yml` + `cargo-audit-lockfile.yml` 三层硬门禁 |
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
| **K-SUPPLY-04** | npm-audit 升格策略 | P2 | **Observe** — `defu` override 6.1.5 清零高危（2026-07-12）；`vue3-sfc-loader`/postcss 链 **0 高危 / 4 中危 / 1 低危** 待专项；下复核 **2026-08** |
| **K-SUPPLY-05** | deny 重复依赖 warn→deny | P2 | **Partial** — `LAYERING_BASELINE.json` `cargo_duplicate_groups: 103` · dimension5 ratchet；`deny.toml` 仍 warn |
| **K-SUPPLY-06** | 位级可重复构建（reproducible） | — | **Deferred** · 见 SECURITY_AUDIT_SCOPE 局限 |
| **K-SUPPLY-07** | SBOM（CycloneDX/SPDX） | — | **Deferred** · 政企/校企采购需求触发 |
| **MEGA-SD-01** | `scene_director.rs` 巨无霸拆分 | 见 §2 解冻条件；零语义变更 PR |
| **MEGA-TS-01** | `useTheaterShell.ts` 巨无霸拆分 | 见 §2；`mapTheaterInvokeError` 已先行减负（轮次 22） |
| **K-SUPPLY-08** | crate 作者信誉 / 发布历史系统审计 | — | **Observe** · 无成熟自动化方案 |

**现在就能做（低成本）**：维持 dimension5 十八检绿 · `Cargo.lock` PR 更新 KNOWN_VULN · 发版前本地 `oclive lint --deny` · 校企仓要求组员 `npm ci && cargo build` 从源码跑通。

**下一工程动作（P1）**：K-SUPPLY-02 Release 哈希清单（与 `kernel_manifest` / bundled kernel 发版对齐）。

---

## §2 冻结 / registry（明确「不动」）

| ID | 项 | 解冻条件 |
|----|-----|----------|
| **Deep / deep_capsule** | Turn Thinking Deep 路由 + deep_capsule 资产 | **已交付**（K-PERF-D1 / K-TURN-F1 Done；非冻结） |
| **dual_core** / **expert_routing** / **blueprint v3** | 实验管线 | **可选解冻 · 默认仍关**（蓝图 `dual_core.enabled` / 角色包 `expert_routing.json` 显式配置） |
| **D-READ-03** | `dual_pipeline` 表驱动 | 随 `dual_core` opt-in |
| **D-PORT-02** / **D-SLOT-01** | god-port collapse / 槽调度 | phase 1 memory 已拆；余组 Observe |
| **§3.1** | 纯 library API 对称化 | [`RFC_OCLIVE_KERNEL_LIBRARY.md`](./RFC_OCLIVE_KERNEL_LIBRARY.md) T0 |
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
| **K-UID-POST-01** | mumu 可选 `reply_post_processor` profile（care-package 句级裁剪 · remote/builtin） | P3 | 主链已用 `trim_template_repeat_reply` + Prompt 上一轮约束兜底；见 [USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md](./USER_IDENTITY_REPLY_POST_PROCESSOR_PHASE2.md) | **Deferred**（不默认开启 post-processor） |
| **K-PERF-10** | Chat chrome 懒加载 | **Partial** — overlay 已 lazy；chat chrome 仍 eager | 真人 playtest 归因首屏慢 **或** perf mark 超阈值 | 激活 chat chrome lazy PR |

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
| **V-LORA-WORKSHOP-01** | 创作者微调工坊（T0–T3）+ `slot.lora.apply` 运行时 | 三发行版 smoke 后；愿景 [VISION_ROADMAP_MONTHLY.md](../creator-docs/roadmap/VISION_ROADMAP_MONTHLY.md)「微调工坊」；冻结期内仅 T0 契约 + T1 原型 |
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
| **D-DOCDRIFT-01** | monorepo 后文档路径机械迁移 | `migrate-doc-paths.mjs` / `fix-remaining-doc-paths.mjs`；206 文件；`check-stale-paths` 硬门禁 |
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