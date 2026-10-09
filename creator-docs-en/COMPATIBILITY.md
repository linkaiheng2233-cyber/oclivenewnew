# Pack editor (oclive-pack-editor) ↔ host (oclivenewnew) compatibility

[中文](../creator-docs/COMPATIBILITY.md)

This page explains how **`ui.json` in role packs** relates to the **desktop host**, so authors do not ship fields the host ignores—or miss fields the host already supports.

**Version format**: both repos use **SemVer** `MAJOR.MINOR.PATCH` from root **`package.json`** `version`.

**Snapshot (aligned with release review)**:

- **oclivenewnew** (desktop / Tauri host): **`0.5.2`** (root `package.json` and `distros/desktop-tauri/Cargo.toml` must match)
- **oclive_kernel_runtime** (shared contract crate): **`0.2.0`** (`kernel/crates/oclive_kernel_runtime/Cargo.toml`; DTO / `API_VERSION` live in that crate)
- **oclive_kernel_host** (complete host and in-process facade crate): **`0.2.0`**. `OcliveKernel` is the supported Rust integration entry; internal `AppState` is not a compatibility contract.
- **oclive-cli** (scaffold CLI): **`0.1.0`** (`kernel/crates/oclive-cli/Cargo.toml`; **independent semver**, not required to match the desktop host; when scaffolding with `init --kernel-source`, path deps align contracts). **Default build** depends on `oclive_kernel_runtime` + `oclive_validation` only (`cargo tree -p oclive-cli --no-default-features` has **no** `libsqlite3-sys` / `axum`). **`doctor config-resolve`** defaults to runtime pure resolution; **`--via-host`** (feature **`diagnostics-host`**) optionally runs in-memory `AppState` deep diagnosis.
- **oclive-pack-editor** (sister repo): **`0.5.1`** (`ui.json` parity with host **0.5.x**)
- **oclive-vscode** (VS Code extension, sister repo): **`0.5.0`** (independent semver; spawn/attach contract needs host **≥0.4.0**, **0.5.2** recommended)

---

## Compatibility matrix

| Editor version | Minimum host | New or required `ui.json` capability | Notes |
|----------------|--------------|--------------------------------------|-------|
| **0.2.x** | **0.2.0** | `shell`, `slots` (`chat_toolbar`, `settings_panel`, `role_detail`, etc.), base `theme` / `layout` (see schema) | historical baseline |
| **0.3.x** | **0.3.0** | extended theme/layout fields per release notes | lower hosts usually **ignore unknown fields** |
| **0.4.x** | **0.4.0** | full **`sidebar`**, **`chat.header`**, etc. need host **directory bootstrap** for those slots ([DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)) | slot names must match host `pluginStore` constants |
| **0.5.x** | **0.5.0** | portrait catalog / `visual_presentation` export aligned with host `display_metrics` and voice side-channel `ui.json` slot seeds | see [CHANGELOG.en.md](../CHANGELOG.en.md) `[0.5.0]` |
| **dev** | **same dev** | schema and host `UiConfig` on the same branch | local pairing only |

---

## Upgrade and downgrade behavior

1. **Host older than editor target**
   - Unknown **`ui.json`** keys: usually **silently ignored** when models use **`serde` defaults + optional fields**; if a release rejects unknown keys, see that version’s `CHANGELOG`.
   - Declared but unimplemented slots: may **not render** or **do nothing** until the host is upgraded.

2. **Editor older than host**
   - New host slots / theme keys may be uneditable in the old editor; **edit `ui.json` manually** against [ui.json.schema.json](../creator-docs/role-pack/ui.json.schema.json).

3. **Role blueprint and legacy configuration compatibility**
   - Stable v4 uses `pipeline.ocblueprint` `slot_registry` (plus optional `runtime_config` / `extensions`) as the disk authority. `settings.json` / `plugin_backends` are legacy migration inputs only. Compatibility is governed by **`min_runtime_version`** and host `load_role`; see [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md) and [CHANGELOG.en.md](../CHANGELOG.en.md).

---

## In-repository module compatibility contract

OCLive capability is bounded by the complete module chain, not by the newest individual component. Every capability update must check:

```text
pack/plugin assets → kernel contract and orchestration → Tauri/Bridge → distros/shared → Chat Pro/Theater → Vue/iframe/legacy fallback
```

