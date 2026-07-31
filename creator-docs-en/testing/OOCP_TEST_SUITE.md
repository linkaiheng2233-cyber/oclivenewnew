# OOCP protocol test suite (16 default scenarios; optional S13 / S14)

**Status (`main`)**: Checked in under **`examples/oocp-test-suite/`** (`run.mjs` + JSON schema); CI workflow **`.github/workflows/ci.yml`** job **`oocp-test-suite`** builds `oclivenewnew-tauri --features dual_core`, starts the **`--api` HTTP** service, polls **`GET /health`**, runs **`node run.mjs --include-dual-core`** (S13/S14), then runs **`scripts/e2e-core-api-restart.mjs`** (restart process, chat again; failure fails the job). The **`frontend`** job runs **Playwright + `vite preview` first-screen smoke** (**A1.1b**) on **Ubuntu** after **`npm run build`** (Windows `frontend` skips Playwright).

## A1.1 PoC: core HTTP restart smoke

- **Script:** repo root **`scripts/e2e-core-api-restart.mjs`** (Node 20+ `fetch`, no extra npm deps).  
- **Behaviour:** **start `--api` → `/health` → `POST /chat` → terminate → start again → `/health` + `POST /chat`** on the same port; both cycles must pass. Defaults to **`OCLIVE_HTTP_API_MOCK_LLM=1`** (**no Ollama**).  
- **Local:** after `cargo build -p oclivenewnew-tauri`, from repo root **`npm run test:e2e:core-api-restart`** (or set `OCLIVE_ROLES_DIR` / `OCLIVE_E2E_PORT` / `OCLIVE_E2E_BINARY`).  
- **Scope:** **host process** “restart and recover” for HTTP (**A1.1a**); **`vite build` + `vite preview` + Playwright** first-screen smoke is **A1.1b** below (CI **`frontend`** job). **Installer / native Tauri window / full WebDriver** remains a separate engineering item in [PRODUCT_LINE_TASK_BUCKETS.md](../../handoff/PRODUCT_LINE_TASK_BUCKETS.md) **§四**.

## A1.1b: Web preview shell (Playwright)

- **Specs:** repo root [`distros/chat-pro/e2e/preview-shell.spec.ts`](../../distros/chat-pro/e2e/preview-shell.spec.ts) (`#app` mount + document title).  
- **Local:** `npm run build && npm run test:e2e:preview` (first time: `npx playwright install chromium`; Linux: `npx playwright install --with-deps chromium`).  
- **CI:** **`frontend`** job on **Ubuntu** starts **`vite preview`** in the background (default port **4180**), sets **`PW_TEST_USE_EXTERNAL=1`**, then runs **`npm run test:e2e:preview`**; **`PLAYWRIGHT_DISABLE_HEADLESS_SHELL=1`** reduces extra browser downloads.

## How to run

- **Locally**: see [`examples/oocp-test-suite/README.md`](../../examples/oocp-test-suite/README.md).
- **Environment variables**:
  - `OCLIVE_API_BASE`: default `http://127.0.0.1:8420`
  - `OCLIVE_OOCP_ROLE_PATH`: role pack directory (default `<repo>/distros/chat-pro/roles/mumu`)
  - **`OCLIVE_HTTP_API_MOCK_LLM=1`** (with `--api` only): in-memory store + fixed-reply mock LLM; **enabled by default in CI**, no local Ollama required.
  - **`OCLIVE_API_TOKEN`**: required when starting headless `--api`; test scripts add `x-oclive-api-token` automatically while `GET /health` remains public. The restart smoke test generates a random token when none is supplied.

## Scenario table (HTTP black-box)

| ID | Assertion focus |
|----|-----------------|
| S0 | `GET /health` → 200, body `ok` |
| S0b | `GET /health` JSON probe → `ok=true` with the startup-warning contract |
| S1 | `POST /chat` empty message → 400, `error.code=EMPTY_MESSAGE` |
| S2 | Invalid `role_path` → 400, `INVALID_ROLE_PATH` or a kernel load code (e.g. `ROLE_NOT_FOUND`) |
| S3 | `role_path=""` → 400 with error body |
| S4 | Valid chat → 200, top-level `reply` non-empty |
| S5 | Success body includes `personality_source` (`vector` \| `profile`) |
| S6 | When `session_id` is sent, it is echoed back |
| S7 | When `scene_id` is sent, response `scene_id` matches |
| S8 | Chinese + emoji user line → 200 |
| S9 | Long user line (400 chars) → 200 |
| S10 | Two consecutive rounds with same `session_id` → both 200 |
| S11 | Success body includes `api_version`, `schema`, `timestamp` |
| S12 | Error body `error.code` is a **string** (`KernelErrorBody`), not a JSON-RPC integer code |
| S15 | `POST /chat/stream` emits an SSE `token` and a final `done` payload with a non-empty `reply` |
| S16 | Fixed disabled/enabled fixtures assert visual-field omission and `visual_state_id` + image `performance_directive` output |

**Default suite:** `run.mjs` runs **S0, S0b, S1–S12, S15, and S16** (**16** scenarios). Dual-core scenarios are optional: **S13** (experimental failure silently falls back to Stable and still returns `reply`) and **S14** (experimental pipeline happy path with supported method DAG still returns `reply`). Enable them with `--include-s13` / `--include-s14`, `OCLIVE_OOCP_INCLUDE_S13=1` / `OCLIVE_OOCP_INCLUDE_S14=1`, or both at once via `--include-dual-core` / `OCLIVE_OOCP_INCLUDE_DUAL_CORE=1`.

## Conformance report

`npm run test:json` emits JSON; field set is defined in `examples/oocp-test-suite/schemas/oclive.protocol_conformance_report.v1.schema.json`. The report includes:

- a `dual_core` summary section (enabled flag, S13/S14 switches, executed dual-core scenarios),
- a `ci_context` section (generation timestamp plus CI metadata: `github_run_id` / `github_sha` / `github_ref`).

This makes CI artifacts directly usable for audit trails and external presentation.

## Relationship to full OOCP

The main app **`--api`** mode is **HTTP** (`GET /health`, `POST /chat`), **without** a WebSocket method chain. This suite validates the **HTTP try-chat contract** and orchestration results; if WS semantics from the spec land later, extend scripts and CI steps under this directory.

**Doc alignment:** Root quick indexes use **S0–S12** for the base numbered segment. The current executable default also includes **S0b/S15/S16, for 16 scenarios total**, plus optional **S13/S14** dual-core scenarios. This page and `run.mjs` are the scenario truth.

## Test stack overview

See [`OVERVIEW.md`](./OVERVIEW.md) in the same folder.

---

[中文](../../creator-docs/testing/OOCP_TEST_SUITE.md)
