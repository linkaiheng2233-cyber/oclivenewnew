# Technical debt inventory

**Last updated:** 2026-06-11 (Fable 5 巡检 Phase 0–4)

**Product freeze (Theater v0):** No new kernel orchestration / six-slot expansion until strangers validate AI Theater v0. See [PRODUCT_FREEZE_THEATER_V0.md](./PRODUCT_FREEZE_THEATER_V0.md). **Deferred unchanged:** K-PERF-10, K-PERF-14/15, §3.1 library API, dual_core (frozen).

### Fable 5 巡检收口（2026-06-11 · Phase 1–4）

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-PERF-20 | `RoleRuntimeSnapshot` 下游复用（emotion / profile personality / relation fallback） | **Done** | `relation_snapshot` / `post` / `pre` Profile 路径；每回合 `get_current_emotion` ≤1（写后刷新除外） |
| K-PERF-21 | `resolve_effective_ollama_model` settings 批量读 | **Done** | `get_app_settings([provider, remote_model])` 单次 RTT |
| K-PERF-22 | 聊天 session 列表 snippet + upsert | **Done** | 窗口函数 JOIN；`upsert_chat_session` `RETURNING` 消除写后重读 |
| K-PERF-23 | `034_perf_indexes.sql` | **Done** | `idx_ltm_role_content` · `idx_operation_logs_role` |
| K-PERF-24 | post `Role` clone 减少 | **Done** | `TurnContext.role_arc` 供 profile evolution spawn |
| D-ERR-02 | `TurnError → AppError` 保留 stage | **Done** | `with_chat_stage`；单测 `kernel_error_body` 含 stage 前缀 |
| D-ORPHAN-04 | 删除无消费方 `RoleRuntimeRepo` | **Done** | `preflight_turn_runtime` supersede |
| D-NAME-01 | `resolve_backend_kind` → `pick_chat_storage_backend_kind` | **Partial** | chat_storage 批次 Done；`resolve_project_root` CLI 收敛仍 Deferred |
| D-PORT-03 | `BackendRegistry` trait 纯转发 | **Observe** | UFCS 必需（trait/inherent 同名防递归）；待 remote policy 或第二实现再 collapse |
| K-DOC-10~12 | 分层 3/1、债务自洽、Agent 规则漂移 | **Done** | ARCHITECTURE_LAYERING · domain README · `.cursor/rules` · oclive-vscode AGENTS |
| K-PERF-19+ | 前端 follow-up | **Done** | `patchMessageById` 原位更新；轮询复用 `kernelConnectionStore` |

**Verification (2026-06-11):** `node scripts/dimension5-acceptance.mjs --ci`（**9** checks）；`cargo test -p oclive_kernel_host --lib`；`node scripts/check-domain-layering.mjs`。

### Phase 4 · Deferred 登记（解冻条件）

| ID | Item | 解冻条件 | 愿景轴 |
|----|------|----------|--------|
| K-PERF-14 | `pre_llm` 串行 await 并行化 | Theater v0 陌生人测试通过 **或** latency 预算失败 | V1 / 剧场实时 |
| K-PERF-15 | 记忆候选池 10 条按时间非相关性 | 产品确认召回语义变更可接受 | V2 |
| K-PERF-10 | Chat chrome 懒加载 | Theater 首屏 perf 验收失败 | V1 |
| K-CONTRACT-WIRING-01 | `extra_sections` 生产接线 | V-CONTRACT Phase 1+ | V2 |
| V-VSCODE-PERF-05 | F5 实机 / `.vsix` 发布验收 | **人工**排期（2026-Q3 建议） | V3 |
| §3.1 | 纯 library API 对称化 | 第二宿主强需求 + RFC | V1 |
| D-POLICY-01 | Policy 三 trait 第二实现 or collapse | remote policy RFC 或连续两发版无第二实现 | V2 |
| D-PORT-03 | BackendRegistry 转发层 collapse | 见上 Observe | V2 |

**建议：** K-PERF-14 与 K-PERF-20 **同批解冻**（均触 `turn_pipeline/pre.rs`）。

### 五维审查收口（2026-06-11 · Batch 1–3 Done）

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-DOC-DRIFT-01 | 契约级文档漂移（共景主链 / VS Code attach 残留 / user_identities 语义） | **Done** | ARCHITECTURE_OVERVIEW · VSCODE_DISTRIBUTION · CROSS_HOST_MEMORY · ROLE_PACK_SPEC §1.1 |
| K-PERF-16 | `role_runtime` 回合预取合并（preflight + SessionCache interaction_mode） | **Done** | `preflight_turn_runtime`；3–4 SELECT/回合 → 1 SELECT + 1 UPDATE |
| K-PERF-17 | 记忆 decay 批 UPDATE（CASE id WHEN） | **Done** | `persist_memory_decay_batch` 单语句 |
| K-PERF-18 | TurnPrefetch / post 事件链减少 clone | **Done** | `pre.rs` borrow+to_vec；`post.rs` extend |
| K-PERF-19 | `chatStore` 加载 aside / addMessage / 懒 split | **Done** | 当前 bucket sync；push+trim；tail 80 懒 split |
| K-BUILD-03 | workspace `default-members` 排除 `fuzz` | **Done** | 根 `Cargo.toml`；CI `--workspace` 不变 |
| V-VSCODE-CI-01 | 姊妹仓 CI 增 `npm run test:unit` | **Done** | `oclive-vscode/.github/workflows/ci.yml` |
| K-CONTRACT-WIRING-01 | `extra_sections` 生产未接线 | **Deferred** | V-CONTRACT Phase 1+ |

**Verification (2026-06-11):** `node scripts/dimension5-acceptance.mjs --ci`；`cargo test -p oclive_kernel_host`；`cargo test -p oclive_validation`。

### V-CONTRACT contract expressiveness (2026-06-10 · Phase 0 Done)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| V-CONTRACT-01 | `SlotExtension` envelope type + re-export in `oclive_kernel_types` | **Done** | `slot_extension.rs`; serde roundtrip tests |
| V-CONTRACT-02 | `EmotionResult.extension` / `ComplexEmotionOutput.extension` (`#[serde(default)]`) | **Done** | Additive; kernel does not interpret `data` |
| V-CONTRACT-03 | `PromptInput.extra_sections` + anchor-before render in `PromptBuilder` | **Done** | All call sites pass `&[]` until host wiring |
| V-CONTRACT-04 | Contract evolution rules in `EXTENSION_POINTS.md` (zh/en) | **Done** | Additive-only, `non_exhaustive`, breaking process |
| V-FUSED-01 | Multi `slot_registry` instance → same directory plugin; validation + docs | **Deferred** | Phase 3; after first external plugin author |

**Phase 1–3 (post Theater demo):** `plugin.describe` capability negotiation; `034_slot_state.sql` per-slot private state; OOCP golden scenarios + `oclive_kernel_contracts` rustdoc publishing.

**Verification (2026-06-10 round 10):** `node scripts/dimension5-acceptance.mjs --ci` PASS (**9** checks); `cargo test -p oclive_kernel_host --lib` green; `npm run test:unit` 18 files / **58** passed; `npx vite build` PASS; `npm run verify:ui` PASS; theater acceptance **9** tests green.

