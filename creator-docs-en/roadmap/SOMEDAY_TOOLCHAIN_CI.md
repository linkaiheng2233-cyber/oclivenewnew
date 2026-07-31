# OCLive Domain-Aware CI · Staged Baseline

> **Status (2026-08-01):** Stage 1 is in development. The detailed design SSOT is the [Chinese document](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md); module boundaries remain in [`MODULE_MAP_AND_HANDOFF.md` §12.7](../../handoff/MODULE_MAP_AND_HANDOFF.md#127-ci-影响元数据与脚手架边界).

[中文](../../creator-docs/roadmap/SOMEDAY_TOOLCHAIN_CI.md)

OCLive keeps conventional layered CI, test pyramids, and merge gates, then adds a deterministic domain-aware planner. It uses paths only to locate directly changed modules and uses versioned metadata plus a centrally owned impact graph to propagate semantic effects. Unknown paths, invalid metadata, and unsupported required extensions fail safe to the full validation set for the active policy.

The three inputs have separate ownership:

- `oclive.module.json` declares logical `runtime_requires`, physical `resource_claims`, additive `declared_affects`, and trusted `validation_profiles`; it cannot contain arbitrary commands or workflow triggers.
- The central impact map owns path bindings, mandatory impact edges, high-risk overrides, supported extensions, and full-fallback policy. Third-party declarations may widen but never narrow it.
- The trusted validation catalog owns tiers, gate strength, platforms, trust levels, finite local reproducer command IDs, and mappings to existing remote workflow jobs. Stage 1 reports these coordinates but executes neither commands nor jobs. Main-repository workflows retain control of runners, secrets, caching, concurrency, and timeouts.

Stage 1 runs in shadow mode: it emits a stable `plan.json`, GitHub Job Summary, and artifact while every existing CI job still runs. Stage 2 compares proposed selections with full-CI results. Selective PR execution is allowed only after evidence establishes safe low-risk classes; merge gates remain pre-merge safeguards, while long-running hardware, soak, and performance work belongs to Nightly/Release.

The scaffold is only an assistant. Once the descriptor contract is stable, it may generate and validate module metadata, list catalog-approved profiles, and invoke the same planner. It may not own workflow orchestration, inject shell commands, approve secrets or self-hosted GPU runners, create a second impact/resource schema, or treat generation as CI success.

In short: **the scaffold produces and preflights knowledge that CI understands; CI consumes that same knowledge in a trusted environment and produces evidence.**
