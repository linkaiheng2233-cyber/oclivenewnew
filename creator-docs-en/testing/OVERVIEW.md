# Where tests live (main repo vs pack editor)

This document pins **what is tested and in which repository**, so it does not conflict with the root `README` / `AGENTS` wording.

## Protocol and kernel (`oclivenewnew` main repo)

| Layer | Content | Location / command |
|-------|---------|---------------------|
| Rust unit and integration tests | Orchestration, `--api` HTTP routes, `process_message`, etc. | `cargo test` under `src-tauri/`; integration tests in `src-tauri/tests/` |
| OOCP-aligned HTTP black-box | Scenarios **S0–S11** (see [`OOCP_TEST_SUITE.md`](./OOCP_TEST_SUITE.md)) | `examples/oocp-test-suite/run.mjs`; CI job **`oocp-test-suite`** |
| Frontend smoke | Minimal Vitest guard | `npm run test:unit` (`src/smoke.test.ts`) |

## Components and plugin shell (`oclive-pack-editor`)

| Scope | Notes |
|-------|--------|
| **T05–T13** (Vue component tests, etc.) | Canonical source lives in the **pack editor** repo; the main repo does not duplicate the 42-case tree. |
| **T14–T20** (`official-vue-test-runner`, etc.) | Editor-built capability, wired to the workspace as a **directory plugin** pattern; see editor docs and plugin READMEs. |

The main app connects via pack format and HTTP / `invoke` contracts; component- and plugin-shell-level tests on the editor side are enough to cover the creator toolchain.