### Round 10 patrol（2026-06-10 · gate + Phase D）

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-GATE-01 | dimension5 第 8 检 `verify:ui` + 第 9 检 `vite build`；CI dimension5 job 增 `npm ci` | **Done** | 本地/CI 快档可捕获 K-BUILD-02 类前端构建回归 |
| D-ORPHAN-03b | `usePluginDebug.ts` 删除；`RpcHistoryItem` → `src/types/pluginDebug.ts` | **Done** | RpcTester 类型迁入 |
| D-ORPHAN-03c | `devTools.pluginDebug.*` i18n 半孤儿键删除 | **Done** | RpcTester 仅用 `devTools.rpc.*` |
| K-DOC-09 | `docs/I18N_PROGRESS.md` 去除 `PluginManagerPanel` / `PluginDebugPanel` 引用 | **Done** | 与 AGENTS.md 一致 |
| D-PORT-02 | god-port 拆为 `SlotBackendFactoryPort` + `LocalPluginRegistryPort` + `AgentMcpRegistryPort`；`PluginBackendRegistryPort` blanket；`SlotResolver` 窄端口 | **Done** | 删除 24 方法单体转发 impl；子 trait impl 保留 |
| D-SLOT-01 | 删除四槽 `BuiltinV2` 实现；serde `builtin_v2` → `builtin` alias | **Done** | 20 格矩阵；breaking 见 BREAKING_CHANGE_PROCESS |
| D-NAME-01 | `pick_*`（backend_registry 目录槽/复杂情感）+ `load_*`（chat_storage config）首批 | **Partial** | 余 `merge_*` / `find_*` 批次待续 |
| D-TRAIT-01 | 单实现 trait 裁决表 | **Done** | 见下表 §D-TRAIT-01 裁决 |

### D-TRAIT-01 单实现 trait 裁决表（2026-06-10）

| 类别 | Trait / Port | 实现数 | 裁决 |
|------|----------------|--------|------|
| 六槽多实现 | `LlmClient`, `MemoryRetrieval`, `AgentProvider`, … | 2+ | **保留 trait** |
| Repository 五件套 | `MemoryRepository`, `FavorabilityRepository`, … | 1 | **Deferred** → 合并 `Arc<DbManager>`（`RoleRuntimeRepo` 已删） |
| Policy 三件套 | `EmotionPolicy`, `MemoryPolicy`, `EventPolicy` | 1 | **保留** 至 remote policy RFC |
| 纯转发 port | `PluginHostPort`, `SlotRegistryResolver`, `BackendRegistry` 子 trait UFCS | 1 | **Observe**（D-PORT-03）；随 remote policy RFC |
| MCP/解析 | `FunctionCallingParserPort`, `McpBridgePort` | 1 | **保留**（测试替身价值） |
| Host port | `DbHealthPort`, `ConversationStore`, … | 1 | **Observe** 随 D-PORT-02 后续批次 |

槽态矩阵：**[SLOT_BACKEND_REALITY_MATRIX.md](./SLOT_BACKEND_REALITY_MATRIX.md)**（24 格 · 2026-06-10）。

### Round 9 patrol（2026-06-10 · 过度工程普查 + 构建修复）

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-BUILD-02 | `TheaterShell.vue` 相对导入少一层 `../`（`../theater/*` 应为 `../../theater/*`）致 **`vite build` 在 HEAD 上直接失败**；`npm run build` 经 `concurrently` 包裹时表现为挂起 | **Done** | 5 处 import 修正；`npx vite build` 4.5s 绿。Theater v0 demo 构建路径恢复 |
| D-SCRIPT-01 | `verify:ui`（`scripts/verify-frontend-patches.mjs`）5 锚点中 4 个引用已删 V1 面板（`PluginManagerPanel` / `PluginBackendSessionPanel` / `panelMainTab`），且 `readFileSync` 异常未捕获直接崩溃；`check:release` 链因此必红 | **Done** | 重写为当前生产锚点（`SimplePluginManagerPanel` / `ModelManagerPanel` / FluentShell 挂载 / hotkeys），逐项 try/catch |
| D-ORPHAN-03 | V1 插件 UI 孤儿组件 | **Done·deleted** | 轮次 9 删面板；轮次 10 删 `usePluginDebug` 壳 + i18n |
| K-DOC-08 | `oclive_runtimed` 删除后幽灵引用（`crates/README.md` 速查表行 + `NAMING_CONVENTIONS.md` §3.1/§3.4 + `P4_CRATE_AUDIT.md`）；`crates/README.md` `prompt_builder.rs` 路径笔误；`AGENTS.md` 仍称 V1 面板「代码保留」；`REGRESSION_COMPLEX_EMOTION_QA.md` 整篇针对已删 UI | **Done** | 全部更正/标注已删；QA 文档加过时横幅 |
| D-TRAIT-01 | 单实现 trait 普查：contracts 22 个 pub trait 中 **16 个仅 1 个生产实现** | **Deferred→Done** | 轮次 9 入账；轮次 10 裁决表见上 |
| D-NAME-01 | `resolve_*` 命名消歧 | **Partial** | `pick_*` backend_registry + `load_*` chat_storage 首批 Done |
| D-PORT-02 | god-port 拆窄 trait + blanket `PluginBackendRegistryPort`；`SlotResolver` → `SlotBackendFactoryPort` | **Done** | 轮次 10；子 trait 仍委托 inherent 方法 |
| D-SLOT-01 | 四槽 `builtin_v2` 测试桩删除 | **Done** | serde alias 兼容旧 settings；矩阵 24→20 格语义 |

### Freeze-safe audit（2026-06-09）

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-PERF-13 | `http_api_roles` 无界 DashMap（`--api` 长跑） | **Done** | `insert_http_api_role` FIFO cap 32（对齐 `role_cache`） |
| K-BUILD-01 | `kernel_server/build.rs` `SystemTime::now()` 致增量编译失效 | **Done** | `SOURCE_DATE_EPOCH`（缺省 0）+ `rerun-if-env-changed` |
| K-PERF-14 | `pre_llm` 独立 await 串行 | **Deferred** | 触碰编排；冻结至 Theater v0 |
| K-PERF-15 | 记忆候选池固定 10 条按时间非相关性 | **Deferred** | 影响召回语义；冻结 |
| D-CLEAN-01 | `ReplayTaskRegistry` 完成不清理 | **Done** | 完成 TTL 600s + `get()` 读后清理（Wave 2） |
| D-NAME-01 | `resolve_*` 命名消歧（全仓 **104** 个 `fn resolve_`） | **Partial** | 轮次 10：`pick_*` + `load_*` 首批；余批次 Deferred |
| D-ORPHAN-01 | `oclive_runtimed` 调度守护原型（8430 端口 / per-role 队列） | **Done·deleted** | 不在 workspace、无产品接线；设计：`OCLIVE_KERNEL_UPSTREAM`→8420、`OCLIVE_SCHEDULER_PORT`→8430；恢复：`git log --diff-filter=D -- crates/oclive_runtimed` |
| D-ORPHAN-02 | `oclive_schema` 微型 crate（18 行 blueprint 片段） | **Observe** | 冻结期不合并回 `oclive_kernel_types`；评估 wasm/独立校验边界后再定 |
| K-DOC-07 | `AGENTS.md` / `LIGHTWEIGHT_PROFILE` cargo-audit `continue-on-error` 漂移 | **Done** | 更正为 dimension5 + `cargo-audit` job + lockfile workflow 三层硬门禁 |