| Boundary | Current mechanism | Constraint |
|----------|-------------------|------------|
| Pack ↔ kernel | `schema_version`, `min_runtime_version`, `oclive_validation` | Prefer optional fields with defaults; Breaking changes keep read compatibility for at least one release cycle |
| Rust host ↔ kernel | `oclive_kernel_host::OcliveKernel` + `KernelErrorBody` codes | Stability means a Rust source-level facade, not a C ABI. Add APIs compatibly; breaking signatures follow host semver / CHANGELOG. Internal `AppState` is not public contract |
| Kernel ↔ frontend | `api_version`, Rust DTOs, `distros/shared/src/api` mirrors, error-code drift gate | DTO/command changes must update consumers and contract tests; Rust compilation alone is insufficient |
| Tauri ↔ directory plugin | manifest `schema_version: 1`, slot names, `bridge.invoke`, events, and `rpcMethods` allowlists | Plugin `version` identifies the plugin only; it is **not a host compatibility range**. Unexpressed host requirements need a fallback or the Breaking/RFC process |
| Chat Pro ↔ plugin UI | iframe `entry` plus optional `vueComponent`, shared `PluginSlotEmbed` | When both entries exist, they must expose the same capability; updating Vue alone is incomplete |
| Chat Pro shells ↔ shared | Fluent and Tool consume shared stores/composables | Layout may differ, but contracts, state ownership, events, and cancellation semantics must not drift |

**Structural gate**: `npm run check:module-compat` compares kernel/frontend slot registries, bundled manifests, Vue/iframe files, RPC timeout declarations, and plugin-index versions. It does not prove sidecar, device, or real-WebView behavior; those still require targeted integration/smoke tests.

Follow [`AI_CHANGE_BOUNDARIES.md`](../handoff/AI_CHANGE_BOUNDARIES.md) G17 for associated changes and completion claims, and [`BREAKING_CHANGE_PROCESS.md`](../handoff/BREAKING_CHANGE_PROCESS.md) for incompatible changes.

---

<a id="six-slot-minimal-compatibility-draft"></a>

## Six-slot Base / minimal logical role compatibility review scope (Adopted · 2026-10-09)

