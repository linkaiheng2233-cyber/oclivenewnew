# Technical debt inventory

**Last updated:** 2026-06-09 (freeze-safe audit + Opus 4.8 Theater v0)

**Product freeze (Theater v0):** No new kernel orchestration / six-slot expansion until strangers validate AI Theater v0. See [PRODUCT_FREEZE_THEATER_V0.md](./PRODUCT_FREEZE_THEATER_V0.md). **Deferred unchanged:** D-PORT-02, D-SLOT-01, K-PERF-10, K-PERF-14/15, D-NAME-01, §3.1 library API, dual_core (frozen).

**Verification (2026-06-09 freeze-safe audit):** `cargo build -p oclive_kernel_server`; `cargo test -p oclive_kernel_host`; `node scripts/check-domain-layering.mjs` — layering ratchet 4/5 无回退。Prior (2026-06-08): `node scripts/dimension5-acceptance.mjs --ci` PASS (7 checks); `cargo test -p oclive_kernel_host --lib` 180 passed; `npm run test:unit` 46 passed.

### Freeze-safe audit（2026-06-09）

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-PERF-13 | `http_api_roles` 无界 DashMap（`--api` 长跑） | **Done** | `insert_http_api_role` FIFO cap 32（对齐 `role_cache`） |
| K-BUILD-01 | `kernel_server/build.rs` `SystemTime::now()` 致增量编译失效 | **Done** | `SOURCE_DATE_EPOCH`（缺省 0）+ `rerun-if-env-changed` |
| K-PERF-14 | `pre_llm` 独立 await 串行 | **Deferred** | 触碰编排；冻结至 Theater v0 |
| K-PERF-15 | 记忆候选池固定 10 条按时间非相关性 | **Deferred** | 影响召回语义；冻结 |
| D-CLEAN-01 | `ReplayTaskRegistry` 完成不清理 | **Open** | 非热路径；可选后续 |
| D-NAME-01 | `resolve_turn` 三义命名消歧 | **Deferred** | 触 dual_core 冻结路径 |
| K-DOC-07 | `AGENTS.md` / `LIGHTWEIGHT_PROFILE` cargo-audit `continue-on-error` 漂移 | **Done** | 更正为 dimension5 + `cargo-audit` job + lockfile workflow 三层硬门禁 |

**Opus 4.8 follow-up (2026-06-08):** five-dimension re-review — `node scripts/dimension5-acceptance.mjs --ci` PASS; K-PERF-01 batched memory-decay writes; K-PERF-02 stage tracing + PERFORMANCE.md §6; K-DOC-02 CHANGELOG parity CI; K-PROFILE-01 unified `distro_oclive_file`; D-OPUS-05 re-export ratchet; D-OPUS-01/02/04 RC lightweight sweep Done. See §Opus 4.8 follow-up.

**Opus 4.8 second pass (2026-06-08):** five-dimension re-review #2 — gate re-run PASS (7 checks), no regressions. Fixed (zero-risk): K-PERF-09 lazy shells, K-DOC-03/04 doc drift. **Implemented (PR-A/B/C/D/E matrix):** D-LAYER-05 (FQ 14→5 + turn ports), D-DTO-01, D-ERR-01 (incremental), K-PERF-03..08/11/12, K-DOC-05/06; K-PERF-10 **Partial** (overlay panels lazy, chat chrome still eager). Still Deferred: D-PORT-02, D-SLOT-01. See §Opus 4.8 五维复审追加.

### Kernel profile scheduling (2026-06-08)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| K-PROFILE-01 | Dual TOML parse (`kernel_distro_profile` vs `host_profile`) | **Done** | SSOT `oclive_kernel_runtime::distro_oclive_file`; RFC [RFC_PROFILE_AND_DOMAIN_REEXPORT.md](../creator-docs/rfc/RFC_PROFILE_AND_DOMAIN_REEXPORT.md) |
| K-PROFILE-02 | `/health` summary non-runtime | **Done** | `HostProfile::active_profile_summary()` SSOT |
| K-PROFILE-03 | `distro_id`-only weak compat | **Done** | Hash required without summary; else Unknown |
| K-PROFILE-04 | Desktop missing bundled `distro.oclive.toml` | **Partial** | `{resource}/distro.oclive.toml` + anchors; ship in installer TBD |
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
| D-OPUS-05 | Host/runtime `pub use` 去重 | 二 | **Partial** | `#[deprecated]` + `check-host-reexport-imports.mjs` ratchet（baseline 78）；全仓 import 迁移未排期 |
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
| D-ERR-01 | profile / MCP 增量 `AppError` | 二 | **增量 Done**：`host_profile_from_distro_file`、`PluginHost::call_mcp_tool`；`user_llm_env`/`startup_health` DbManager 留后续 |
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
| D-PORT-02 | `PluginBackendRegistryPort` 为 20+ 方法 god-port，唯一实现 `BackendRegistry` 纯转发 | 二 | L | `plugin_backend_registry.rs` + `backend_registry.rs:797-919`；建议收窄到 `PluginHost`/`SlotResolver` 真用面 |
| D-SLOT-01 | 各槽 Builtin V1/V2/Placeholder 并行实现，选择逻辑散落 `BackendRegistry` | 二 | M | `memory_retrieval`/`user_emotion_analyzer`/`prompt_assembler`/`event_estimator` 各有 `*V2` + `*Placeholder`；建议每槽收一份 builtin + 选择矩阵集中 |

**结论**：Opus 4.8 主体已落地（热路径 DB 合并、SessionCache 淘汰、turn ports、FQ ratchet 14→5）；维度五 gate PASS。**收尾验收（2026-06-08）**：PR-C1→PR-E 五 commit 已入库；见 header Verification。**未阻塞滩头**的后续项：D-PORT-02/D-SLOT-01 god-port 与槽合并、K-PERF-10 chat chrome lazy、D-LAYER-05b `post.rs` PolicySet 端口化、`user_llm_env`/`startup_health` 剩余 FQ refs。