**Opus 4.8 follow-up (2026-06-08):** five-dimension re-review — `node scripts/dimension5-acceptance.mjs --ci` PASS; K-PERF-01 batched memory-decay writes; K-PERF-02 stage tracing + PERFORMANCE.md §6; K-DOC-02 CHANGELOG parity CI; K-PROFILE-01 unified `distro_oclive_file`; D-OPUS-05 re-export ratchet; D-OPUS-01/02/04 RC lightweight sweep Done. See §Opus 4.8 follow-up.

**Opus 4.8 second pass (2026-06-08):** five-dimension re-review #2 — gate re-run PASS (7 checks), no regressions. Fixed (zero-risk): K-PERF-09 lazy shells, K-DOC-03/04 doc drift. **Implemented (PR-A/B/C/D/E matrix):** D-LAYER-05 (FQ 14→5 + turn ports), D-DTO-01, D-ERR-01 (incremental), K-PERF-03..08/11/12, K-DOC-05/06; K-PERF-10 **Partial** (overlay panels lazy, chat chrome still eager). Still Deferred: D-PORT-02, D-SLOT-01. See §Opus 4.8 五维复审追加.

### Kernel profile scheduling (2026-06-08)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-PROFILE-01 | Dual TOML parse (`kernel_distro_profile` vs `host_profile`) | **Done** | SSOT `oclive_kernel_runtime::distro_oclive_file`; RFC [RFC_PROFILE_AND_DOMAIN_REEXPORT.md](../creator-docs/rfc/RFC_PROFILE_AND_DOMAIN_REEXPORT.md) |
| K-PROFILE-02 | `/health` summary non-runtime | **Done** | `HostProfile::active_profile_summary()` SSOT |
| K-PROFILE-03 | `distro_id`-only weak compat | **Done** | Hash required without summary; else Unknown |
| K-PROFILE-04 | Desktop missing bundled `distro.oclive.toml` | **Done** | `src-tauri/resources/distro-profiles/{desktop,theater}.oclive.toml` + policy resolve |
| K-PROFILE-05 | Legacy attach bypasses profile | **Done** | Graded fallback + profile-aware attach |
| K-PROFILE-06 | Duplicated resolve/health types | **Done** | `build_resolve_plan`, `KernelHealthJson` in types, `kernel_port_ops` |

### Phase 2 (2026-06-07)

| Item | Status | Notes |
|------|--------|-------|
| Six-slot `none` + MODULE_NONE_SEMANTICS | **Done** | Noop backends; co-present blocks `llm`/`prompt=none` |
| Agent remote/directory | **Done** | Host-orchestrated `agent.process` + `FallbackAgentProvider` → builtin |
| Memory decay persist (`weight` / `accessed_at`) | **Done** | Wall-clock + immersive decay; ranked touch |
| `HostProfile [memory].retrieval` | **Done** | `default`=8 / `light`=4 |
| Remote LLM JSON-RPC client | **Done** | `remote_llm_jsonrpc_roundtrip.rs` |
| Remote LLM `process_message` E2E | **Done** | `remote_llm_process_message_roundtrip.rs` |
| dual_pipeline hints wiring | **Done** | `host_state_expression_hint` + `relation_transition_hint` |
| pack-editor config + user_identities UI | **Done** | RolePackEditorPanel JSON sections |
| Canonical re-export cleanup (Tauri P1) | **Done** | `src-tauri` imports → `oclive_kernel_host` / `oclive_kernel_types` |
| Host/runtime engine re-export (P3) | **Partial** | `#[deprecated]` + `check-host-reexport-imports.mjs` ratchet (78 baseline); remove block when ratchet → 0 |
| `ExplicitUnsupportedAgentProvider` dead code | **Done** | Removed from `agent.rs` |
| VS Code per_scene identity (`scene_set`) | **Done** | `kernelClient.setSceneUserIdentity` → `POST /user_identity/scene_set` |

This file tracks **mid/long-term** engineering debt, activation criteria, and batch status. Short-term slices live in [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md) and [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md).

---

## Opus 4.7 scan vs local verification (2026-05-20)

Cross-repo optimization scan (Opus 4.7) plus **local grep/build verification**. Status key: **Done** · **Corrected** · **Confirmed** · **Pending**.

| Finding | Opus claim | Verified | Status |
|---------|------------|----------|--------|
| `db.rs` ~36 `.unwrap()` on hot path | Production panic risk on `row.get` | **2×** `unwrap_or_else` in prod (RFC3339 fallback); rest in `mod tests` | **Corrected** → `parse_memory_created_at` + warn log; file split to `infrastructure/db/*` |
| `role_pack.rs` / `dual_pipeline.rs` unwrap | User-input panic | All in `#[cfg(test)]` | **Corrected** |
| `dual_pipeline_steps` `.expect("emotion_result set")` | Invariant panic | Replaced with `ProcessMessageError::dual_core_invalid` | **Done** |
| `plugin_host.rs` 63× `.clone()` | Hot-path copy waste | Arc 热路径保留；消除 `PluginBackends` / 全量 `provider_id` 克隆 | **Superseded**（见下行 L59 **Done**） |
| `tauri-api.ts` monolith | Hard to navigate / camelCase drift | Split → `src/api/{helpers,chat,role,settings,plugin,agent,diagnostics}.ts` + `toCamelPayload` | **Done** |
| `zh-CN.ts` / `en-US.ts` size | Linear bundle growth | Aggregators + `fragments/{app,settings,pluginManager,common,roleRuntime,editor}.*` | **Done** |
| `@vue-flow` first-screen | Lazy-load + chunk | `PluginManagerPanel` not in current `App.vue` entry; `manualChunks` + `defineAsyncComponent` ready | **Corrected** / chunk when V1 panel wired |
| `pluginStore` module `Map` memo | Multi-instance stale cache | Moved to `slotOrderMemoBySlot` in store state | **Done** |
| `check:release` gaps | No `test:unit` / `verify:ui` | Chained in `package.json` | **Done** |
| Prod `console.*` | Not stripped | `esbuild.drop: ['console','debugger']` in prod | **Done** |
| `AppState` 14× `RwLock<HashMap>` | Lock contention | **4** session maps + `role_cache`; extracted `SessionCache` with per-field locks | **Done** (1× `role_cache` remains) |
| `db.rs` 1950 lines / 62 fns | Split by table | `db/{mod,long_term_memory,role_runtime,relation_state,session_state,plugin_state}.rs` + `RoleRuntimeRepo` | **Done** |
| Sister-repo i18n drift | Shared package | `src/i18n/shared/` + mirror sync + `verify:shared-i18n` | **Done** |
| `fuzz/` sparse | Add validation targets | `fuzz_oclive_validation`, `fuzz_function_call_parser`, `fuzz_role_pack_loader` | **Done** |
| `plugin_host.rs` clone audit | 63× `.clone()` | Arc 热路径保留；消除 `PluginBackends` / 全量 `provider_id` 克隆 | **Done** |
| CI `npm audit --omit=dev` | Visibility job | CI job `npm-audit` | **Done**（D-NPM-01） |

