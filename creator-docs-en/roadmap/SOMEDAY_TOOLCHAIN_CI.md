# OCLive Domain-Aware CI · Staged Baseline

> **Status (2026-08-21):** Stage 3.1 uses `domain-aware-pr-v2` for real development feedback. Warning-free, non-fallback, non-shadow draft PR plans with non-empty scope and jobs execute only the trusted plan and publish `ci-draft-gate`, which cannot satisfy main's required check. Marking a PR ready retriggers the same commit through `ready_for_review`; only the already-evidenced docs-only canary remains selective, while every other ready PR runs full and publishes `ci-gate`. Pushes, high-risk or unknown paths, and planner failures remain full. Every full run automatically emits same-workflow-SHA Compare JSON/Markdown retained for 90 days; selective runs are execution evidence only and never authoritative false-negative comparisons. The simulation corpus now has **21** passing scenarios (**18 targeted / 3 fail-safe**), including Unicode/NUL path transport, mixed domains, merge policies, and Dependabot cadence. The detailed design SSOT is the [Chinese document](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md); module boundaries remain in [`MODULE_MAP_AND_HANDOFF.md` §12.7](../../handoff/MODULE_MAP_AND_HANDOFF.md#127-ci-影响元数据与脚手架边界).

[中文](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)

OCLive keeps conventional layered CI, test pyramids, and merge gates, then adds a deterministic domain-aware planner. It uses paths only to locate directly changed modules and uses versioned metadata plus a centrally owned impact graph to propagate semantic effects. Unknown paths, invalid metadata, and unsupported required extensions fail safe to the full validation set for the active policy.

The three inputs have separate ownership:

- `oclive.module.json` declares logical `runtime_requires`, physical `resource_claims`, additive `declared_affects`, and trusted `validation_profiles`; it cannot contain arbitrary commands or workflow triggers.
- The central impact map owns path bindings, mandatory impact edges, high-risk overrides, supported extensions, and full-fallback policy. Third-party declarations may widen but never narrow it.
- The trusted validation catalog owns tiers, gate strength, platforms, trust levels, finite local reproducer command IDs, and mappings to existing remote workflow jobs. The planner never executes commands; Stage 3 only lets the trusted main-repository workflow select existing jobs by those coordinates. Main-repository workflows retain control of runners, secrets, caching, concurrency, and timeouts.

Expensive validation also has a single execution owner. In the ownership slice integrated on 2026-10-12, `kernel-contracts` owns complete contracts/types lint and tests, while `rust` keeps the remaining non-CLI workspace. CLI still exclusively owns serial nested-Cargo E2E, Dimension 5 owns `cargo audit`, and frontend owns lint, Vue/TypeScript type checking, unit tests, and its build. Multiple validator coordinates may map to one trusted job without duplicating an identical command. Stage 3.1 keeps the same safe-draft and ready-docs-only selection rules; other ready PRs and pushes remain full. `tier=nightly` groups run separately.

The active workflow emits `plan.json`, an execution-policy artifact, and a Job Summary. For PRs, the planner, impact contracts, execution policy, and gate verifier run from the trusted comparison-base commit; PR code supplies changed paths and the content under test but cannot rewrite its own selection result. A shadow plan can still be requested explicitly and is never eligible to skip jobs. Planner failure starts every validation responsibility through fail-safe conditions and still fails the active gate; full fallback, warnings, pushes, and non-PR events also remain full. Draft PRs publish `ci-draft-gate`; `ready_for_review` retriggers the current commit, where non-document PRs run full and publish the branch-protected `ci-gate`. `.github/workflows/nightly-advisory.yml` continues to own Loom, fuzz, native-window, visual-smoke, and threshold-free performance evidence. Automatic failure reruns are disabled; the separate manual workflow accepts a run ID and reruns failed jobs once only for an allowlisted Rust-matrix failure set.

Shadow evidence has two distinct levels. `data/ci/shadow-scenarios.v1.json` and `npm run ci:shadow-samples` provide routing simulations only, not validator execution or zero-false-negative proof. The **21** scenarios (**18 targeted / 3 fail-safe**) retain their original scope; the 2026-10-11 slice updates only public-owner validator/job coordinates. Shared frontend, role-pack, and directory-plugin scenarios now select nine validators (the original eight plus the new owner), still including Rust because of the conservative frontend impact cycle. This does not authorize removing an edge. Real Compare binds `plan.json`, `execution.json`, and terminal remote jobs from one workflow SHA; full runs upload 90-day artifacts, while incomplete snapshots are observational and selective runs are not authoritative Compare samples.

