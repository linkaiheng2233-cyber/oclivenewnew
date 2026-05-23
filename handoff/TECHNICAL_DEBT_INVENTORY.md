# Technical debt inventory

**Last updated:** 2026-05-20 (batch 4 assessment)

This file tracks **mid/long-term** engineering debt, activation criteria, and batch status. Short-term slices live in [PRODUCT_LINE_TASK_BUCKETS.md](./PRODUCT_LINE_TASK_BUCKETS.md) and [PRODUCT_AND_KERNEL_GAP_CHECKLIST.md](./PRODUCT_AND_KERNEL_GAP_CHECKLIST.md).

**Related (not duplicated here):** five-repo workspace hygiene (gitignore, binaries, `.env`, test gaps) — [docs/WORKSPACE_OPTIMIZATION_REPORT.md](../docs/WORKSPACE_OPTIMIZATION_REPORT.md) with cross-links to [ARCHITECTURE_LAYERING.md](./ARCHITECTURE_LAYERING.md) §Cursor 优化轮.

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