### Opus 4.7 second pass — build / perf / architecture (2026-05-20)

| # | Item | Status | Notes |
|---|------|--------|-------|
| 1 | `[profile.release]` `opt-level = "z"` | **Done** | `opt-level = 3`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`, `lto = true` |
| 2 | Tauri / reqwest feature tightening | **Done** | Workspace `reqwest` `default-features = false`, `json` + `rustls-tls` only; no `fs-*` / `blocking` in `src-tauri` / host manifests (2026-06-08 RC sweep) |
| 3 | SQLite WAL + pool 16 | **Done** | `infrastructure/sqlite_pool.rs`; `AppState::new` + tests |
| 4 | Split `App.vue` (`TopBarMorePanel`) | **Done** | `TopBarMorePanel.vue` + `useReturnFocusOnClose`; App.vue ~1100 lines |
| 5 | Plugin bridge script → static asset | **Done** | `plugin_protocol.rs` `include_str!(…/assets/plugin-bridge.iife.js)`; `npm run build:plugin-bridge` |
| 6 | `TurnContext` in `process_message` | **Done** | `domain/chat_engine/turn_context.rs`; co_present / remote / dual-core |
| 7 | `AppState` builder / policy extract | **Done** | `state/mod.rs` ~447 lines; `app_state_builder.rs`, `policy_registry.rs`, `session_backends.rs` |
| 8 | `load_role_cached` inflight map leak | **Resolved** | 2026-06-08 复核不可复现；见 D-OPUS-03 |
| 9 | `generate_handler!` grouping | **Done** | Domain comments in `lib.rs` `invoke_handler` |
| 10 | Dual prompt_builder module dedup | **Done** | SSOT: `crates/oclive_kernel_runtime/src/domain/prompt_builder/mod.rs` (+ `sections.rs`); tauri re-exports only |
| 11 | Vite `manualChunks` (i18n / pinia persist) | **Done** | `vendor-i18n`, `vendor-pinia-persist` |
| 12 | Tracing file sink / JSON | **Done** | `init_tracing_with_log_dir` + `OCLIVE_LOG_FORMAT=json` or `RUST_LOG` containing `json`; `--api` / `OCLIVE_LOG_DIR` rolling file |
| 13 | `Cache` read-lock + TTL | **Done** | read-first `get`, cap 1000, `Instant` TTL |
| 14 | `package.json` devDeps trim | **Corrected** | `webdriverio` + `acorn` in use; note in `e2e/tauri-native.spec.ts` |
| 15 | Split `e2e_init.rs` | **Done** | `e2e_init_{minimal,monolith,templates,legacy}.rs` + `tests/common/` |
| 16 | `tools/scan-source-sizes.ps1` | **Done** | Renamed from `_scan_sizes.ps1` |
| 17 | `clippy::await_holding_lock` deny | **Done** | Workspace `[workspace.lints.clippy]` |

**Prior batch (same day):** tauri-api split, i18n fragments, db split, `SessionCache`, plugin_host clone audit, shared i18n, fuzz targets — see rows above.

**Scripts (repeatable):** `scripts/split-tauri-api.mjs`, `scripts/migrate-tauri-api-imports.mjs`, `scripts/split-i18n-locales.mjs`, `scripts/split-db-rs.mjs`, `tools/scan-source-sizes.ps1`.

---

## Batch 4 summary (2026-05)

| Original ID | Item | Effort | Batch 4 decision | Status |
|-------------|------|--------|------------------|--------|
| **1.2** | A1.1c — installer / native Tauri window / full GUI E2E | Large infra | **Start foundation** | **Foundation started** — [`e2e/tauri-native.spec.ts`](../e2e/tauri-native.spec.ts), CI `e2e-tauri` (`continue-on-error`) |
| **1.5** | T05–T13 ~42 component cases (pack editor) | Large | **Phased supplement** | **Complete (critical path)** — **119** Vitest cases in `oclive-pack-editor`; T05–T13 mapped (see [OVERVIEW](../creator-docs/testing/OVERVIEW.md)) |
| **3.1** | `library` vs `kernel_server` capability parity | Architecture | **Defer — RFC** | **Deferred** — see §3.1 below |
| **3.5** | Multimodal / barge-in / multi-tenant | New product | **Defer — product** | **Deferred** — see §3.5 below |
| **3.6** | Reference hardware / docker-compose targets | Hardware | **Defer — resources** | **Deferred** — see §3.6 below |
| **3.7** | Edge OTA / remote ops | Infra at scale | **Defer — scale** | **Deferred** — see §3.7 below |
| **5.3** | Plugin market UGC (signing, moderation) | Product + legal | **Defer — ecosystem** | **Deferred** — see §5.3 below |

---

## 1.2 · A1.1c native GUI E2E (started)

| | |
|--|--|
| **Why not full coverage yet** | Installer signing, multi-OS WebDriver (Windows Edge driver, macOS gap), and flake control need dedicated pipeline work beyond one smoke spec. |
| **Activation criteria** | Stable `e2e-tauri` job green for N releases; release engineering owns signed installer matrix. |
| **Current work** | Minimal WebDriver smoke: window title + `.left-pane` + role selector; `tauri-driver` + `xvfb` on Ubuntu CI (`continue-on-error: true`). |
| **Artifacts** | [`e2e/tauri-native.spec.ts`](../e2e/tauri-native.spec.ts), [`scripts/e2e-tauri-native-ci.sh`](../scripts/e2e-tauri-native-ci.sh), CI job **`e2e-tauri`**. |

---

## 1.5 · T05–T13 component tests (phased)

| | |
|--|--|
| **Why not 42 cases at once** | Full tree duplicates pack-editor UI churn; ROI higher on **contract + critical editor paths** first. |
| **Activation criteria** | Pack-editor Vitest ≥ **20** cases mapped to T05–T08; remaining T09–T13 when studio UX stabilizes. |
| **Status (2026-05-20)** | **T05–T13 critical path complete** — **119** Vitest tests in `oclive-pack-editor` (`npm run test` green). |
| **Authority** | [creator-docs/testing/OVERVIEW.md](../creator-docs/testing/OVERVIEW.md) §T05–T13 table. |

---

## 3.1 · `library` shape capability asymmetry

### 预留设计

| | |
|--|--|
| **预留原因** | Full `process_message` orchestration still lives in the Tauri host; forcing it into `library` would fork host vs embedded orchestration and double maintenance. |
| **当前已有的拓展基础** | `oclive_kernel_types` and `oclive_kernel_contracts` are decoupled from the host and usable in `library` builds; `oclive init --project-type library` scaffolds a library project. |
| **未来启动注意事项** | Migrate orchestration to `kernel_runtime` first (**single source of truth**); then expose a symmetric `library` API per RFC. |

| | |
|--|--|
| **Why deferred** | Full orchestration still lives in the Tauri host / `kernel_server`; extracting a symmetric `library` API requires an **RFC** (trait surface, error model, async runtime ownership). |
| **Activation criteria** | Documented embedded customer use case (device firmware, headless daemon) with acceptance tests; RFC approved. |
| **What we can do now** | [PURE_KERNEL_BOUNDARY.md](../creator-docs/getting-started/PURE_KERNEL_BOUNDARY.md) documents current capability boundary; `oclive-cli` / `headless-kernel-minimal` for `--api` path. |

---

## 3.5 · Multimodal / barge-in / multi-tenant

### 预留设计

| | |
|--|--|
| **预留原因** | Current architecture is **text-turn** centric; multimodal needs a new input pipeline and slot contracts—premature engineering would lock evolution. |
| **当前已有的拓展基础** | Blueprint `slot_registry` accepts arbitrary types; experimental kernels can load non-six-slot modules; `pipeline.experimental` can customize orchestration order. |
| **未来启动注意事项** | Do **not** hard-code multimodal input into `process_message`; register new **slot types** via `slot_registry`. |

| | |
|--|--|
| **Why deferred** | New product surfaces (audio stream, half-duplex interrupt, tenant isolation) need **product decision + PoC** before kernel changes. |
| **Activation criteria** | Signed PRD with MVP boundary vs `send_message` orchestration; security review for multi-tenant keys/memory namespaces. |
| **What we can do now** | Blueprint / `plugin_backends` / directory plugins reserve extension slots; no kernel API commitment yet. |

---

## 3.6 · Reference hardware / docker-compose targets

### 预留设计

| | |
|--|--|
| **预留原因** | Hardware purchase and lab setup need a target platform and budget; early hardware choices may age out. |
| **当前已有的拓展基础** | CI **ARM64 cross-compile** (`aarch64-unknown-linux-gnu`); `oclive init --template headless-api` for headless kernel projects. |
| **未来启动注意事项** | First target: **common ARM64 Linux SBC** in the community; validate boot + basic chat before exotic SoCs. |

| | |
|--|--|
| **Why deferred** | Requires **hardware purchase**, lab network, and repeatable device images beyond cross-compile CI. |
| **Activation criteria** | Hardware partner or budget; target SoC + OS matrix signed off. |
| **What we can do now** | ARM64 cross-compile smoke in CI (`rust-arm64-cross`); doll-core / deployment docs for school deployments. |

---

## 3.7 · Edge OTA / remote operations

### 预留设计

| | |
|--|--|
| **预留原因** | OTA design is highly deployment- and hardware-specific; early design may not match real fleet needs. |
| **当前已有的拓展基础** | Kernel serves **`--api` HTTP**; OTA can be a **sidecar plugin** or separate process without touching `process_message`. |
| **未来启动注意事项** | Implement OTA as an **independent sidecar** talking to the kernel via directory-plugin protocol—do not write OTA into orchestration. |

| | |
|--|--|
| **Why deferred** | OTA and fleet ops pay off at **scale**; current user base is desktop-first early adopters. |
| **Activation criteria** | Stable release channel + enough field devices to justify update signing, rollback, and audit. |
| **What we can do now** | Sidecar / Remote plugin protocol documented; no fleet controller in product. |

---

## 5.3 · Plugin market UGC (signing, moderation, malicious packs)

### 预留设计

| | |
|--|--|
| **预留原因** | Market content moderation needs **product, legal, and ops**—a pure-tech solution cannot decide policy alone. |
| **当前已有的拓展基础** | **`high_risk_grants`** permission system; manifest **`permissions`** validation; **`pack validate`** for package shape. |
| **未来启动注意事项** | Add content scanning and user reports **on top of** the permission system—do not rebuild grants from scratch. |

| | |
|--|--|
| **Why deferred** | UGC needs **product, legal, and ops** (moderation queue, DMCA, malware response) before opening uploads. |
| **Activation criteria** | Market index traction (content + active users); trust model (signing, publisher identity) agreed. |
| **What we can do now** | High-risk capability grants + manifest permission validation; curated git index + local zip install. |

---

## Verification (batch 4 closure)

**2026-05-20 — passed locally:**

| Check | Result |
|-------|--------|
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ zero warnings |
| `cargo test --workspace --lib` | ✅ 127 passed |
| `npm run test:unit` (oclivenewnew) | ✅ 23 passed |
| `npm run build` (oclivenewnew) | ✅ success |
| `npm run test` (oclive-pack-editor) | ✅ **119** passed (T05–T13 complete) |

Update this file when batch status changes.

---

## 冻结决定（2026-06-01）· dual_core / blueprint v3 / expert_routing

**背景与判据**：定位对齐（见 [OCLIVE_POSITIONING_DIFFERENTIATION.md](./OCLIVE_POSITIONING_DIFFERENTIATION.md)）后确认，以下三项属 **speculative generality（为尚不存在的需求提前建造）**。统一判据：

> **凡「默认关 / 主路径不调用 / 文档写了代码没有」= 过早建造。处置 = 冻结（保留代码、停止投入、明确标注未启用），而非删除。** 注意力收回到有真实牵引的两件事：**VSCode 滩头** 与 **「两个 OC 互动」原语**。冻结项将来作为 **重磅更新** 择机解冻发布。

| 项 | 现状（已核实） | 冻结含义 | 解冻条件（→ 可作重磅更新） |
|----|----------------|----------|------------------------------|
| **双核双态 `dual_core`** | `dual_pipeline*.rs` ~970 行**全在 `#[cfg(feature="dual_core")]` 下，默认发行版不编译**；唯一入口 `role.dual_core_gated()` + feature。**已是真冻结，零成本。** | 无需任何操作；勿再投入。比较类需求优先用**离线 bench/eval harness**，不要把双管道烤进运行时热路径。 | 出现**真实的第二个成熟实现**需要在线灰度 A/B 时；开 feature 即重磅发布。 |
| **Blueprint v3 + `runtime_config`** | 加载器 dispatch v2/v3 双 schema；v3 为草案，主载荷（`dual_core`/`runtime_config`）大多默认关。**默认编译，带维护成本。** | 冻结 = **v3 别再长**；**v2 仍为 SSOT**；不新增 v3 字段/迁移。 | v2 **真的**无法表达某个真实需求时再扩 v3。 |
| **专家路由 `expert_routing`（407 行）** | `expert_routing.rs` 顶部 `#![cfg(feature = "dual_core")]`——**与双核同一 flag，默认不编译，已是真冻结、零默认成本**；常规回合管道不触发。复用 slot 抽象（过度程度较轻）。 | 无需操作（随 `dual_core` 一并冻结）；勿再投入。 | 随 `dual_core` 解冻，或出现**真实「按意图路由到不同专家模型」需求**时。 |