**Status:** the maintainer reviewed and adopted these bounded compatibility review rules on 2026-10-09 for changes to the existing public layer. This is not a new stable-release guarantee. Implementation baseline: `d9847b70c8c6164ab4715813f7e55bcfb6bb20ed`. No Rust API, validator, version or execution semantics changed. Responsibilities and capability semantics remain in [MODULE_MAP §0.4–0.9](../handoff/MODULE_MAP_AND_HANDOFF.md#six-slot-base-extension); the logical role definition remains in [ROLE_PACK_BOUNDARY](../handoff/ROLE_PACK_BOUNDARY.md#01-已确认的最小逻辑-contract). Listing a symbol does not cover every export of its crate.

### Bounded public inventory

| Review subject | Existing public entry and source | Bounded scope |
|---|---|---|
| Six calls | `oclive_kernel_contracts::{MemoryBase, EmotionBase, EventBase, PromptBase, LlmBase, AgentBase}`; [call binding](../kernel/crates/oclive_kernel_contracts/src/slot_base.rs) | Existing single-method names, signatures, borrows, normal results and confirmed capability semantics; no fixed six-slot schedule |
| Async binding | `BaseCallFuture` in that crate; same source | Existing boxed local future usable through `dyn`; no mandatory `Send`, `Sync` or `'static`, and no inferred cancellation, rollback or retry safety |
| Requests and failure carrier | `oclive_kernel_types::{MemoryBaseRequest, EmotionBaseRequest, EventBaseRequest, PromptBaseRequest, LlmBaseRequest, AgentBaseRequest, BaseCallError, BaseCallErrorKind}`; [data binding](../kernel/crates/oclive_kernel_types/src/slot_base.rs) | Existing fields/types, material versus purpose, normal results versus failed calls; no HTTP/SSE, permission handle or product terminal state |
| Minimum logical definition | `oclive_validation::MinimalRoleDefinition`, re-exported as `oclive_kernel_types::MinimalRoleDefinition`; [definition](../kernel/crates/oclive_validation/src/minimal_role.rs) | Nonblank `persona_prompt` and at least one `visual_assets` reference, each nonblank; preserve authored content and asset order without seven portraits, filenames, URI schemes or rich `Role` |
| Logical validation and JSON projection | `oclive_validation::{validate_minimal_role_definition, parse_minimal_role_definition}`; same source | Existing logical validity and accepted/rejected projection inputs; unknown fields are ignored, not retained. Passing proves neither existence, safety nor rendering of assets and defines no universal disk pack |

Legacy reference-Host ports, `AppState`, local role-loading/session DTOs, concrete shared-consumer assembly, rich packs, directory-plugin protocols, network bridges and storage do not become stable small-Kernel APIs through this table. The shared runtime retains minimum adaptation; distros own resources, lifecycle and extensions and may set separate compatibility rules. Advanced emotion-driven long-term memory remains an [optional extension under the confirmed deferral](../creator-docs/architecture/DESIGN_DECISIONS.md#emotion-memory-extension-deferred), not a Base obligation.

### Compatible / breaking review interpretations

| Example change | Interpretation and required check |
|---|---|
| Replace internal retrieval/decoding or fix a basic defect while preserving public input, result and failure commitments | May preserve the contract; use actual consumers and defect regressions. Algorithm changes are not automatically compatible or proof of model quality |
| Add required Base methods/supertraits, strengthen `Send` / `Sync` / `'static`, or require moving the local future across threads | Existing valid implementations may stop compiling; treat as source Breaking, not an internal refactor |
| Add an `Option` field to a current public request struct | External struct literals can still stop compiling; assess Rust-source compatibility separately from optional JSON-field compatibility |
| Rename/remove fields, change borrows/results, or reinterpret normal empty results as failures or not-called | Review source and behavior Breaking separately; `Ok`, empty text and capability return still do not prove product success or invocation termination |
| Add a reason to `#[non_exhaustive] BaseCallErrorKind` | Existing binding permits an external unknown branch; still review behavior and adapters. Unknown means non-normal completion, not retryable, effect-free or stopped; `detail` / Display remains no machine protocol |
| Require rich fields in the minimum projection, reject currently accepted unknown fields, or inject relation/seven-dimensional personality defaults | Alters minimal input or validation; review data/behavior Breaking. Ignoring unknown fields does not promise lossless rich-pack preservation |
| Add an independent optional extension | Specify extension inputs, outputs, associations, authorization and required versions; existing Base-only implementations acquire no extra obligations. Do not report required enhancements satisfied through silent degradation or activate unselected enhancements implicitly |

State whether compatibility concerns **Rust source, logical data, behavior or a particular transport**. One change can preserve one layer and break another. Existing Rust requests/errors carry no serde wire and a shared logical role creates neither a C ABI nor a universal network protocol.

### Review, migration and bounded verification

Reuse the existing [Breaking process](../handoff/BREAKING_CHANGE_PROCESS.md), not a second approval system. The proposer lists affected symbols, old/new behavior, downstreams and migration; Host/module authors update actual adapters and tests; role-converter authors own distro-to-logical mapping and resource checks. The maintainer reviews the need, feasibility and duration of compatibility layers for each concrete Breaking change. Apply the existing release-cycle read-compatibility rule only to data where it actually applies; do not invent runtime compatibility for Rust traits.

Reuse the [Base-only fixture](../kernel/crates/oclive_kernel_contracts/tests/base_only_fixture.rs), [request/error tests](../kernel/crates/oclive_kernel_types/src/slot_base.rs), [minimum logical validation](../kernel/crates/oclive_validation/src/minimal_role.rs) and [replaceable six-slot consumer case](../kernel/crates/oclive_kernel_runtime/tests/minimal_role_six_slots.rs). Actual public-Rust changes run affected regressions plus G8 workspace doctests and a named old-surface consumer; JSON/validation changes check accepted and rejected examples. A prior full CI or a test file alone proves no universal future plug-in compatibility. Stop when evidence distinguishes this change's compatibility effect; do not enumerate all Hosts, algorithms and devices.

**Versions and effect:** retain current [independent artifact version rules](development/RELEASE_VERSIONING.md). Do not substitute crate `0.2.0`, proposal numbering, `API_VERSION` or pack schema for one another; add no six-slot negotiation or 1.0 announcement. This inventory and interpretation have been adopted; each actual change still identifies its impact and passes applicable review and verification. Actual version bumps, retirement of old interfaces and new execution semantics belong to their concrete changes and approvals. These rules replace no concrete approval, prove no stable release or acceptance of every downstream, and close none of the other K-CORE-BOUNDARY-01 scopes.

---

## One-page external compatibility (host / editor / launcher / packs / kernel / CLI)

| Component | Version source | Relation to host | Notes |
|-----------|----------------|------------------|-------|
| **oclivenewnew (host)** | root `package.json` / `distros/desktop-tauri/Cargo.toml` | — | snapshot **0.5.2** |
| **oclive_kernel_runtime** | `kernel/crates/oclive_kernel_runtime/Cargo.toml` | path dep for host and headless HTTP; `SendMessageResponse.api_version` (`API_VERSION` **u32**, currently **1**), `RUNTIME_API_VERSION` (string **0.2.0**) | OOCP / black-box scripts: `creator-docs/testing/OOCP_TEST_SUITE.md` |
| **oclive_kernel_host** | `kernel/crates/oclive_kernel_host/Cargo.toml` | HTTP, Tauri, and `OcliveKernel` share complete `process_message` / SQLite / plugin / Event Ring orchestration | **0.2.0** today; integrate through `OcliveKernel`, not direct `AppState` assembly |
| **oclive-cli** | `kernel/crates/oclive-cli/Cargo.toml` | without `--kernel-source`, emits a serde stub; a linked `library` gets complete `OcliveKernel` path dependencies/facade | [OCLIVE_CLI_GUIDE.md](cli/OCLIVE_CLI_GUIDE.md) |
| **oclive-pack-editor** | sister `package.json` | writes `distros/chat-pro/roles/{id}/`; **`ui.json`** matrix above | `HOST_RUNTIME_VERSION` must match host `version` |
| **oclive-vscode** | sister `package.json` | spawn/attach **`kernel_server --api`**; `distro.oclive.toml` mirrors `examples/distro-profiles/vscode.oclive.toml` | **0.5.0** today; host **0.5.2** recommended |
| **oclive-launcher** | sister `package.json` | sets **`OCLIVE_ROLES_DIR`**, optional model name, zip install; **does not** replace host contract | [launcher README](https://github.com/linkaiheng2233-cyber/oclive-launcher/blob/main/README.md) |
| **role packs** | `manifest.json` (`schema_version`, `min_runtime_version`) | older hosts may refuse load or degrade | [PACK_VERSIONING.md](role-pack/PACK_VERSIONING.md) |
| **host SQLite** | `kernel/crates/oclive_kernel_host/migrations/*.sql` | ships only with **host** releases; do not downgrade DB after a forward migration unless `CHANGELOG` says so | breaking migrations need bilingual `CHANGELOG` + this table |

On breaking changes: update **`CHANGELOG.md` / `CHANGELOG.en.md`**, this matrix, **`oclive_validation`** (if touched keys), and sister-repo README minimum versions.

### Release review (maintainers)

1. Verify snapshot semver: root **`package.json`**, **`distros/desktop-tauri/Cargo.toml`**, **`oclive_kernel_runtime`**.
2. Follow [CONTRIBUTING](../CONTRIBUTING.en.md) and [release versioning](development/RELEASE_VERSIONING.md) for external notes when contracts or sister dependencies change.
3. **HTTP / OOCP**: if `API_VERSION` or `RUNTIME_API_VERSION` changes, sync tests and docs (`creator-docs/testing/OOCP_TEST_SUITE.md`).

Headless HTTP authentication is part of the host launch contract: `--api` requires `OCLIVE_API_TOKEN` by default, and callers send `x-oclive-api-token` on every route except `/health`. Never use `OCLIVE_API_ALLOW_UNAUTHENTICATED=1` with production or persistent data.

---

## How to read versions

| Product | Where |
|---------|--------|
| **Host** | in-app About (if present); install name; repo **`package.json`** / **`CHANGELOG.md`** |
| **Editor** | editor About; repo **`package.json`** |

---

## Remote LLM env (pointer)

Env matrix for Remote LLM (`OCLIVE_LLM_BACKEND`, `OCLIVE_REMOTE_LLM_*`, `OCLIVE_LLM_CLOUD_API_STYLE`, OpenAI aliases) lives in the Chinese SSOT [REMOTE_PLUGIN_PROTOCOL.md §2.0](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md); EN protocol page links there.

## Related

- [Historical acceptance evidence: A5_CLOSURE_SUMMARY.md](../handoff/archive/A5_CLOSURE_SUMMARY.md)
- [ui.json.schema.json](../creator-docs/role-pack/ui.json.schema.json)
- [DIRECTORY_PLUGINS.md](plugin-and-architecture/DIRECTORY_PLUGINS.md)
- [REMOTE_PLUGIN_PROTOCOL.md](../creator-docs/plugin-and-architecture/REMOTE_PLUGIN_PROTOCOL.md) §2.0 — Remote LLM env matrix
- [CHANGELOG.en.md](../CHANGELOG.en.md)
