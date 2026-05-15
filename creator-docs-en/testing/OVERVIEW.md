# Where tests live (main repo vs pack editor)

This document pins **what is tested and in which repository**, so it does not conflict with the root `README` / `AGENTS` wording.

## Protocol and kernel (`oclivenewnew` main repo)

| Layer | Content | Location / command |
|-------|---------|---------------------|
| Rust unit and integration tests | Orchestration, `--api` HTTP routes, `process_message`, **`invoke` hot path (nine `*_impl` chains)** ([`invoke_hotpath_matrix.rs`](../../src-tauri/tests/invoke_hotpath_matrix.rs), see [`handoff/INVOKE_HOTPATH_MATRIX.md`](../../handoff/INVOKE_HOTPATH_MATRIX.md)), etc. | `cargo test` under `src-tauri/`; integration tests in `src-tauri/tests/` |
| OOCP-aligned HTTP black-box | Scenarios **S0–S11** (see [`OOCP_TEST_SUITE.md`](./OOCP_TEST_SUITE.md)) | `examples/oocp-test-suite/run.mjs`; CI job **`oocp-test-suite`**; plus **`scripts/e2e-core-api-restart.mjs`** (process-restart smoke, **A1.1a**) |
| Frontend smoke | Vitest guard + **`vite preview` + Playwright** shell (**A1.1b**; **CI: Ubuntu `frontend` only**) | `npm run test:unit`; `npm run build && npm run test:e2e:preview` ([`e2e/preview-shell.spec.ts`](../../e2e/preview-shell.spec.ts); see CONTRIBUTING **Windows** note) |

## Components and plugin shell (`oclive-pack-editor`)

| Scope | Notes |
|-------|--------|
| **T05–T13** (Vue component tests, etc.) | Canonical source lives in the **pack editor** repo; the main repo does not duplicate the 42-case tree. |
| **T14–T20** (`official-vue-test-runner`, etc.) | Editor-built capability, wired to the workspace as a **directory plugin** pattern; see editor docs and plugin READMEs. |

The main app connects via pack format and HTTP / `invoke` contracts; component- and plugin-shell-level tests on the editor side are enough to cover the creator toolchain.

---

[中文](../../creator-docs/testing/OVERVIEW.md)