**未冻结、已翻案（保留）**：三套存储后端 `hybrid/file/sqlite` —— 在「可嵌入内核（无头/嵌入式）」论题下 `file`/`sqlite` 单后端合理；**生产路径为 hybrid**；`file`/`sqlite` **保持可编译 + 最小测试即可，不再扩展新功能**（见 [CHAT_STORAGE_ARCHITECTURE.md](./CHAT_STORAGE_ARCHITECTURE.md) §Investment boundary）。

**叙事对齐（2026-06-08）**：对外文档对 **dual_core / blueprint v3 / expert_routing** 统一表述为 **「机制已预埋，默认关闭；解冻条件见本表」**——代码冻结不变，保留平台可扩展叙事。见 [OCLIVE_POSITIONING_DIFFERENTIATION.md](./OCLIVE_POSITIONING_DIFFERENTIATION.md)。

### Dimension 5 closure（工程纪律，2026-06-08）

| ID | Item | Phase | Status |
|----|------|-------|--------|
| D-CI-01 | workspace clippy/test in CI | 1 | **Done** |
| D-CI-02 | e2e-kernel-profile in CI | 1 | **Done** |
| D-CI-03 | cargo-audit lockfile gate | 1 | **Done** (`cargo-audit-lockfile.yml`) |
| D-LAYER-01 | domain→infra ratchet | 2 | **Done** (`scripts/check-domain-layering.mjs`) |
| D-LAYER-02 | CODEOWNERS frozen paths | 2 | **Done** |
| D-PORT-01 | DEFAULT_API_PORT SSOT | 2 | **Done** (`oclive_kernel_runtime::DEFAULT_API_PORT`) |
| D-HONEST-01 | remote placeholder user-visible | 2 | **Done** (`startup_health` warnings) |
| D-VSCODE-01 | vscode CI | 3 | **Done** |
| D-VSCODE-02 | EnsureReport golden | 3 | **Done** |
| D-SSOT-01 | DTO/schema naming doc | 4 | **Done** |
| D-POLICY-01 | policy trait second impl or collapse | 4+ | **Deferred**（保留 trait；第二实现 = 角色包 policy / remote，未排期；连续两发版无第二实现则评估 collapse） |
| D-NPM-01 | npm audit CI visibility | 6 | **Done** |
| D-SIZE-01 | prompt_builder / http_api split | 5 | **Done** |
| D-FREEZE-01 | dual_core / monolith PR 须引用 RFC | 2 | **Done**（`.github/CODEOWNERS` + 本表） |

