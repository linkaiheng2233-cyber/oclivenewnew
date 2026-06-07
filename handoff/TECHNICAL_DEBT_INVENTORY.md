# Technical debt inventory

**Last updated:** 2026-06-07 (Phase 2 legacy closure batch)

**Verification (2026-06-07, Phase 2 closure):** `cargo check --workspace`; `cargo test --workspace --lib`; `cargo test -p oclivenewnew-tauri --tests` (Windows: `-j 1` if parallel link hits paging-file limits); `cargo clippy --workspace --all-targets --all-features -- -D warnings` (Windows: `-j 1` if needed); `npm run test:unit` (oclivenewnew); `npm run compile` (oclive-vscode).

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
| Host/runtime engine re-export (P3) | **Pending** | `oclive_kernel_host::domain` still `pub use runtime::domain::*` |
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
| `plugin_host.rs` 63× `.clone()` | Hot-path copy waste | Mix of `Arc` vs owned; needs per-field audit | **Pending** |
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
| CI `npm audit --omit=dev` | Visibility job | Not started | **Pending** |

### Opus 4.7 second pass — build / perf / architecture (2026-05-20)

| # | Item | Status | Notes |
|---|------|--------|-------|
| 1 | `[profile.release]` `opt-level = "z"` | **Done** | `opt-level = 3`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`, `lto = true` |
| 2 | Tauri / reqwest feature tightening | **Pending** | `fs-*`, `blocking` still enabled |
| 3 | SQLite WAL + pool 16 | **Done** | `infrastructure/sqlite_pool.rs`; `AppState::new` + tests |
| 4 | Split `App.vue` (`TopBarMorePanel`) | **Done** | `TopBarMorePanel.vue` + `useReturnFocusOnClose`; App.vue ~1100 lines |
| 5 | Plugin bridge script → static asset | **Pending** | `lib.rs` inline JS |
| 6 | `TurnContext` in `process_message` | **Done** | `domain/chat_engine/turn_context.rs`; co_present / remote / dual-core |
| 7 | `AppState` builder / policy extract | **Done** | `state/mod.rs` ~447 lines; `app_state_builder.rs`, `policy_registry.rs`, `session_backends.rs` |
| 8 | `load_role_cached` inflight map leak | **Pending** | `Arc::strong_count` cleanup |
| 9 | `generate_handler!` grouping | **Done** | Domain comments in `lib.rs` `invoke_handler` |
| 10 | Dual `prompt_builder.rs` dedup | **Done** | SSOT: `crates/oclive_kernel_runtime/src/domain/prompt_builder.rs`; tauri re-exports only |
| 11 | Vite `manualChunks` (i18n / pinia persist) | **Done** | `vendor-i18n`, `vendor-pinia-persist` |
| 12 | Tracing file sink / JSON | **Pending** | |
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

**未冻结、已翻案（保留）**：三套存储后端 `hybrid/file/sqlite` —— 在「可嵌入内核（无头/嵌入式）」论题下 `file`/`sqlite` 单后端合理；保持「能编译 + 最小测试」即可，勿做花活。

**配套动作（owner：作者本人）**：(1) **统一文档**一次，将 roadmap（未做）与 status（已做）明确分开。注：经核实 `oclive_kernel_server` **确实存在且可跑**（`[[bin]] oclive-kernel-server`），先前「文档有、代码无」判断为搜索假象（目录名 `oclive_kernel_server`，非 `kernel_server`）；**真正待澄清的窄点**是它仍链接 `oclivenewnew-tauri` 取编排（编排尚未抽到 host-independent 纯内核 `library`，见 §3.1），文档措辞应区分「无头发行版已存在」与「纯内核 library 待抽离」。(2) 之后**重心转向宣传与滩头**，停止过度拓展。