<a id="kernel-host-ci-first-slice"></a>

## Kernel / Host responsibility: first local slice

The responsibility model starts with Kernel public contracts, then generic Host integration, concrete Host/distro behavior, and the selected release combination. Architecture ownership stays in [MODULE_MAP §0.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-responsibilities). Model-specific prompts, repair policies, emotion presentation, and tuning belong to their implementation/Host; model size is not itself a module boundary. Evidence for these behaviors must name the model and conditions. Dependency propagation remains necessary when a public contract changes.

The 2026-10-11 preparation is bound to `24f62303e9bdea37061fe44acc7a0830dc519f98`. It checks eight grouped entry points at D1 depth, rather than reclassifying every test. The [Chinese design](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#kernel-host-ci-first-slice) owns the detailed mapping:

| Existing group | Responsibility and retained limitation |
|---|---|
| `kernel-contracts` / `rust-workspace` | Public Base/minimal-role checks can have a separate local entry. The profile currently also reaches Host/combination jobs; the Rust job builds frontend and Tauri resources. Neither the whole profile nor all rich exports are pure Kernel. |
| Shared minimal consumer and external Memory example | Finite generic integration evidence outside the small Kernel; not all runtime implementations or a mandatory six-slot turn. |
| `kernel-host-dual-core` / `rust-arm64-cross` | Concrete Host feature/target checks; no dual-core unfreeze or ARM device acceptance. |
| `cross-host-e2e` / `oocp-test-suite` | Existing reference Host and distro combinations, including reference role paths; not universal Host compliance. |
| `frontend` / `plugins-index` / `remote-plugin-demo` | Distro presentation and finite plugin/adapter examples; no automatic IPC, audio, or plugin-function acceptance. |
| `cli` / scaffold | Generation/tool checks are separate from the explicitly enabled `diagnostics-host` tests in the same job. Generation is not module-function acceptance. |
| Layering / Dimension 5 / Cargo and npm audit / stale paths | Cross-cutting repository governance; not another Kernel capability and not a reason to mechanically split jobs. |
| Nightly Loom / fuzz / CLI bench / visual / native Tauri | Target-dependent evidence. Tier describes scheduling, not semantic ownership; deeper splitting and performance gains are unknown in this preparation. |

The first implementation is **`npm run check:kernel-base`**, run from the OCLive source root. It needs Node (per package.json), Cargo, and Clippy, without preparing frontend or desktop resources. [`check-kernel-base.mjs`](../../scripts/check-kernel-base.mjs) reuses the native-command helper and stops with a nonzero exit on failure, a missing tool, or timeout. Each step has a 20-minute limit; the command appears first and Cargo output appears after the step finishes. Generated projects still run their own `cargo check` / `cargo test`; this entry does not automatically check external modules. It runs these existing commands in order:

```sh
cargo test --locked -p oclive_kernel_contracts --test base_only_fixture
cargo test --locked -p oclive_kernel_types --test minimal_role_contract
cargo clippy --locked -p oclive_kernel_contracts -p oclive_kernel_types --all-targets -- -D warnings
cargo test --locked -p oclive_kernel_contracts -p oclive_kernel_types --doc
```

The two crates do not depend on Host/runtime/Tauri, but retain rich compatibility exports. Clippy/doctest cover their current compilation surface, not an assertion that every export is small Kernel. The entry covers existing Base/minimal-role contracts and compilation examples without preparing desktop resources, models, or a database. It does not cover every Kernel hard boundary, module quality, shared-consumer execution, Host business, or all distros. Public API changes still require the applicable explicit workspace doctest.

The [slice report](../../handoff/debt-marathon/waves/WAVE-20261011-KERNEL-BASE-LOCAL-CHECK.md) records the actual chain, stop-on-failure regression, and dependency boundary; the [local gate matrix](../../human-docs-en/08_PR_GATE_MATRIX.md) explains narrow versus full checks. Stop at the usable local entry. Do not copy Base tests, add DTOs/traits, or manufacture six equivalent wrappers. A later remote slice must synchronize the trusted catalog, local command, workflow ownership, and repository contract tests. Retain existing combination regressions, prove coverage/unique ownership before moving tests out of `rust`, and do not call duplicated expensive execution successful layering. New ready-selective classes still require same-SHA real Compare and the formal gate. This slice changes no workflow, catalog/schema, impact edges, required context, Push/ready policy, public API, or freeze, and closes no debt.

<a id="ci-layering-development-handoff"></a>

## Layering handoff: separate execution ownership from selection policy

The planner, trusted catalog, execution policy, and final gate already exist. Execution ownership assigns a check to a responsibility group; selection policy decides which groups a change may skip. Giving public foundation checks an entry independent of desktop resources does not authorize PRs to skip Host regressions.

The [main workflow](../../.github/workflows/ci.yml) and [execution policy](../../scripts/ci-execution-policy.mjs) currently enforce:

| Trigger / condition | Execution scope | Gate |
|---|---|---|
| Safe draft PR: non-shadow, warning/fallback-free, non-empty scope and jobs | Trusted selected groups | `ci-draft-gate`, never main's required check |
| Ready PR whose direct modules are only promoted `oclive.docs`, with the same safety conditions | Docs canary selection | `ci-gate` |
| Other ready PR | All main-CI groups | `ci-gate` |
| Push to main/master, or manual main-workflow dispatch | All main-CI groups | `ci-gate`; pushing an ordinary backup branch alone does not trigger this workflow |
| Planner warnings, unknown/high-risk fallback, or planner failure | Full; a planner failure also fails the final gate | Successful full validation cannot mask planner failure |

Nightly remains separate. The first ownership slice defines 14 validation jobs: public contracts, remaining Rust, and frontend each expand across two platforms, giving 19 configured jobs with the planner and gate. It was integrated and pushed at main `12bc7dc8` on 2026-10-12. Its formal conclusion must use that SHA's actual CI result, not the old 17-job baseline's green status; counts are not quality or performance targets.

The integrated first slice synchronizes the existing catalog, complete local entry, workflow, and [repository contracts](../../kernel/crates/oclive_ci_plan/tests/repository_contract.rs). It retains Push/ready policy, required context, full migrated test targets, and remaining regressions. Shared dependencies may still compile; this is different from running the same tests twice. More jobs do not mean all layering is complete.

Evaluate wider ready selection separately, after ownership is stable, using real same-SHA Compare and contract evidence. Local speed, one successful full run, or a responsibility table cannot justify removing impact edges. Cache research, performance matrices, and deeper unclassified ownership are not prerequisites for this first slice. Keep targeted development checks and applicable full acceptance at a thematic milestone.

The [historical handoff snapshot](../../handoff/debt-marathon/waves/WAVE-20261011-CI-BASELINE-HANDOFF.md) records the earlier classified commits and formal main evidence; the [ownership slice report](../../handoff/debt-marathon/waves/WAVE-20261011-CI-CONTRACT-OWNER.md) retains implementation and then-local checks. This slice is now integrated/pushed at main `12bc7dc8`, as recorded in the [three-layer handoff](../../handoff/debt-marathon/waves/WAVE-20261012-CI-THREE-LAYER-INTEGRATION.md); exact-SHA formal results are disclosed separately in final delivery. The subsequent default reference-Host entry remains local and does not borrow main's acceptance. K-CI-IMPACT-01 remains In progress.

<a id="ci-responsibility-map"></a>

## Maintainer handoff: first integrated execution-owner slice

The 2026-10-11 confirmed architecture and source entry points belong to [MODULE_MAP §0.3.1](../../handoff/MODULE_MAP_AND_HANDOFF.md#kernel-reference-host-distro-boundaries). Shared minimal-role consumption remains outside the small Kernel. A distro may reuse the current reference Host or a developer's own Host; the discussion name “7B reference Host” changes no names or APIs and imposes no model restriction. Directory/crate names alone do not establish responsibility.

<a id="reference-host-local-check"></a>

**Default reference-Host feedback:** `npm run check:reference-host` runs Host-only all-targets Clippy with `--no-deps`, then unfiltered default `cargo test -p oclive_kernel_host`, both locked. Node/Cargo/Clippy are required; the existing native helper stops on failure, missing tools or timeout, with 20 minutes per step. It does not prepare frontend/Tauri resources or start a production Host/model; existing isolated test fixtures retain their behavior. Current Host defaults are empty, optional features are not enabled, and its manifest keeps `doctest=false`. This is not all-feature, desktop, distro or real-model acceptance. Existing workspace/dual-core/combination jobs remain responsible for those applicable scopes. No workflow/catalog or selection permission changes are made. The [slice report](../../handoff/debt-marathon/waves/WAVE-20261012-REFERENCE-HOST-LOCAL-CHECK.md) records the existing desktop-to-Host feature coupling; Host job migration is deferred until its tested conditions can be preserved.

For the distro frontend, reuse the existing **`npm run check:frontend`**: lint, typecheck, shared/ChatPro unit tests, Theater smoke and the ChatPro E2E build. It invokes no Cargo and does not replace desktop Rust/IPC, Linux preview E2E, packaging or real audio acceptance. No second frontend runner is added; distros using their own Host remain responsible for their actual product/combination checks.

The maintainer confirmed three main reading layers on 2026-10-12: public Kernel, the Host of a specialized runtime project, and distros built on that project. Their meaning belongs to the linked architecture SSOT. Shared support, developer tools, governance and release combinations remain separately identified. The first ownership slice passed the complete local chain and was integrated/pushed at `12bc7dc8`; formal CI must bind this frozen SHA. See the [integration handoff](../../handoff/debt-marathon/waves/WAVE-20261012-CI-THREE-LAYER-INTEGRATION.md) and final delivery for actual results. Remaining mixed Host/distro jobs are not marked as already separated.

The [Chinese CI mapping](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md#ci-responsibility-map) now assigns 14 integrated main validation jobs and the other four workflows; the original 13-job mapping is frozen at `5e483aa6`. `kernel-contracts` owns public contracts and existing compatibility checks; `rust` remains **MIXED** across shared support, reference Host, desktop, and governance. Dimension 5 and CLI also retain their existing mixed responsibilities. Frontend validates distro/components; OOCP and cross-host cover existing combinations. Layering, audits, paths, and index rules are governance. Nightly is a schedule; checksum artifacts are a server release check. Planner and gate are separate control-plane jobs.

**Integrated first slice:** complete contracts/types checks have a separate public-contract/compatibility owner on Linux and Windows. The crates retain rich and Stream compatibility surfaces, so their entirety is not small Kernel. `check:kernel-base` remains unchanged narrow feedback; the complete **`npm run check:kernel-contracts`** runs from the source root:

```sh
cargo clippy --locked -p oclive_kernel_contracts -p oclive_kernel_types --all-targets --all-features --no-deps -- -D warnings
cargo test --locked -p oclive_kernel_contracts -p oclive_kernel_types
```

It reuses the native helper, stops nonzero on missing tools/failure/timeout, and limits each step to 20 minutes. Node/Cargo/Clippy are required, without Host/frontend preparation. Unfiltered tests retain both library targets, `base_only_fixture`, `minimal_role_contract`, `runtime_event_stream_contract`, `runtime_event_stream_onebot_review`, and both applicable library doctest surfaces. Neither manifest currently defines its own features; default test conditions remain unchanged and Clippy retains all-targets/all-features. The entry does not test external author modules automatically.

Remaining workspace lint/tests exclude only CLI and the two migrated roots. Clippy uses `--no-deps` in both owners to assign lint to selected roots; their union retains the former non-CLI root set, including fuzz. Shared-consumer, Host, server, desktop, combination checks and workspace formatting remain. Dependencies can still compile in multiple jobs.

All five existing profiles selecting `rust-workspace` also select the new owner; the impact graph and ready promotion classes are unchanged. `ci-gate` requires the new owner when selected or full, and still permits legitimate docs-selection skips. A trusted base missing the new validator/job mapping, or failing to parse, uses the existing `trusted_policy_bootstrap` full fallback. The PR's catalog cannot approve selective execution. Required `ci-gate`, Push/other-ready full execution, draft/docs-canary rules, and unknown/full fallback stay intact.

The benefit is public checks independent of frontend/Tauri preparation, without a speed promise. Stop at this finite slice, not exhaustive separation or a job-count target. No APIs, runtime behavior, debt status, or freezes change; real models/TTS, platforms, combinations and crash windows retain separate requirements. The [ownership report](../../handoff/debt-marathon/waves/WAVE-20261011-CI-CONTRACT-OWNER.md) retains its original local evidence, and the [three-layer handoff](../../handoff/debt-marathon/waves/WAVE-20261012-CI-THREE-LAYER-INTEGRATION.md) records subsequent integration. Do not rewrite historical reports to manufacture new evidence. The [earlier mapping report](../../handoff/debt-marathon/waves/WAVE-20261011-CI-RESPONSIBILITY-MAP.md) retains its original documentation-only scope.

The scaffold is only an assistant. Once the descriptor contract is stable, it may generate and validate module metadata, list catalog-approved profiles, and invoke the same planner. It may not own workflow orchestration, inject shell commands, approve secrets or self-hosted GPU runners, create a second impact/resource schema, or treat generation as CI success. Scaffold discovery, source locking, command namespaces, and compatibility are owned by [`RFC_SCAFFOLD_PACKAGE_V1.md`](../rfc/RFC_SCAFFOLD_PACKAGE_V1.md); this document owns only the CI boundary.

In short: **the scaffold produces and preflights knowledge that CI understands; CI consumes that same knowledge in a trusted environment and produces evidence.**