### Opus 4.8 审查摘要（2026-06-08）

| ID | Item | Status |
|----|------|--------|
| D-LAYER-03 | `plugin_host` factory port (`PluginBackendRegistryPort`) | **Done**（ratchet 22→8；见 [LAYERING_BASELINE.json](./LAYERING_BASELINE.json)） |
| D-LAYER-04 | 生产路径剩余 4 处 domain→infra（`ComplexEmotionHintStore` / `VirtualTimeStore` / `UserLlmSecretsPort` / env check） | **Done**（ratchet 8→4；仅 `#[cfg(test)]` Mock/test_db） |
| D-SIZE-02 | `directory_plugins/runtime` split (`transport` / `spawn` / `rpc`) | **Done** |
| Dimension 5 验收门 | `scripts/dimension5-acceptance.mjs` + [DIMENSION5_CLOSURE_SIGNOFF.md](./DIMENSION5_CLOSURE_SIGNOFF.md) | **Done** |
| OOCP `startup_warnings` | S0b in `examples/oocp-test-suite/run.mjs` | **Done** |
| `oclive_sqlx` 文档 + lockfile guard | [crates/oclive_sqlx/README.md](../crates/oclive_sqlx/README.md) | **Done** |

未纳入本维度项见 §Opus 4.8 Deferred。

### Opus 4.8 follow-up 审查（2026-06-08，五维复审）

DeepSeek 五维方向复审 + Opus 4.8 计划收尾。维度五基线：`node scripts/dimension5-acceptance.mjs --ci` → **PASS (7 checks)**（layering ratchet / cargo audit / lockfile / ensure-plan / CHANGELOG parity / host re-export ratchet；`--ci` 跳过抽样 cargo test）。所有 D-CI/D-LAYER/D-HONEST 未回退。

| ID | 项 | 维度 | Status | 备注 |
|----|-----|------|--------|------|
| K-PERF-01 | `persist_memory_decay_batch` N+1 写 | 一·运行时 | **Done** | 单事务批量提交；`memory_decay_persist` + host `--lib db`（14）绿 |
| K-PERF-02 | 热路径 stage tracing | 一·运行时 | **Done** | `staged.rs` target `oclive_turn` + `elapsed_ms`；采样见 PERFORMANCE.md §6 |
| K-DOC-01 | `CHANGELOG.en.md [Unreleased]` 落后中文版 | 四·文档 | **Done** | 英文镜像补齐 |
| K-DOC-02 | CHANGELOG `[Unreleased]` CI 门 | 四·文档 | **Done** | `scripts/check-changelog-parity.mjs` → dimension5 |
| K-PROFILE-01 / D-OPUS-06 | 双 TOML 解析统一 | 二/三 | **Done** | SSOT `distro_oclive_file.rs`；RFC [RFC_PROFILE_AND_DOMAIN_REEXPORT.md](../creator-docs/rfc/RFC_PROFILE_AND_DOMAIN_REEXPORT.md) |
| D-OPUS-05 | Host/runtime `pub use` 去重 | 二 | **Partial** | ratchet **77**（Wave 3：`pre.rs` MemoryEngine）；Phase 2 import 迁移 Deferred |
| D-OPUS-01/02/04 | 发版前轻量化 | 一 | **Done** | reqwest features / plugin bridge 静态资源 / JSON tracing sink（见 Opus 4.7 表 #2/#5/#12） |
| D-OPUS-03 | `load_role_cached` inflight 泄漏 | 一/三 | **Resolved** | 双路 `role_load_inflight.remove` + `turn_locks` 软上限 |

**仍 Deferred（长期，不阻塞滩头）**：D-POLICY-01（policy trait 第二实现 or collapse）；D-OPUS-05 Phase 2（re-export import 清零）；§3.1 `library` 对称 API；K-PROFILE-04 安装包 bundled profile。姊妹仓文档 sweep 见 [SISTER_REPO_DOC_SWEEP.md](./SISTER_REPO_DOC_SWEEP.md)。

**设计维度结论**：`oclive_kernel_contracts` 下 ~24 个 trait 均为正当 DI 端口（`Arc<dyn>` 经 `AppState::*_for` / `domain/ports` 注入，含测试 Mock 替换面），无需降级为具体类型；单实现的 `EmotionPolicy`/`EventPolicy`/`MemoryPolicy` 已由 D-POLICY-01 跟踪。

### Opus 4.8 Deferred（2026-06-08）

不在 Dimension 5 / Opus 4.8 工程纪律维强做的项；避免 Pending 悬空。

| ID | 项 | 决定 |
|----|-----|------|
| D-OPUS-01 | Tauri/reqwest feature 收紧 | **Done**（2026-06-08 RC sweep；见 Opus 4.7 表 #2） |
| D-OPUS-02 | Plugin bridge 内联 JS → 静态资源 | **Done**（见 Opus 4.7 表 #5） |
| D-OPUS-03 | `load_role_cached` inflight 泄漏 | **Resolved**（不可复现；见上表） |
| D-OPUS-04 | Tracing JSON 文件 sink | **Done**（`OCLIVE_LOG_FORMAT=json`；见 Opus 4.7 表 #12） |
| D-OPUS-05 | Host/runtime `pub use` 去重 P3 | **Partial**（RFC 已落地 Phase 1：deprecated + ratchet 78；Phase 2 import 迁移 Deferred） |
| D-OPUS-06 | K-PROFILE-01 双 TOML 解析统一 | **Done**（合并入 K-PROFILE-01 / `distro_oclive_file`） |
| D-POLICY-01 | policy trait second impl or collapse | **Deferred**（见上表 Dimension 5） |

**配套动作（owner：作者本人）**：(1) **统一文档**一次，将 roadmap（未做）与 status（已做）明确分开 — **Done**（[DOCUMENTATION_INDEX.md](../creator-docs/getting-started/DOCUMENTATION_INDEX.md) §工程纪律 / 审查状态）。注：经核实 `oclive_kernel_server` **确实存在且可跑**（`[[bin]] oclive-kernel-server`），先前「文档有、代码无」判断为搜索假象（目录名 `oclive_kernel_server`，非 `kernel_server`）；其 `Cargo.toml` 仅依赖 **`oclive_kernel_host` + `oclive_kernel_runtime`**（**不**依赖 `oclivenewnew-tauri`，旧表述已更正），编排经 `oclive_kernel_host::run_api_server`。**真正待澄清的窄点**：编排（`process_message`）仍内嵌于 `oclive_kernel_host` 并与其 `infrastructure` 耦合（尚未抽到 host-independent 纯内核 `library`，见 §3.1），文档措辞应区分「无头发行版已存在」与「纯内核 library 待抽离」。(2) 之后**重心转向宣传与滩头**，停止过度拓展。

### Opus 4.8 五维复审追加（2026-06-08，第二轮）

DeepSeek 五维方向二轮复审（Opus 4.8）。维度五基线复跑：`node scripts/dimension5-acceptance.mjs --ci` → **PASS (7 checks)**，所有 D-CI/D-LAYER/D-HONEST 未回退。第二轮以**核对 + 据实入库**为主；第三轮（落实审查）完成 PR-A/B/C/D/E 主体实现与文档同步。

**本轮已修复 / 已实现（Done）**

| ID | 项 | 维度 | 备注 |
|----|-----|------|------|
| K-PERF-09 | `App.vue` 同时静态导入 `FluentShell` + `ToolShell`（仅一个渲染） | 一·前端 | 改 `defineAsyncComponent` 动态导入；`npm run test:unit` 45/45 绿 |
| K-DOC-03 | `EXTENSION_POINTS.md`（中/英）引用不存在的 `domain/policy.rs` | 四·文档 | trait → `oclive_kernel_contracts/src/policy.rs`；impl → `oclive_kernel_runtime/src/domain/policy.rs`；wiring → `infrastructure/policy_registry.rs` |
| K-DOC-04 | 「`oclive_kernel_server` 仍链接 `oclivenewnew-tauri`」表述过期 | 四·文档 | 实际仅依赖 host + runtime；本表 L315 与 `OCLIVE_POSITIONING_DIFFERENTIATION.md` 已更正 |
| K-DOC-05 | `domain/README.md` adapter 清单过期（未含 FQ-path 现状） | 四·文档 | 同步为 `use`-import (4, 全 test) + 生产 FQ-path (5) + turn ports；见 `domain/README.md` |
| D-LAYER-05 | layering ratchet FQ-path + turn ports | 二/五 | `check-domain-layering.mjs` + `LAYERING_BASELINE.json`；`ChatTurnPersistencePort`/`TurnPoliciesPort`/`ConversationPersistPort`；FQ **14→5** |
| D-DTO-01 | reply-post-processor 配置去重 | 二 | 统一 `ReplyPostProcessorEffectiveConfig`；`resolve_builtin` SSOT 于 `domain/reply_post_processor.rs`（2026-06-09 去重 wiring 副本） |
| D-ERR-01 | profile / MCP 增量 `AppError` | 二 | **Done** | `host_profile_from_distro_file`、`PluginHost::call_mcp_tool`；`user_llm_env`/`startup_health` → `AppSettingsPort`/`DbHealthPort`（Wave 1） |
| K-PERF-03 | TurnPrefetch 共享 / agent lazy | 一·运行时 | `turn_prefetch.rs` + `agent=none` 跳过 agent DB |
| K-PERF-04 | `role_runtime` 单查快照 | 一·运行时 | `get_role_runtime_snapshot` 扩展字段 + `TurnContext` |
| K-PERF-05 | session 配置一次解析 | 一·运行时 | `EffectiveSessionConfig` |
| K-PERF-06 | memory decay 单事务 | 一·运行时 | rank 后一次 `persist_memory_decay_batch` |
| K-PERF-07 | SessionCache 淘汰 | 一·运行时 | cap 512 + TTL + turn_lock 联动 |
| K-PERF-08 | personality_vector 索引 | 一·运行时 | migration `033_personality_vector_index.sql` |
| K-PERF-11 | kernel status 轮询退避 | 一·前端 | hidden 60s |
| K-PERF-12 | 微优化打包 | 一·运行时 | hybrid_store / Arc Role / replay / role_cache / probe |
| K-DOC-06 | `simplePluginManager.slots.*` i18n | 四·文档/前端 | fragments 已齐 + parity 测覆盖 |

**Partial**

| ID | 项 | 维度 | 备注 |
|----|-----|------|------|
| K-PERF-10 | 壳内面板懒加载 | 一·前端 | `useMainShell` async 导出 Settings/RoleDetail/SceneTravel 等 overlay；`FluentShell`/`ToolShell` 内 ChatInput、MessageList、KernelStatusBar 等 ~15 组件仍静态 import |

**仍 Deferred（按工作量）**

| ID | 项 | 维度 | 工作量 | 现状/证据 |
|----|-----|------|--------|-----------|
| K-CONTRACT-WIRING-01 | `PromptInput.extra_sections` 生产路径未接线 | 二 | M | V-CONTRACT Phase 0 类型已入库；host 仍传 `&[]` |
| D-POLICY-01 | Policy 三 trait 单实现 DI 评估 | 二 | M | 无功能缺陷；见 D-TRAIT-01 裁决 |

**Round 10 已关闭（勿与上表混读）**：D-PORT-02、D-SLOT-01、K-PROFILE-04 见 §Round 10 patrol **Done**。

**结论**：Opus 4.8 主体已落地（热路径 DB 合并、SessionCache 淘汰、turn ports、FQ ratchet 14→5）；维度五 gate PASS。**收尾验收（2026-06-08）**：PR-C1→PR-E 五 commit 已入库；见 header Verification。**未阻塞滩头**的后续项：K-PERF-10 chat chrome lazy、D-LAYER-05b `post.rs` PolicySet 端口化、`user_llm_env`/`startup_health` 剩余 FQ refs、K-CONTRACT-WIRING-01。

---

## 巡检债 Wave（Opus 4.8 工程夯实轨，2026-06-09）

与 [RECURRING_OPTIMIZATION_PLAYBOOK.md](./RECURRING_OPTIMIZATION_PLAYBOOK.md) 半档/全档巡检正交；**不替代** Theater / desktop 发行版功能开发。每波结束更新 §8 日志与本表状态。

| Wave | 窗口 | 目标 | 退出标准 |
|------|------|------|----------|
| 0 | 第 1 周 | 机制就位 + 债项登记 | 本表 + §8 轮次 1 |
| 1 | 第 2–3 周 | A 档补漏 + Theater 测试护栏 | D-ERR-01 余量、K-PROFILE-04、theater 验收测绿、半档 PASS |
| 2 | 第 4–5 周 | 可观测 + 性能预算 | poke 延迟有数字、health 警告首屏可见、D-CLEAN-01 |
| 3 | 第 6–7 周 | 文档对拍 + re-export 切片 | ratchet 78→≤77、CREATOR_GOLDEN_PATH 大纲 |
| 4 | 第 8 周（条件） | 陌生人测试后全档 + Phase 5 解冻评估 | [THEATER_STRANGER_TEST_ROUND1.md](./THEATER_STRANGER_TEST_ROUND1.md) 汇总后触发 |

### Wave 轨新增债项（愿景类 · V-*）

| ID | 项 | 档 | Status | 位置 / 约束 |
|----|-----|-----|--------|-------------|
| **V-THEATER-PERF-01** | 剧场戳点 E2E 延迟预算：`probe → patch → 首条新台词` 分段计时 | B | **Done** | `useTheaterBeatPatch.ts` performance.mark + [PERFORMANCE.md](../creator-docs/getting-started/PERFORMANCE.md) §7 |
| **V-SLOT-HONEST-01** | remote 缺 env 时 UI/health `startup_warnings` 首屏可见（强化 D-HONEST-01） | B | **Done** | `StartupWarningsBanner.vue` + `GET /health` JSON；不改槽解析 |

**Wave 1–3 已落地（2026-06-09）**：D-ERR-01 余量（AppSettingsPort / DbHealthPort）、K-PROFILE-04 bundled profile、Theater 验收测试（无 Ollama / patch 降级）、V-THEATER-PERF-01、V-SLOT-HONEST-01、D-CLEAN-01、D-OPUS-05 切片（77）。

**Wave 4 解冻评估（2026-06-09，条件未满足）**

| 条件 | 状态 | 本轨结论 |
|------|------|----------|
| [THEATER_STRANGER_TEST_ROUND1.md](./THEATER_STRANGER_TEST_ROUND1.md) 5–10 人 | **未执行** | 不触发 C 档 thaw |
| 首屏 perf 失败 | 无数据 | K-PERF-10 维持 Partial |
| 可替换性反馈差 | 无数据 | D-SLOT-01 维持 Deferred |
| Phase 5 表逐项评估 | — | **C 档开工 0**；dual_core / expert_routing **维持冻结** |

全档巡检（轮次 3）基线 PASS；待陌生人测试 ≥60% 通过后复评 Phase 5。

### VS Code 发行版（姊妹仓 `oclive-vscode` · 2026-06-10）

| ID | Item | Status | Notes |
|----|------|--------|-------|
| **V-VSCODE-IA-01** | 双 webview 范式（内联 Chat HTML vs Svelte Settings） | **Done** | 单一 Svelte `App.svelte` + `view` 路由；`getShellHtml` 已移除 |
| **V-VSCODE-PERF-01** | `ensureReady` 每 API 调用重跑 discover/cli/health | **Done** | 5s TTL + in-flight 去重；`reconnectKernel`/失败时 `invalidateEnsureReady` |
| **V-VSCODE-PERF-02** | 设置快照串行 6 连发 | **Done** | `buildStateSnapshot` `Promise.all` 并行只读请求 |
| **V-VSCODE-PERF-03** | 角色快照轮询侧栏隐藏仍 15s | **Done** | `onDidChangeVisibility`：可见 15s / 隐藏 60s（对齐 K-PERF-11） |
| **V-VSCODE-HONEST-01** | `penetration.*` / `portraitMaxHeight` 占位配置 | **Done** | schema 移除；原高级区折叠已删（见 HONEST-02） |
| **V-VSCODE-PERF-04** | Chat 对话流每 patch 全量 innerHTML | **Done** | `appendLines` 增量 + Svelte `{#each}` |
| **V-VSCODE-PERF-05** | F5 实机 / `.vsix` 发布验收 | **Pending** | 见 `oclive-vscode/ROADMAP.md` |
| **V-VSCODE-FIX-01** | 设置内即时/连点切角色 → 插件卡死 | **Done** | `SettingsController.handleMessage` 经 `serialQueue` 串行化；`switchRoleInFlight` guard 全程保持（含尾部 pushState）；`handleSelectRole` 去重 pushState；切角色仅在设置面可见时跑快照 |
| **V-VSCODE-FIX-02** | 模型调用不稳（连接抖动打断内核 → fallback） | **Done** | `ensureReady` 决策抽到纯函数 `ensureReadyPolicy`（trust/revalidate/replan）：健康连接不再整轮重规划、mock 模式不再反复杀端口重启；连接相关设置改动 `invalidateEnsureReady` |
| **V-VSCODE-UI-01** | 角色切换下拉栏过亮 | **Done** | `.role-select` / 共享 `Select` 改用 `--vscode-dropdown-*` 主题色 + 常规字重 + focusBorder 描边（Cursor 风格） |
| **V-VSCODE-QA-01** | 纯逻辑无单测 | **Done** | `scripts/test-unit.mjs` 覆盖 `ensureReadyPolicy` + `serialQueue`；`npm run test:unit`（tsc + node） |
| **V-VSCODE-IA-02** | 设置页与 Chat 顶栏重复角色切换 | **Done** | `RoleSection` 只读包信息；切角色仅 Chat `.role-select` / QuickPick |
| **V-VSCODE-LAND-01** | `autoDiscover` 无即时触发入口 | **Done** | 内核区「重新发现…」→ `rediscover` → `applyAutoDiscovery(forcePrompt)` + `ensureReady(force)` |
| **V-VSCODE-HONEST-02** | 高级区「实验性（未实现）」死占位 | **Done** | 移除 `Collapsible`；渗透说明仅保留 `ROADMAP.md` |
| **V-VSCODE-LATENCY-01** | Chat 无取消/冷启动体感差 | **Done** | 停止按钮 + 计时/8s 提示 + `warmupModel` + `oclive.chat.warmup*` |
| **V-VSCODE-UNDO-01** | 无撤回/编辑/重生成 | **Done** | 四形态 + `meta_action_templates` + `/chat/storage`；删记录不回退记忆（tooltip/文档） |
| **V-VSCODE-STREAM-01** | 无 SSE 流式 | **Done** | 内核 `POST /chat/stream`（Gate：[VSCODE_STREAM_THEATER_GATE.md](./VSCODE_STREAM_THEATER_GATE.md)）；`chatStream` + `oclive.chat.streaming` |
| **V-VSCODE-IMMERSE-01** | 环境调优文档 | **Done** | ROADMAP 渗透节 + 冷启动 hint；Q4/GPU/keep_alive 见 PERFORMANCE 交叉引用 |
